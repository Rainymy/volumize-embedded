#![allow(unused)]

use core::fmt::Display;

type Timestamp = u64;

const DEBOUNCE_MS: Timestamp = 150;
const LONG_PRESS_MS: Timestamp = 600;

#[derive(PartialEq, Clone, Copy, Debug, defmt::Format)]
pub enum ButtonPressState {
    None,
    ShortPress,
    LongPress,
}

use super::navigation::{Button, InputEvent};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ButtonEvent {
    pub is_pressed: bool,
    pub event: ButtonPressState,
}

pub struct ButtonTracker {
    pressed: bool,
    last_change_ms: Timestamp,
    press_start_ms: Timestamp,
    long_fired: bool,
}

impl ButtonTracker {
    pub fn new() -> Self {
        Self {
            pressed: false,
            last_change_ms: 0,
            press_start_ms: 0,
            long_fired: false,
        }
    }

    /// Returns (state_changed, event). `raw_pressed` is true when the pin
    /// reads LOW (button pulled to ground when pressed).
    pub fn poll(&mut self, raw_pressed: bool, now_ms: Timestamp) -> Option<InputEvent> {
        let mut event = ButtonPressState::None;

        if raw_pressed != self.pressed && now_ms.wrapping_sub(self.last_change_ms) > DEBOUNCE_MS {
            self.pressed = raw_pressed;
            self.last_change_ms = now_ms;

            if self.pressed {
                self.press_start_ms = now_ms;
                self.long_fired = false;
            } else if !self.long_fired {
                // released before the long-press threshold -> short press
                event = ButtonPressState::ShortPress;
            }
        }

        if self.pressed
            && !self.long_fired
            && now_ms.wrapping_sub(self.press_start_ms) > LONG_PRESS_MS
        {
            self.long_fired = true;
            event = ButtonPressState::LongPress;
        }

        match event {
            ButtonPressState::None => None,
            ButtonPressState::ShortPress => Some(InputEvent::ShortPress(Button::Select)),
            ButtonPressState::LongPress => Some(InputEvent::LongPress(Button::DoubleClick)),
        }
    }
}
