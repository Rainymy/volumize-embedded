use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor, primitives::Rectangle};

use crate::display::style::{Align, Bitmap, Style};

pub struct IconWidget<'a> {
    pub bitmap: Bitmap<'a>,
    pub color: BinaryColor,
    pub style: Style<BinaryColor>,
}

impl<'a> IconWidget<'a> {
    pub fn new(bitmap: Bitmap<'a>, color: BinaryColor) -> Self {
        Self {
            bitmap,
            color,
            style: Style::new(color.invert())
                .background(color)
                // .border(1, BinaryColor::On)
                .align(Align::Center),
        }
    }

    pub fn render<D>(&self, display: &mut D, area: Rectangle) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let area = self.style.paint(display, area)?;
        self.style
            .draw_bitmap(display, area, &self.bitmap, self.color.invert(), None)?;
        Ok(())
    }
}
