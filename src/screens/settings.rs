use super::super::InputEvent;
use super::Transition;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SettingsState {
    pub selected: usize,
}

pub fn handle_settings(_state: &mut SettingsState, event: InputEvent) -> Transition {
    match event {
        InputEvent::LongPress => Transition::Pop,
        _ => Transition::Ignored,
    }
}
