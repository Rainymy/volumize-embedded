use embedded_graphics::{
    draw_target::DrawTarget, mono_font::MonoFont, pixelcolor::BinaryColor, primitives::Rectangle,
};

use crate::display::style::{Align, Style};

pub struct MuteIndicator<'a> {
    pub font: &'a MonoFont<'a>,
    pub style: Style<BinaryColor>,
}

impl<'a> MuteIndicator<'a> {
    pub fn new(font: &'a MonoFont<'a>) -> Self {
        Self {
            font,
            style: Style::new(BinaryColor::On)
                .background(BinaryColor::Off)
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
        D: DrawTarget<Color = BinaryColor>,
    {
        let area = self.style.paint(display, area)?;
        let text = if is_muted { "M" } else { "U" };
        self.style.draw_text(display, area, text, self.font)?;
        Ok(())
    }
}
