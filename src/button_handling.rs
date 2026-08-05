#![allow(unused)]

type Timestamp = u64;

const DEBOUNCE_MS: Timestamp = 75;
const LONG_PRESS_MS: Timestamp = 300;

#[derive(PartialEq, Clone, Copy)]
pub enum ButtonEvent {
    None,
    ShortPress,
    LongPress,
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
    pub fn poll(&mut self, raw_pressed: bool, now: Timestamp) -> (bool, ButtonEvent) {
        let mut changed = false;
        let mut event = ButtonEvent::None;

        if raw_pressed != self.pressed && now.wrapping_sub(self.last_change_ms) > DEBOUNCE_MS {
            self.pressed = raw_pressed;
            self.last_change_ms = now;
            changed = true;

            if self.pressed {
                self.press_start_ms = now;
                self.long_fired = false;
            } else if !self.long_fired {
                // released before the long-press threshold -> short press
                event = ButtonEvent::ShortPress;
            }
        }

        if self.pressed && !self.long_fired && now.wrapping_sub(self.press_start_ms) > LONG_PRESS_MS
        {
            self.long_fired = true;
            event = ButtonEvent::LongPress;
        }

        (changed, event)
    }
}
