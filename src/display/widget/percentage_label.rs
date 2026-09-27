use alloc::{format, string::String};
use embedded_graphics::{
    draw_target::DrawTarget, mono_font::MonoFont, pixelcolor::PixelColor, primitives::Rectangle,
};

use crate::display::{
    Percentage,
    style::{Align, Style},
    theme::{FromTheme, Theme, ThemeColor},
};

pub struct PercentageLabel<'a, Color: PixelColor + FromTheme> {
    pub font: &'a MonoFont<'a>,
    pub style: Style<Color>,
}

impl<'a, Color: PixelColor + FromTheme> PercentageLabel<'a, Color> {
    pub fn new(font: &'a MonoFont<'a>, theme: &Theme, token: ThemeColor) -> Self {
        Self {
            font,
            style: Style::new().color_theme(theme, token).align(Align::Center),
        }
    }

    pub fn render<D>(
        &self,
        display: &mut D,
        area: Rectangle,
        percentage: &Percentage,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Color>,
        D::Color: FromTheme,
    {
        let area = self.style.paint(display, area)?;
        let text: String = format!("{:.1}%", percentage.to_percentage());
        self.style.draw_text(display, area, &text, self.font)?;
        Ok(())
    }
}
