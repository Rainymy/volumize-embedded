use super::super::{InputEvent, RotationEvent};
use super::Transition;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MainMenuState {
    pub selected: usize,
}

pub fn handle_main_menu(state: &mut MainMenuState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.selected = state.selected.saturating_add(1);
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            state.selected = state.selected.saturating_sub(1);
            Transition::Stay
        }
        InputEvent::SingleClick => match state.selected {
            _ => Transition::Stay,
        },
        _ => Transition::Ignored,
    }
}
