use super::MainMenuState;
use super::Screen;

use alloc::vec::Vec;

pub struct UIState {
    stack: Vec<Screen>,
}

impl UIState {
    pub fn new() -> Self {
        let mut inner = Vec::new();
        inner.push(Screen::MainMenu(MainMenuState::default()));

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
