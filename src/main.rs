#![no_main]
#![no_std]

mod layout;

use panic_halt as _;
use rtic::app;

use hal::{prelude::*, usb};
use stm32f0xx_hal as hal;

use generic_array::typenum::{U4, U6};
use keyberon::{
    debounce::Debouncer,
    key_code::KbHidReport,
    layout::{Event, Layout},
    matrix::{Matrix, PressedKeys},
};
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceState},
};

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, usb::UsbBusType>,
        usb_class: keyberon::Class<'static, usb::UsbBusType, ()>,
        matrix: Matrix<layout::Cols, layout::Rows>,
        layout: Layout,
        debouncer: Debouncer<PressedKeys<U4, U6>>,
        transform: fn(Event) -> Event,
        boot_btn: (
            hal::gpio::gpiob::PB8<hal::gpio::Input<hal::gpio::Floating>>,
            bool,
        ),
        timer: hal::timers::Timer<hal::pac::TIM3>,
        tx: hal::serial::Tx<hal::pac::USART1>,
        rx: hal::serial::Rx<hal::pac::USART1>,
        is_main_half: bool,
    }

    #[init]
    fn init(mut c: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<usb::UsbBusType>> = None;

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

        *USB_BUS = Some(usb::UsbBusType::new(usb));
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

        let (tx, rx) = {
            // Set up TX (PA9), RX (PA10)
            let (pa9, pa10) = (gpioa.pa9, gpioa.pa10);
            let pins = cortex_m::interrupt::free(|cs| {
                (pa9.into_alternate_af1(cs), pa10.into_alternate_af1(cs))
            });

            let mut serial =
                hal::serial::Serial::usart1(c.device.USART1, pins, 38_400.bps(), &mut rcc);
            serial.listen(hal::serial::Event::Rxne);
            serial.split()
        };

        let (pa0, pa1, pa2, pa3, pa4, pa5, pb4, pb5, pb6, pb7) = (
            gpioa.pa0, gpioa.pa1, gpioa.pa2, gpioa.pa3, gpioa.pa4, gpioa.pa5, gpiob.pb4, gpiob.pb5,
            gpiob.pb6, gpiob.pb7,
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
                    pb4.into_push_pull_output(cs),
                    pb5.into_push_pull_output(cs),
                    pb6.into_push_pull_output(cs),
                    pb7.into_push_pull_output(cs),
                ),
            )
        })
        .unwrap();

        init::LateResources {
            usb_dev,
            usb_class,
            debouncer: Debouncer::new(Default::default(), Default::default(), 5),
            matrix,
            layout: Layout::new(layout::LAYERS),
            boot_btn: (gpiob.pb8, false),
            transform,
            timer,
            tx,
            rx,
            is_main_half: false,
        }
    }

    #[task(binds = USART1, priority = 5, resources = [rx, layout])]
    fn rx(c: rx::Context) {
        static mut BUF: [u8; 3] = [0; 3];

        if let Ok(b) = c.resources.rx.read() {
            BUF.rotate_left(1);
            BUF[2] = b;

            if b == 0xff {
                if let Ok(event) = de(&BUF[..]) {
                    c.resources.layout.event(event);
                }
            }
        }
    }

    #[task(binds = USB, priority = 4, resources = [usb_dev, usb_class, is_main_half])]
    fn usb_rx(c: usb_rx::Context) {
        if c.resources.usb_dev.poll(&mut [c.resources.usb_class]) {
            use usb_device::class::UsbClass as _;
            c.resources.usb_class.poll();
        }
        if !*c.resources.is_main_half && c.resources.usb_dev.state() == UsbDeviceState::Configured {
            *c.resources.is_main_half = true;
        }
    }

    #[task(binds = TIM3, priority = 3, resources = [matrix, debouncer, timer, &transform, tx, is_main_half, layout, usb_class, boot_btn])]
    fn tick(mut c: tick::Context) {
        // Clear the interrupt flag
        c.resources.timer.wait().ok();

        let is_main: bool = c.resources.is_main_half.lock(|c| *c);

        for event in c
            .resources
            .debouncer
            .events(c.resources.matrix.get().unwrap())
            .map(c.resources.transform)
        {
            // Send events to the main half through USART
            if !is_main {
                for &b in &ser(event) {
                    nb::block!(c.resources.tx.write(b)).unwrap();
                }
            }
            c.resources.layout.lock(|c| c.event(event));
        }

        // Handle the BOOT button
        {
            let now = c.resources.boot_btn.0.is_high().unwrap();
            let prev = c.resources.boot_btn.1;
            let event: Option<Event> = match (now, prev) {
                (true, false) => Some((c.resources.transform)(Event::Press(3, 0))),
                (false, true) => Some((c.resources.transform)(Event::Release(3, 0))),
                _ => None,
            };
            if let Some(e) = event {
                c.resources.layout.lock(|l| l.event(e));
                c.resources.boot_btn.1 = now;
            }
        }

        c.resources.layout.lock(|c| c.tick());

        // Send the USB report
        if is_main {
            let report: KbHidReport = c.resources.layout.lock(|c| c.keycodes().collect());
            if c.resources
                .usb_class
                .lock(|c| c.device_mut().set_keyboard_report(report.clone()))
            {
                while let Ok(0) = c.resources.usb_class.lock(|c| c.write(report.as_bytes())) {}
            }
        }
    }

    extern "C" {
        fn CEC_CAN();
    }
};

// If the most significant bit of i is set, it's a press event

fn ser(e: Event) -> [u8; 3] {
    match e {
        Event::Press(i, j) => [i | 0x80, j, 0xff],
        Event::Release(i, j) => [i, j, 0xff],
    }
}

fn de(bytes: &[u8]) -> Result<Event, ()> {
    match *bytes {
        [i, j, 0xff] => {
            if (i & 0x80) != 0 {
                Ok(Event::Press(i & 0x7f, j))
            } else {
                Ok(Event::Release(i & 0x7f, j))
            }
        }
        _ => Err(()),
    }
}
