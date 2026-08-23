use super::InputEvent;

type Timestamp = u64;

const LONG_PRESS_MS: Timestamp = 600;
const DOUBLE_CLICK_GAP_MS: Timestamp = 300;

#[derive(Clone, Copy, PartialEq, Eq, Default, defmt::Format)]
enum ButtonState {
    #[default]
    Idle,
    Pressed {
        since: Timestamp,
    },
    WaitingForSecondPress {
        released_at: Timestamp,
    },
    PressedSecond {
        since: Timestamp,
    },
}

#[derive(Default)]
pub struct ButtonTracker {
    state: ButtonState,
}

impl ButtonTracker {
    pub fn on_edge(&mut self, is_down: bool, now: u64) -> Option<InputEvent> {
        match self.state {
            ButtonState::Idle => {
                if is_down {
                    self.state = ButtonState::Pressed { since: now };
                }
                None
            }
            ButtonState::Pressed { since: _ } => {
                if !is_down {
                    // released — could be single or double click, wait and see
                    self.state = ButtonState::WaitingForSecondPress { released_at: now };
                }
                None
            }
            ButtonState::WaitingForSecondPress { .. } => {
                if is_down {
                    self.state = ButtonState::PressedSecond { since: now };
                }
                None
            }
            ButtonState::PressedSecond { .. } => {
                if !is_down {
                    self.state = ButtonState::Idle;
                    return Some(InputEvent::DoubleClick);
                }
                None
            }
        }
    }

    /// Call this every loop iteration (regardless of new edges) with the
    /// current time, to resolve timeouts that don't depend on a new press.
    pub fn check_timeouts(&mut self, now: u64) -> Option<InputEvent> {
        match self.state {
            ButtonState::Pressed { since } if now.saturating_sub(since) >= LONG_PRESS_MS => {
                self.state = ButtonState::Idle;
                Some(InputEvent::LongPress)
            }
            ButtonState::WaitingForSecondPress { released_at }
                if now.saturating_sub(released_at) > DOUBLE_CLICK_GAP_MS =>
            {
                self.state = ButtonState::Idle;
                Some(InputEvent::SingleClick)
            }
            _ => None,
        }
    }
}
