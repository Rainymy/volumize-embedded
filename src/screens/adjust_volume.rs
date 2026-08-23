use super::super::{InputEvent, RotationEvent};
use super::Transition;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VolumeAdjustState {
    pub value: i32,
}

pub fn handle_volume_adjust(state: &mut VolumeAdjustState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.value = (state.value + 1).min(100);
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            state.value = state.value.saturating_sub(1);
            Transition::Stay
        }
        InputEvent::LongPress => Transition::Pop,
        _ => Transition::Ignored,
    }
}
