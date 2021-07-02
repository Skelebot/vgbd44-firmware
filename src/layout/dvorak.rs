use super::*;
use keyberon::action::Action;

static S_ENTER: Action = Action::HoldTap {
    timeout: 280,
    hold: &Action::KeyCode(RShift),
    tap: &Action::KeyCode(Enter),
    config: HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
};

pub static LAYERS: keyberon::layout::Layers = keyberon::layout::layout! {
    // Dvorak
    {
        [ Tab '\'' , . P Y    F G C R L BSpace ]
        [ LCtrl A O E U I    D H T N S - ]
        [ LShift ; Q J K X   B M W V Z / ]
        [ n n LGui (1) Space Escape   BSpace {S_ENTER} (2) RAlt n n ]
    }
    {
        [ Tab    1 2 3 4 5   6 7 8 9 0 BSpace ]
        [ LCtrl  n n n n n   Left Down Up Right n [LCtrl '`'] ]
        [ LShift n n n n n   n n n n n n ]
        [ n n LGui (2) t t   t t t RAlt n n ]
    }
    {
        [ Tab    ! @ # $ %   ^ & * '(' ')' BSpace ]
        [ LCtrl  n n n n n   - = '[' ']' '\\' '`' ]
        [ LShift n n n n n   '_' + '{' '}' | ~    ]
        [ n n LGui t t t   t t (1) RAlt n n ]
    }
    {
        [ Tab    F1 F2 F3 F4 F5   F6 F7 F8 F9 F10 BSpace ]
        [ LCtrl  n n n n n   n n n n n n ]
        [ LShift n n n n n   n n n n n n ]
        [ n n LGui t t t   t t (1) RAlt n n ]
    }
};

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
