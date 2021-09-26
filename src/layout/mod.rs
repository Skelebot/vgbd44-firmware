use keyberon::action::{Action, HoldTapConfig};
use keyberon::key_code::KeyCode;

pub mod dvorak;
pub mod qwerty;

const S_ENTER: Action<!> = Action::HoldTap {
    timeout: 280,
    hold: &Action::KeyCode(KeyCode::RShift),
    tap: &Action::KeyCode(KeyCode::Enter),
    config: HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
};
