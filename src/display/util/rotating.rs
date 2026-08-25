#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WrappingInt {
    pub value: i32,
    pub max_value: i32,
}

impl WrappingInt {
    pub fn new(value: i32, max_value: i32) -> Self {
        Self { value, max_value }
    }

    pub fn value(&self) -> i32 {
        self.value
    }

    pub fn next(&mut self) {
        if self.max_value != 0 {
            self.value = (self.value + 1).rem_euclid(self.max_value);
        }
    }

    pub fn prev(&mut self) {
        if self.max_value != 0 {
            self.value = (self.value - 1).rem_euclid(self.max_value);
        }
    }
}
