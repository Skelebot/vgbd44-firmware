use keyberon::action::{*, k, l, m, HoldTapConfig, Action::*};
use keyberon::key_code::KeyCode::*;

//#[rustfmt::skip]
//pub static LAYERS: keyberon::layout::Layers = &[
//    &[
//        &[k(Tab), k(Q), k(W), k(E), k(R), k(T), k(Y), k(U), k(I), k(O), k(P), k(Delete)],
//        &[k(Tab), k(A), k(S), k(D), k(F), k(G), k(H), k(J), k(K), k(L), k(SColon), k(Delete)],
//        &[k(Tab), k(Z), k(X), k(C), k(V), k(B), k(N), k(M), k(Comma), k(Dot), k(Slash), k(Bslash)],
//        &[k(B), Trans, k(LCtrl), k(LGui), k(Space), k(LShift), k(RShift), k(Space), k(RGui), k(RCtrl), Trans, k(R)],
//    ],
//];

const CUT: Action = m(&[LShift, Delete]);
const COPY: Action = m(&[LCtrl, Insert]);
const PASTE: Action = m(&[LShift, Insert]);
const L2_ENTER: Action =  Action::HoldTap {
    config: HoldTapConfig::Default,
    tap_hold_interval: 0,
    timeout: 140,
    hold: &l(2),
    tap: &k(Enter),
};
const L1_SP: Action = Action::HoldTap {
    config: HoldTapConfig::Default,
    tap_hold_interval: 0,
    timeout: 200,
    hold: &l(1),
    tap: &k(Space),
};
const CSPACE: Action = m(&[LCtrl, Space]);
macro_rules! s {
    ($k:ident) => {
        m(&[LShift, $k])
    };
}
macro_rules! a {
    ($k:ident) => {
        m(&[RAlt, $k])
    };
}

#[rustfmt::skip]
pub static LAYERS: keyberon::layout::Layers = &[
    &[
        &[k(Tab),   k(Q), k(W), k(E), k(R), k(T), k(Y), k(U), k(I),    k(O),   k(P),      k(Bslash)],
        &[k(RCtrl), k(A), k(S), k(D), k(F), k(G), k(H), k(J), k(K),    k(L),   k(SColon), k(Quote)   ],
        &[k(Escape),k(Z), k(X), k(C), k(V), k(B), k(N), k(M), k(Comma),k(Dot), k(Slash),  k(Menu)  ],
        &[k(Escape), Trans, k(LGui),k(LAlt),L1_SP,k(LBracket), k(RBracket),L2_ENTER,k(RAlt),k(RGui),Trans,Trans],
    ], &[
        &[Trans,         k(Pause),Trans,     k(PScreen),Trans,    Trans,Trans,      Trans,  k(Delete),Trans,  Trans,   Trans ],
        &[Trans,         Trans,   k(NumLock),k(Insert), k(Escape),Trans,k(CapsLock),k(Left),k(Down),  k(Up),  k(Right),Trans ],
        &[k(NonUsBslash),k(Undo), CUT,       COPY,      PASTE,    Trans,Trans,      k(Home),k(PgDown),k(PgUp),k(End),  Trans ],
        &[Trans,         Trans,   Trans,     Trans,     Trans,    Trans,Trans,      Trans,  Trans,    Trans,  Trans,   Trans ],
    ], &[
        &[s!(Grave),s!(Kb1),s!(Kb2),s!(Kb3),s!(Kb4),s!(Kb5),s!(Kb6),s!(Kb7),s!(Kb8),s!(Kb9),s!(Kb0),s!(Minus)],
        &[ k(Grave), k(Kb1), k(Kb2), k(Kb3), k(Kb4), k(Kb5), k(Kb6), k(Kb7), k(Kb8), k(Kb9), k(Kb0), k(Minus)],
        &[a!(Grave),a!(Kb1),a!(Kb2),a!(Kb3),a!(Kb4),a!(Kb5),a!(Kb6),a!(Kb7),a!(Kb8),a!(Kb9),a!(Kb0),a!(Minus)],
        &[Trans,    Trans,  Trans,  Trans,  CSPACE, Trans,  Trans,  Trans,  Trans,  Trans,  Trans,  Trans    ],
    ],
];


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
