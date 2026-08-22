use alloc::vec::Vec;

use super::button_handling::ButtonEvent;

#[derive(Clone, Copy, Debug, defmt::Format)]
pub enum Screen {
    MainMenu { selected: u8 },
    VolumeAdjust { value: i32 },
    Settings { selected: u8 },
}

pub struct UIState {
    stack: Vec<Screen>,
}

impl UIState {
    pub fn new() -> Self {
        let mut inner = Vec::new();
        inner.push(Screen::MainMenu { selected: 0 });

        Self { stack: inner }
    }

    pub fn push(&mut self, screen: Screen) {
        let _ = self.stack.push(screen);
    }

    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn current(&mut self) -> &Screen {
        debug_assert!(self.stack.len() != 0, "Navigation stack is empty");
        // It is safe to unwrap. The stack is never empty.
        self.stack.last().unwrap()
    }

    #[allow(dead_code)]
    pub fn current_mut(&mut self) -> &mut Screen {
        debug_assert!(self.stack.len() != 0, "Navigation stack is empty");
        // It is safe to unwrap. The stack is never empty.
        self.stack.last_mut().unwrap()
    }
}

#[allow(dead_code)]
#[derive(Debug, defmt::Format)]
pub enum Button {
    DoubleClick,
    Select,
    Back,
}

pub fn handle_event(ui_state: &mut UIState, event: ButtonEvent) {
    match (&*ui_state.current(), event) {
        (Screen::MainMenu { selected }, ButtonEvent::LongPress) => {
            let _ = selected;
            ui_state.push(Screen::VolumeAdjust { value: 0 });
        }
        (Screen::VolumeAdjust { .. }, ButtonEvent::LongPress) => {
            ui_state.pop();
        }
        (Screen::VolumeAdjust { value }, ButtonEvent::SingleClick) => {
            let new_value = *value + 1;
            ui_state.push(Screen::VolumeAdjust { value: new_value });
        }
        (Screen::MainMenu { .. }, ButtonEvent::DoubleClick) => {
            ui_state.push(Screen::Settings { selected: 0 });
        }
        _ => {} // event doesn't apply to current screen — ignored
    }
}
