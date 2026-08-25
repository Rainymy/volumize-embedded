use super::Screen;
use alloc::vec::Vec;

pub struct UIState {
    stack: Vec<Screen>,
}

impl UIState {
    pub fn new(root: Screen) -> Self {
        Self {
            stack: Vec::from([root]),
        }
    }

    pub fn push(&mut self, screen: Screen) {
        self.stack.push(screen);
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

    pub fn current_mut(&mut self) -> &mut Screen {
        debug_assert!(self.stack.len() != 0, "Navigation stack is empty");
        // It is safe to unwrap. The stack is never empty.
        self.stack.last_mut().unwrap()
    }
}
