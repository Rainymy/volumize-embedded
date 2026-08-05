mod ssd1306_impl;

pub use ssd1306_impl::*;

pub trait RenderDisplay {
    fn render(&mut self, value: u16, button_pressed: bool) -> Result<(), u16>;
}

#[allow(unused)]
pub fn render<D>(display: &mut D, value: u16, button_pressed: bool) -> Result<(), u16>
where
    D: RenderDisplay,
{
    display.render(value, button_pressed)
}
