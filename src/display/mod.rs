mod percentage;
mod rounded_rectangle;
mod ssd1306_impl;
mod style;
pub mod text_style;

pub use percentage::*;
pub use rounded_rectangle::*;
pub use style::*;

use crate::navigation::Screen;

// use super::button_handling::ButtonEvent;

pub trait RenderDisplay {
    async fn render(&mut self, value: f32, screen: Screen) -> Result<(), u16>;
}

pub async fn render<D>(display: &mut D, value: f32, screen: Screen) -> Result<(), u16>
where
    D: RenderDisplay,
{
    display.render(value, screen).await
}
