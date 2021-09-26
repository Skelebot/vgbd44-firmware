#![allow(dead_code)]
use super::*;

pub static LAYERS: keyberon::layout::Layers<!, 12, 4, 4> = keyberon::layout::layout! {
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
