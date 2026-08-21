mod percentage;
mod ssd1306_impl;
mod style;
pub mod text_style;

pub use percentage::*;
pub use style::*;

use super::button_handling::ButtonEvent;

pub trait RenderDisplay {
    async fn render(&mut self, value: f32, button_state: ButtonEvent) -> Result<(), u16>;
}

pub async fn render<D>(display: &mut D, value: f32, button_state: ButtonEvent) -> Result<(), u16>
where
    D: RenderDisplay,
{
    display.render(value, button_state).await
}
