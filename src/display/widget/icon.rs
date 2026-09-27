use embedded_graphics::{draw_target::DrawTarget, pixelcolor::PixelColor, primitives::Rectangle};

use crate::display::{
    style::{Align, Bitmap, Style},
    theme::{FromTheme, Theme, ThemeColor},
};

pub struct IconWidget<'a, Color: PixelColor + FromTheme> {
    pub bitmap: Bitmap<'a>,
    pub color: Color,
    pub style: Style<Color>,
}

impl<'a, Color: PixelColor + FromTheme> IconWidget<'a, Color> {
    pub fn new(bitmap: Bitmap<'a>, theme: &Theme, color: Color) -> Self {
        Self {
            bitmap,
            color,
            style: Style::new()
                .background_theme(theme, ThemeColor::Background)
                .color_theme(theme, ThemeColor::Foreground)
                // .border(1, BinaryColor::On)
                .align(Align::Center),
        }
    }

    pub fn render<D>(&self, display: &mut D, area: Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Color>,
        D::Color: FromTheme,
    {
        let area = self.style.paint(display, area)?;
        self.style
            .draw_bitmap(display, area, &self.bitmap, self.color, None)?;
        Ok(())
    }
}
