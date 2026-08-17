mod percentage;
mod ssd1306_impl;
mod style;

pub use percentage::*;
pub use style::*;

// pub use ssd1306_impl::*;

use super::button_handling::ButtonEvent;

pub trait RenderDisplay {
    fn render(&mut self, value: f32, button_state: ButtonEvent) -> Result<(), u16>;
}

#[allow(unused)]
pub fn render<D>(display: &mut D, value: f32, button_state: ButtonEvent) -> Result<(), u16>
where
    D: RenderDisplay,
{
    display.render(value, button_state)
}
