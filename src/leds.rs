#![allow(dead_code)]

use crate::hal::{gpio, pac::TIM2, timers::Timer};
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_timer_delay::Ws2812;

pub enum LedCommand {
    Solid(RGB8),
    Breathing(RGB8),
    Off,
}

pub struct Leds {
    ws: Ws2812<Timer<TIM2>, gpio::gpioa::PA15<gpio::Output<gpio::PushPull>>>,
    counter: u16,
    step_ms: u16,
    on: bool,
}

impl Leds {
    pub fn new(timer: Timer<TIM2>, pin: gpio::gpioa::PA15<gpio::Output<gpio::PushPull>>) -> Self {
        let ws = Ws2812::new(timer, pin);
        Leds {
            ws,
            counter: 0,
            step_ms: 0,
            on: true,
        }
    }
    pub fn turn_off(&mut self) {
        self.off();
        self.on = false
    }
    pub fn turn_on(&mut self) {
        self.on = true
    }
    pub fn step(&mut self) {
        if self.on {
            self.step_rainbow()
        }
    }
    pub fn step_rainbow(&mut self) {
        self.step_ms += 1;
        if self.step_ms != 10 {
            return;
        }
        self.step_ms = 0;
        let counter = self.counter;
        let i = (0..6).map(|i| wheel((((i * 256) as u16 / 6 + counter) & 255) as u8));
        self.ws.write(i).unwrap();
        self.counter += 1;
        if self.counter > 256 * 5 {
            self.counter = 0;
        }
    }
    pub fn step_blink(&mut self) {
        self.step_ms += 1;
        if self.step_ms != 500 {
            return;
        }
        self.step_ms = 0;
        if self.counter & 1 != 0 {
            self.ws
                .write(
                    core::iter::repeat(RGB8 {
                        r: 0x10,
                        g: 0x10,
                        b: 0x10,
                    })
                    .take(6),
                )
                .unwrap();
        } else {
            self.ws
                .write(core::iter::repeat(RGB8 { r: 0, g: 0, b: 0 }).take(6))
                .unwrap();
        }
        self.counter += 1;
    }
    pub fn solid(&mut self, r: u8, g: u8, b: u8) {
        self.on = false;
        self.ws
            .write(core::iter::repeat(RGB8 { r, g, b }).take(6))
            .unwrap();
    }
    pub fn off(&mut self) {
        self.ws
            .write(core::iter::repeat(RGB8 { r: 0, g: 0, b: 0 }).take(6))
            .unwrap();
    }
}

/// Input a value 0 to 255 to get a color value
/// The colours are a transition r - g - b - back to r.
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
