//pub mod dvorak;
pub mod qwerty;

use keyberon::action::HoldTapConfig;
use keyberon::key_code::KeyCode::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CustomActions {
    #[cfg(feature = "leds")]
    LedsOff,
    #[cfg(feature = "leds")]
    LedsOn,
    #[cfg(feature = "leds")]
    LedsWhite,
    #[cfg(feature = "leds")]
    LedsSolid,
    Dummy,
}

use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::typenum::{U4, U6};
use stm32f0xx_hal::gpio::{gpioa, gpiob, Input, Output, PullUp, PushPull};

pub struct Cols(
    pub gpioa::PA0<Input<PullUp>>,
    pub gpioa::PA1<Input<PullUp>>,
    pub gpioa::PA2<Input<PullUp>>,
    pub gpioa::PA3<Input<PullUp>>,
    pub gpioa::PA4<Input<PullUp>>,
    pub gpioa::PA5<Input<PullUp>>,
);

keyberon::impl_heterogenous_array! {
    Cols,
    dyn InputPin<Error = Infallible>,
    U6,
    [0, 1, 2, 3, 4, 5]
}

pub struct Rows(
    pub gpiob::PB4<Output<PushPull>>,
    pub gpiob::PB5<Output<PushPull>>,
    pub gpiob::PB6<Output<PushPull>>,
    pub gpiob::PB7<Output<PushPull>>,
);

keyberon::impl_heterogenous_array! {
    Rows,
    dyn OutputPin<Error = Infallible>,
    U4,
    [0, 1, 2, 3]
}
