#![feature(never_type)]
#![no_main]
#![no_std]

mod boot;
mod layout;
use boot::BootButton;

use hal::{
    gpio::{self, Input, Output, PullUp, PushPull},
    prelude::*,
};
use stm32f0xx_hal as hal;

use panic_halt as _;
use rtic::app;

use keyberon::{
    debounced_matrix::DebouncedMatrix,
    key_code::KbHidReport,
    layout::{Event, Layout},
};
use usb_device::{
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceState},
};

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        usb_dev: UsbDevice<'static, hal::usb::UsbBusType>,
        usb_class: keyberon::Class<'static, hal::usb::UsbBusType, ()>,
        matrix: DebouncedMatrix<
            gpio::Pin<Input<PullUp>>,
            gpio::Pin<Output<PushPull>>,
            BootButton,
            6,
            4,
            5,
        >,
        layout: Layout<!, 12, 4, { layout::NUM_LAYERS }>,
        is_right_half: bool,
        timer: hal::timers::Timer<hal::pac::TIM3>,
        tx: hal::serial::Tx<hal::pac::USART1>,
        rx: hal::serial::Rx<hal::pac::USART1>,
        is_main_half: bool,
    }

    #[init]
    fn init(mut c: init::Context) -> init::LateResources {
        static mut USB_BUS: Option<UsbBusAllocator<hal::usb::UsbBusType>> = None;

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

        *USB_BUS = Some(hal::usb::UsbBusType::new(usb));
        let usb_bus = USB_BUS.as_ref().unwrap();
        let usb_class = keyberon::new_class(usb_bus, ());
        let usb_dev = usb_device::device::UsbDeviceBuilder::new(
            usb_bus,
            usb_device::device::UsbVidPid(0x16c0, 0xcafe),
        )
        .manufacturer("HoldIT")
        .product("vgbd44 keyboard")
        .serial_number("Yes")
        .device_release(0x0010)
        .build();

        let mut timer = hal::timers::Timer::tim3(c.device.TIM3, 1.khz(), &mut rcc);
        timer.listen(hal::timers::Event::TimeOut);

        let is_right_half = gpioa.pa8.is_high().unwrap();

        let (tx, rx) = {
            let pins = cortex_m::interrupt::free(|cs| {
                (
                    gpioa.pa9.into_alternate_af1(cs),
                    gpioa.pa10.into_alternate_af1(cs),
                )
            });

            let mut serial =
                hal::serial::Serial::usart1(c.device.USART1, pins, 38_400.bps(), &mut rcc);
            serial.listen(hal::serial::Event::Rxne);
            serial.split()
        };

        let matrix = cortex_m::interrupt::free(move |cs| {
            DebouncedMatrix::new(
                [
                    gpioa.pa0.into_pull_up_input(cs).downgrade(),
                    gpioa.pa1.into_pull_up_input(cs).downgrade(),
                    gpioa.pa2.into_pull_up_input(cs).downgrade(),
                    gpioa.pa3.into_pull_up_input(cs).downgrade(),
                    gpioa.pa4.into_pull_up_input(cs).downgrade(),
                    gpioa.pa5.into_pull_up_input(cs).downgrade(),
                ],
                [
                    gpiob.pb4.into_push_pull_output(cs).downgrade(),
                    gpiob.pb5.into_push_pull_output(cs).downgrade(),
                    gpiob.pb6.into_push_pull_output(cs).downgrade(),
                    gpiob.pb7.into_push_pull_output(cs).downgrade(),
                ],
                BootButton(gpiob.pb8.into_pull_down_input(cs)),
            )
        })
        .unwrap();

        init::LateResources {
            usb_dev,
            usb_class,
            matrix,
            layout: Layout::new(&layout::LAYERS),
            is_right_half,
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
                c.resources.layout.event(de(BUF));
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

    #[task(binds = TIM3, priority = 3, resources = [matrix, timer, is_right_half, tx, is_main_half, layout, usb_class])]
    fn tick(mut c: tick::Context) {
        // Clear the interrupt flag
        c.resources.timer.wait().ok();

        c.resources.layout.lock(|l| l.tick());
        let is_main: bool = c.resources.is_main_half.lock(|c| *c);

        if let Some(events) = c.resources.matrix.scan().unwrap() {
            for mut event in events {
                if *c.resources.is_right_half {
                    event = event.transform(|r, c| (r, 11 - c))
                }
                if !is_main {
                    for &b in &ser(event) {
                        nb::block!(c.resources.tx.write(b)).unwrap();
                    }
                }
                c.resources.layout.lock(|c| c.event(event));
            }
        }

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
};

// If the most significant bit of i is set, it's a press event
fn ser(e: Event) -> [u8; 3] {
    match e {
        Event::Press(i, j) => [i | 0x80, j, 0xff],
        Event::Release(i, j) => [i, j, 0xff],
    }
}

fn de(bytes: &[u8; 3]) -> Event {
    if (bytes[0] & 0x80) != 0 {
        Event::Press(bytes[0] & 0x7f, bytes[1])
    } else {
        Event::Release(bytes[0] & 0x7f, bytes[1])
    }
}
