use embedded_graphics::{
    draw_target::DrawTarget, geometry::Size, pixelcolor::BinaryColor, primitives::Rectangle,
};

use crate::display::{
    Percentage,
    style::{Align, Corners, Style},
};

pub struct VerticalSlider {
    pub width: u32,
    pub outer_style: Style<BinaryColor>,
    pub inner_style_full: Style<BinaryColor>,
    pub inner_style_partial: Style<BinaryColor>,
}

impl Default for VerticalSlider {
    fn default() -> Self {
        let outer_style = Style::new(BinaryColor::On)
            .margin_all(1)
            .padding_all(1)
            .border(1, BinaryColor::On)
            .background(BinaryColor::Off)
            .radius_all(3)
            .align(Align::Center);

        let inner_style_full = Style::new(BinaryColor::On)
            .padding_all(1)
            .background(BinaryColor::On)
            .border(1, BinaryColor::On)
            .radius_all(3);

        let inner_style_partial = Style::new(BinaryColor::On)
            .padding_all(1)
            .background(BinaryColor::On)
            .border(1, BinaryColor::On)
            .radius(Corners::new(0, 0, 3, 3));

        Self {
            width: 10,
            outer_style,
            inner_style_full,
            inner_style_partial,
        }
    }
}

impl VerticalSlider {
    pub fn render<D>(
        &self,
        display: &mut D,
        area: Rectangle,
        percentage: &Percentage,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = BinaryColor>,
    {
        let slider_size = Size::new(self.width, area.size.height);
        let area = self
            .outer_style
            .align_element_x(area, slider_size, Align::Center);

        let working_area = self.outer_style.paint(display, area)?;

        let inner_style = if percentage.is_max() {
            &self.inner_style_full
        } else {
            &self.inner_style_partial
        };

        let fill_size = Size::new(
            working_area.size.width,
            (working_area.size.height as f32 * percentage.to_float()) as u32,
        );

        let aligned_area = inner_style.align_element_y(working_area, fill_size, Align::End);
        inner_style.paint(display, aligned_area)?;

        Ok(())
    }
}
