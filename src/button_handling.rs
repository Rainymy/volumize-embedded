#![allow(unused)]

use core::fmt::Display;

type Timestamp = u64;

const DEBOUNCE_MS: Timestamp = 150;
const LONG_PRESS_MS: Timestamp = 600;

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ButtonPressState {
    None,
    ShortPress,
    LongPress,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct ButtonEvent {
    pub is_pressed: bool,
    pub event: ButtonPressState,
}

impl defmt::Format for ButtonPressState {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            ButtonPressState::None => defmt::write!(fmt, "None"),
            ButtonPressState::ShortPress => defmt::write!(fmt, "ShortPress"),
            ButtonPressState::LongPress => defmt::write!(fmt, "LongPress"),
        }
    }
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
    pub fn poll(&mut self, raw_pressed: bool, now: Timestamp) -> ButtonEvent {
        let mut event = ButtonPressState::None;

        if raw_pressed != self.pressed && now.wrapping_sub(self.last_change_ms) > DEBOUNCE_MS {
            self.pressed = raw_pressed;
            self.last_change_ms = now;

            if self.pressed {
                self.press_start_ms = now;
                self.long_fired = false;
            } else if !self.long_fired {
                // released before the long-press threshold -> short press
                event = ButtonPressState::ShortPress;
            }
        }

        if self.pressed && !self.long_fired && now.wrapping_sub(self.press_start_ms) > LONG_PRESS_MS
        {
            self.long_fired = true;
            event = ButtonPressState::LongPress;
        }

        ButtonEvent {
            is_pressed: raw_pressed,
            event: event,
        }
    }
}
