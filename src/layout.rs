use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use generic_array::typenum::{U4, U6};
use keyberon::impl_heterogenous_array;
use stm32f0xx_hal::gpio::{gpioa, gpiob, Input, Output, PullUp, PushPull};

pub struct Cols(
    pub gpioa::PA0<Input<PullUp>>,
    pub gpioa::PA1<Input<PullUp>>,
    pub gpioa::PA2<Input<PullUp>>,
    pub gpioa::PA3<Input<PullUp>>,
    pub gpioa::PA4<Input<PullUp>>,
    pub gpioa::PA5<Input<PullUp>>,
);

impl_heterogenous_array! {
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

impl_heterogenous_array! {
    Rows,
    dyn OutputPin<Error = Infallible>,
    U4,
    [0, 1, 2, 3]
}

use keyberon::action::{k, Action::*};
use keyberon::key_code::KeyCode::*;
#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers = &[
    &[
        &[k(Tab), k(Q), k(W), k(E), k(R), k(T), k(Y), k(U), k(I), k(O), k(P), k(Delete)],
        &[k(Tab), k(A), k(S), k(D), k(F), k(G), k(H), k(J), k(K), k(L), k(SColon), k(Delete)],
        &[k(Tab), k(Z), k(X), k(C), k(V), k(B), k(N), k(M), k(Comma), k(Dot), k(Slash), k(Bslash)],
        &[Trans, Trans, k(LCtrl), k(LGui), k(Space), k(LShift), k(RShift), k(Space), k(RGui), k(RCtrl), Trans, Trans],
    ],
];
