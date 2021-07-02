use keyberon::action::{Action, HoldTapConfig};
use keyberon::key_code::KeyCode::*;

use super::*;

macro_rules! c {
    ($var:ident) => {
        Action::Custom(CustomActions::$var)
    };
}

static S_ENTER: Action<CustomActions> = Action::HoldTap {
    timeout: 280,
    hold: &Action::KeyCode(RShift),
    tap: &Action::KeyCode(Enter),
    config: HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
};

pub static LAYERS: &[&[&[Action<CustomActions>]]] = keyberon::layout::layout! {
    {
        [ Tab    Q W E R T   Y U I O P BSpace ]
        [ LCtrl  A S D F G   H J K L ; Quote  ]
        [ LShift Z X C V B   N M , . / Escape ]
        [ {Action::DefaultLayer(4)} n LGui (1) Space Escape   BSpace {S_ENTER} (2) RAlt n {Action::DefaultLayer(4)} ]
    }
    {
        [ Tab    1 2 3 4 5   6 7 8 9 0 BSpace ]
        [ LCtrl  n n n n n   Left Down Up Right n [LCtrl '`'] ]
        [ LShift n n n n n   n n n n n n ]
        [ n n t (2) t t   t t t t n n ]
    }
    {
        [ Tab    ! @ # $ %   ^ & * '(' ')' t ]
        [ LCtrl  n n n n n   - = '{' '}' '\\' '`' ]
        [ LShift n {c!(LedsSolid)} {c!(LedsWhite)} {c!(LedsOn)} {c!(LedsOff)}   '_' + '[' ']' | ~    ]
        //[ LShift n n n n n   '_' + '[' ']' | ~    ]
        [ n n t t t t   t t (1) t n n ]
    }
    {
        [ F12 F1 F2 F3 F4 F5   F6 F7 F8 F9 F10 F11 ]
        [ LCtrl  n n n n n   n n n n n n ]
        [ LShift n n n n n   n n n n n n ]
        [ n n t n t t   t t n t n n ]
    }
    // Gamer mode
    {
        [ Tab     T Q W E R   I Y Up U O P ]
        [ LShift   G A S D F   B Left Down Right ; Quote  ]
        [ LCtrl LShift Z X C V    H J K L N Escape ]
        [{Action::DefaultLayer(0)} n LGui (1) Space Escape   M {S_ENTER} (2) RAlt n {Action::DefaultLayer(0)} ]
    }
};
