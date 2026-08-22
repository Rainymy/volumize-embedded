type Timestamp = u64;

const LONG_PRESS_MS: Timestamp = 600;
const DOUBLE_CLICK_GAP_MS: Timestamp = 300;

#[derive(Clone, Copy, PartialEq, Eq, defmt::Format)]
enum ButtonState {
    Idle,
    Pressed { since: Timestamp },
    WaitingForSecondPress { released_at: Timestamp },
    PressedSecond { since: Timestamp },
}

#[derive(Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum ButtonEvent {
    SingleClick,
    DoubleClick,
    LongPress,
}

pub struct ButtonTracker {
    state: ButtonState,
}

impl ButtonTracker {
    pub fn new() -> Self {
        Self {
            state: ButtonState::Idle,
        }
    }

    pub fn on_edge(&mut self, is_down: bool, now: u64) -> Option<ButtonEvent> {
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
                    return Some(ButtonEvent::DoubleClick);
                }
                None
            }
        }
    }

    /// Call this every loop iteration (regardless of new edges) with the
    /// current time, to resolve timeouts that don't depend on a new press.
    pub fn check_timeouts(&mut self, now: u64) -> Option<ButtonEvent> {
        match self.state {
            ButtonState::Pressed { since } if now.saturating_sub(since) >= LONG_PRESS_MS => {
                self.state = ButtonState::Idle;
                Some(ButtonEvent::LongPress)
            }
            ButtonState::WaitingForSecondPress { released_at }
                if now.saturating_sub(released_at) > DOUBLE_CLICK_GAP_MS =>
            {
                self.state = ButtonState::Idle;
                Some(ButtonEvent::SingleClick)
            }
            _ => None,
        }
    }
}
