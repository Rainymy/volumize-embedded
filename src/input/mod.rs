mod button;
mod rotary;

pub use button::ButtonTracker;
pub use rotary::{RotaryTracker, RotationEvent};

#[derive(Clone, Copy, PartialEq, Eq, defmt::Format)]
pub enum InputEvent {
    SingleClick,
    DoubleClick,
    LongPress,
    Rotation(RotationEvent),
}
