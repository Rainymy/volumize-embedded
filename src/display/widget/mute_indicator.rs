use embedded_graphics::{
    draw_target::DrawTarget, mono_font::MonoFont, pixelcolor::PixelColor, primitives::Rectangle,
};

use crate::display::{
    style::{Align, Style},
    theme::{FromTheme, Theme, ThemeColor},
};

pub struct MuteIndicator<'a, Color: PixelColor + FromTheme> {
    pub font: &'a MonoFont<'a>,
    pub style: Style<Color>,
}

impl<'a, Color: PixelColor + FromTheme> MuteIndicator<'a, Color> {
    pub fn new(
        font: &'a MonoFont<'a>,
        theme: &Theme,
        bg_color: ThemeColor,
        fg_color: ThemeColor,
    ) -> Self {
        Self {
            font,
            style: Style::new()
                .background_theme(theme, bg_color)
                .color_theme(theme, fg_color)
                .align(Align::Center),
        }
    }

    pub fn render<D>(
        &self,
        display: &mut D,
        area: Rectangle,
        is_muted: bool,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = Color>,
        D::Color: FromTheme,
    {
        let area = self.style.paint(display, area)?;
        let text = if is_muted { "M" } else { "U" };
        self.style.draw_text(display, area, text, self.font)?;
        Ok(())
    }
}
