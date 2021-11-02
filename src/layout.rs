use keyberon::action::{Action, HoldTapConfig};
use keyberon::key_code::KeyCode;

// Enter key, shift when held for more than 280ms
// or when another key is pressed while this one is held
const S_ENTER: Action<!> = Action::HoldTap {
    timeout: 280,
    hold: &Action::KeyCode(KeyCode::RShift),
    tap: &Action::KeyCode(KeyCode::Enter),
    config: HoldTapConfig::PermissiveHold,
    tap_hold_interval: 0,
};

// Switch to i3 workspace number $n
macro_rules! w {
    ( $n:ident ) => {
        Action::MultipleKeyCodes(&[KeyCode::LGui, KeyCode::$n])
    };
}

// Switch to layout layer number $n
macro_rules! l {
    ( $n:expr ) => {
        Action::DefaultLayer($n)
    };
}

// Number of layers in the layout; modify accordingly
pub const NUM_LAYERS: usize = 5;

pub const LAYERS: keyberon::layout::Layers<!, 12, 4, NUM_LAYERS> = keyberon::layout::layout! {
    {
        [ Tab    Q W E R T   Y U I O P BSpace ]
        [ LCtrl  A S D F G   H J K L ; Quote  ]
        [ LShift Z X C V B   N M , . / Escape ]
        //[ {l!(4)} n LGui (1) Space Escape   BSpace {S_ENTER} (2) RAlt n {l!(4)} ]
        [ N n LGui (1) Space Escape   BSpace {S_ENTER} (2) RAlt n G ]
    }
    {
        [ t 1 2 3 4 5   6 7 8 9 0 t ]
        [ t {w!(Kb1), w!(Kb2), w!(Kb3), w!(Kb4), w!(Kb5)}  Left Down Up Right n [LCtrl '`'] ]
        [ t n Delete n n n   MediaPreviousSong MediaVolDown MediaVolUp MediaNextSong MediaPlayPause n ]
        [ n n t (2) t t   t t t t n n ]
    }
    {
        [ t ! @ # $ %   ^ & * '(' ')' t ]
        [ t n n n n n   - = '{' '}' '\\' '`' ]
        [ t n n n n n   '_' + '[' ']' | ~    ]
        [ n n t t t t   t t (1) t n n ]
    }
    {
        [ F12 F1 F2 F3 F4 F5   F6 F7 F8 F9 F10 F11 ]
        [ t n n n n n   n n n n n n ]
        [ t n n n n n   n n n n n n ]
        [ n n t n t t   t t n t n n ]
    }
    // Gamer mode
    {
        [ Tab        T Q W E R   I Y Up U O P ]
        [ LShift     G A S D F   B Left Down Right ; Quote ]
        [ LCtrl LShift Z X C V   H J K L N Escape ]
        [{l!(0)} n LGui (1) Space Escape   M {S_ENTER} (2) RAlt n {l!(0)} ]
    }
};
