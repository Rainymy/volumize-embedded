use super::InputEvent;

#[derive(Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum RotationEvent {
    Next,
    Previous,
}

#[derive(Default)]
pub struct RotaryTracker {
    last_value: i16,
}

impl RotaryTracker {
    pub fn poll(&mut self, value: i16) -> Option<InputEvent> {
        let delta = value.wrapping_sub(self.last_value);
        self.last_value = value;

        if delta > 0 {
            Some(InputEvent::Rotation(RotationEvent::Next))
        } else if delta < 0 {
            Some(InputEvent::Rotation(RotationEvent::Previous))
        } else {
            None
        }
    }
}
