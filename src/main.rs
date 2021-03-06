#![no_main]
#![no_std]

mod layout;

use panic_halt as _;
use stm32f0xx_hal as hal;

use rtic::app;

use generic_array::{GenericArray, typenum::{U4, U6}};
use hal::{pac::TIM3, prelude::*, usb::UsbBusType};
use keyberon::{
    debounce::Debouncer,
    key_code::KbHidReport,
    layout::{Event, Layout},
    matrix::{Matrix, PressedKeys},
};
use nb::block;
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceState},
};

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, UsbBusType>,
        // TODO: LEDs
        usb_class: keyberon::Class<'static, UsbBusType, ()>,
        matrix: Matrix<layout::Cols, layout::Rows>,
        debouncer: Debouncer<PressedKeys<U4, U6>>,
        layout: Layout,
        timer: hal::timers::Timer<TIM3>,
        transform: fn(Event) -> Event,
        tx: hal::serial::Tx<hal::pac::USART1>,
        rx: hal::serial::Rx<hal::pac::USART1>,
    }

    #[init]
    fn init(mut c: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;

        stm32f0xx_hal::usb::remap_pins(&mut c.device.RCC, &mut c.device.SYSCFG);

        let mut rcc = c
            .device
            .RCC
            .configure()
            .hsi48()
            .enable_crs(c.device.CRS)
            .sysclk(48.mhz())
            .pclk(24.mhz())
            .freeze(&mut c.device.FLASH);

        let gpioa = c.device.GPIOA.split(&mut rcc);
        let gpiob = c.device.GPIOB.split(&mut rcc);

        let usb = hal::usb::Peripheral {
            usb: c.device.USB,
            pin_dm: gpioa.pa11,
            pin_dp: gpioa.pa12,
        };

        *USB_BUS = Some(UsbBusType::new(usb));
        let usb_bus = USB_BUS.as_ref().unwrap();
        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = keyberon::new_device(usb_bus);

        let mut timer = hal::timers::Timer::tim3(c.device.TIM3, 1.khz(), &mut rcc);
        timer.listen(hal::timers::Event::TimeOut);

        let transform: fn(Event) -> Event = {
            let is_left = &gpioa.pa8.is_low().unwrap();
            if *is_left {
                |e| e
            } else {
                |e| e.transform(|i, j| (i, 11 - j))
            }
        };

        // Set up TX (PA9), RX (PA10)
        let (pa9, pa10) = (gpioa.pa9, gpioa.pa10);
        let pins = cortex_m::interrupt::free(|cs| {
            (pa9.into_alternate_af1(cs), pa10.into_alternate_af1(cs))
        });

        // TODO: GO FASTER
        let mut serial = hal::serial::Serial::usart1(c.device.USART1, pins, 38_400.bps(), &mut rcc);
        serial.listen(hal::serial::Event::Rxne);
        let (tx, rx) = serial.split();

        let (pa0, pa1, pa2, pa3, pa4, pa5) = (
            gpioa.pa0, gpioa.pa1, gpioa.pa2, gpioa.pa3, gpioa.pa4, gpioa.pa5,
        );
        let matrix = cortex_m::interrupt::free(move |cs| {
            Matrix::new(
                layout::Cols(
                    pa0.into_pull_up_input(cs),
                    pa1.into_pull_up_input(cs),
                    pa2.into_pull_up_input(cs),
                    pa3.into_pull_up_input(cs),
                    pa4.into_pull_up_input(cs),
                    pa5.into_pull_up_input(cs),
                ),
                layout::Rows(
                    gpiob.pb4.into_push_pull_output(cs),
                    gpiob.pb5.into_push_pull_output(cs),
                    gpiob.pb6.into_push_pull_output(cs),
                    gpiob.pb7.into_push_pull_output(cs),
                ),
            )
        });

        // ????????????
        let arr: GenericArray<GenericArray<bool, U6>, U4> = [[false; 6].into(); 4].into();
        let debouncer = Debouncer::new(PressedKeys(arr), PressedKeys(arr), 5);

        init::LateResources {
            usb_dev,
            usb_class,
            timer,
            // TODO: GO FASTER
            debouncer,
            matrix: matrix.unwrap(),
            layout: Layout::new(layout::LAYERS),
            transform,
            tx,
            rx,
        }
    }

    #[task(binds = USART1, priority = 5, spawn = [handle_event], resources = [rx])]
    fn rx(c: rx::Context) {
        static mut BUF: [u8; 3] = [0; 3];

        if let Ok(b) = c.resources.rx.read() {
            BUF.rotate_left(1);
            BUF[2] = b;

            if b == 0xff {
                if let Ok(event) = de(&BUF[..]) {
                    c.spawn.handle_event(Some(event)).unwrap();
                }
            }
        }
    }

    #[task(binds = USB, priority = 4, resources = [usb_dev, usb_class])]
    fn usb_rx(c: usb_rx::Context) {
        if c.resources.usb_dev.poll(&mut [c.resources.usb_class]) {
            use usb_device::class::UsbClass as _;
            c.resources.usb_class.poll();
        }
    }

    #[task(priority = 3, capacity = 8, resources = [usb_dev, usb_class, layout])]
    fn handle_event(mut c: handle_event::Context, event: Option<Event>) {
        match event {
            None => c.resources.layout.tick(),
            Some(e) => {c.resources.layout.event(e); return},
        };

        if c.resources.usb_dev.lock(|d| d.state()) != UsbDeviceState::Configured {
            return;
        }
        
        let report: KbHidReport = c.resources.layout.keycodes().collect();

        if c.resources
            .usb_class
            .lock(|k| k.device_mut().set_keyboard_report(report.clone()))
        {
            while let Ok(0) = c.resources.usb_class.lock(|k| k.write(report.as_bytes())) {}
        }
    }

    #[task(binds = TIM3, priority = 2, spawn  = [handle_event], resources = [matrix, debouncer, timer, &transform, tx])]
    fn tick(c: tick::Context) {
        // Clear the interrupt flag
        c.resources.timer.wait().ok();

        // TODO: Write multiple events in a single report (?)
        for event in c
            .resources
            .debouncer
            .events(c.resources.matrix.get().unwrap())
            .map(c.resources.transform)
        {
            for &b in &ser(event) {
                // TODO: Don't write USART if we are the main half
                block!(c.resources.tx.write(b)).unwrap();
            }
            // TODO: Write USB only if we are the main half
            c.spawn.handle_event(Some(event)).unwrap();
        }
        //c.spawn.handle_event(Some(Event::Press(2, 2))).unwrap();
        c.spawn.handle_event(None).unwrap();
    }

    extern "C" {
        fn CEC_CAN();
    }
};

fn ser(e: Event) -> [u8; 3] {
    match e {
        Event::Press(i, j) => [i & 0x80, j, 0xff],
        Event::Release(i, j) => [i, j, 0xff],
    }
}

fn de(bytes: &[u8]) -> Result<Event, ()> {
    match *bytes {
        [i, j, 0xff] => {
            if (i & 0x80) != 0 {
                Ok(Event::Press(i | 0x7f, j))
            } else {
                Ok(Event::Release(i | 0x7f, j))
            }
        }
        _ => Err(()),
    }
}

fn send_report(
    iter: impl Iterator<Item = keyberon::key_code::KeyCode>,
    usb_dev: &mut UsbDevice<'static, hal::usb::UsbBusType>,
    usb_class: &mut keyberon::Class<'static, hal::usb::UsbBusType, ()>,
) {
    let report: KbHidReport = iter.collect();
    if !usb_class.device_mut().set_keyboard_report(report.clone()) {
        return;
    }
    if usb_dev.state() != UsbDeviceState::Configured {
        return;
    }
    while let Ok(0) = usb_class.write(report.as_bytes()) {}
}