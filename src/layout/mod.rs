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