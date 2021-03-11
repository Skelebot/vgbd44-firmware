#![no_main]
#![no_std]

mod layout;

use cortex_m_rt::entry;
use panic_halt as _;
//use rtic::app;

use hal::{prelude::*, usb::{self, UsbBusType}, pac::interrupt};
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

static mut USB_BUS: Option<UsbBusAllocator<usb::UsbBusType>> = None;
static mut USB_DEV: Option<UsbDevice<usb::UsbBusType>> = None;
static mut USB_CLASS: Option<keyberon::Class<'static, usb::UsbBusType, ()>> = None;

use hal::gpio::{
    Alternate, AF1,
    gpioa::{PA9, PA10},
};
static mut SERIAL: Option<hal::serial::Serial<hal::pac::USART1, PA9<Alternate<AF1>>, PA10<Alternate<AF1>>>> = None;

struct KeyRes {
    matrix: Matrix<layout::Cols, layout::Rows>,
    layout: Layout,
    debouncer: Debouncer<PressedKeys<U4, U6>>,
    boot_btn: (
        hal::gpio::gpiob::PB8<hal::gpio::Input<hal::gpio::Floating>>,
        bool,
    ),
}

static mut KEY_RES: Option<KeyRes> = None;

static mut IS_MAIN: bool = false;

static mut TIMER: Option<hal::timers::Timer<hal::pac::TIM3>> = None;


#[entry]
fn main() -> ! {
    let mut c = hal::pac::Peripherals::take().unwrap();

    stm32f0xx_hal::usb::remap_pins(&mut c.RCC, &mut c.SYSCFG);

    let mut rcc = c
        .RCC
        .configure()
        .hsi48()
        .enable_crs(c.CRS)
        .sysclk(48.mhz())
        .pclk(24.mhz())
        .freeze(&mut c.FLASH);

    let gpioa = c.GPIOA.split(&mut rcc);
    let gpiob = c.GPIOB.split(&mut rcc);

    let usb = hal::usb::Peripheral {
        usb: c.USB,
        pin_dm: gpioa.pa11,
        pin_dp: gpioa.pa12,
    };

    let usb_bus = unsafe {
        USB_BUS = Some(usb::UsbBusType::new(usb));
        USB_BUS.as_ref().unwrap()
    };

    let usb_class = keyberon::new_class(usb_bus, ());
    let usb_dev = keyberon::new_device(usb_bus);

    let mut timer = hal::timers::Timer::tim3(c.TIM3, 1.khz(), &mut rcc);
    timer.listen(hal::timers::Event::TimeOut);

    let serial = {
        // Set up TX (PA9), RX (PA10)
        let (pa9, pa10) = (gpioa.pa9, gpioa.pa10);
        let pins = cortex_m::interrupt::free(|cs| {
            (pa9.into_alternate_af1(cs), pa10.into_alternate_af1(cs))
        });

        let mut serial =
            hal::serial::Serial::usart1(c.USART1, pins, 38_400.bps(), &mut rcc);
        serial.listen(hal::serial::Event::Rxne);
        serial
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
    
    unsafe {
        USB_DEV = Some(usb_dev);
        USB_CLASS = Some(usb_class);
        SERIAL = Some(serial);
        KEY_RES = Some(KeyRes {
            matrix,
            layout: Layout::new(layout::LAYERS),
            debouncer: Debouncer::new(Default::default(), Default::default(), 5),
            boot_btn: (gpiob.pb8, false),
        });
        IS_MAIN = false;
        TIMER = Some(timer);
    }
    
    loop {}
}

#[interrupt]
unsafe fn USART1() {
    static mut BUF: [u8; 3] = [0; 3];

    if let Ok(b) = SERIAL.as_mut().unwrap().read() {
        BUF.rotate_left(1);
        BUF[2] = b;

        if b == 0xff {
            if let Ok(event) = de(&BUF[..]) {
                KEY_RES.as_mut().unwrap().layout.event(event);
            }
        }
    }
}

#[interrupt]
unsafe fn USB() {
    if USB_DEV.as_mut().unwrap().poll(&mut [USB_CLASS.as_mut().unwrap()]) {
        use usb_device::class::UsbClass as _;
        USB_CLASS.as_mut().unwrap().poll();
    }
    if !IS_MAIN && USB_DEV.as_ref().unwrap().state() == UsbDeviceState::Configured {
        IS_MAIN = true;
    }
}

#[interrupt]
unsafe fn TIM3() {
    // Clear the interrupt flag
    TIMER.as_mut().unwrap().wait().ok();
    
    let res = KEY_RES.as_mut().unwrap();

    for event in res
        .debouncer
        .events(res.matrix.get().unwrap())
        .map(|e| e)
    {
        // Send events to the main half through USART
        if !IS_MAIN {
            for &b in &ser(event) {
                nb::block!(SERIAL.as_mut().unwrap().write(b)).unwrap();
            }
        }
        res.layout.event(event);
    }

    // Handle the BOOT button
    {
        let now = res.boot_btn.0.is_high().unwrap();
        let prev = res.boot_btn.1;
        let event: Option<Event> = match (now, prev) {
            (true, false) => Some(Event::Press(3, 0)),
            (false, true) => Some(Event::Release(3, 0)),
            _ => None,
        };
        if let Some(e) = event {
            res.layout.event(e);
            res.boot_btn.1 = now;
        }
    }

    res.layout.tick();

    // Send the USB report
    if IS_MAIN {
        let report: KbHidReport = res.layout.keycodes().collect();
        if 
            USB_CLASS.as_mut().unwrap()
            .device_mut().set_keyboard_report(report.clone())
        {
            while let Ok(0) = USB_CLASS.as_mut().unwrap().write(report.as_bytes()) {}
        }
    }
}

extern "C" {
    fn CEC_CAN();
}

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
