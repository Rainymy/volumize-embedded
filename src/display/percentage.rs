#![allow(dead_code)]

pub struct Percentage {
    value: f32,
}

impl Percentage {
    const MAX_VALUE: f32 = 1.0;
    const MIN_VALUE: f32 = 0.0;

    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    pub fn from_float(value: f32) -> Self {
        Self {
            value: Self::clamp(value),
        }
    }

    fn clamp(value: f32) -> f32 {
        value.max(Self::MIN_VALUE).min(Self::MAX_VALUE)
    }

    pub fn is_max(&self) -> bool {
        self.value >= Self::MAX_VALUE
    }

    /// Value is clamped to [MIN_VALUE] and [MAX_VALUE]
    pub fn increment_percentage(&mut self, amount: f32) {
        let new_value = self.value + amount / 100.0;
        self.value = new_value.min(Self::MAX_VALUE).max(Self::MIN_VALUE)
    }

    /// Resets to [MIN_VALUE] if the value reaches the maximum after incrementing
    pub fn resetting_increment(&mut self, amount: f32) {
        let is_max = self.is_max();
        self.increment_percentage(amount);
        if is_max {
            self.value = Self::MIN_VALUE;
        }
    }

    pub fn to_float(&self) -> f32 {
        self.value
    }

    pub fn to_percentage(&self) -> f32 {
        self.value * 100.0
    }
}
