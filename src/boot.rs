use hal::prelude::*;
use keyberon::layout::Event;

use crate::hal;

pub struct BootButton(pub hal::gpio::gpiob::PB8<hal::gpio::Input<hal::gpio::PullDown>>);

impl keyberon::debounced_matrix::StateTracker for BootButton {
    type State = bool;

    fn get_state(&self) -> Self::State {
        self.0.is_high().unwrap()
    }
    fn default_state(&self) -> Self::State {
        false
    }

    fn emit_event(&self, last: &Self::State, now: &Self::State) -> Option<Event> {
        match (last, now) {
            (false, true) => Some(Event::Press(3, 0)),
            (true, false) => Some(Event::Release(3, 0)),
            _ => None,
        }
    }
}
