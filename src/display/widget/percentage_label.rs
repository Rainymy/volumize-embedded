use alloc::{format, string::String};
use embedded_graphics::{
    draw_target::DrawTarget, mono_font::MonoFont, pixelcolor::BinaryColor, primitives::Rectangle,
};

use crate::display::{
    Percentage,
    style::{Align, Style},
};

pub struct PercentageLabel<'a> {
    pub font: &'a MonoFont<'a>,
    pub style: Style<BinaryColor>,
}

impl<'a> PercentageLabel<'a> {
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
        percentage: &Percentage,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let area = self.style.paint(display, area)?;
        let text: String = format!("{:.1}%", percentage.to_percentage());
        self.style.draw_text(display, area, &text, self.font)?;
        Ok(())
    }
}
