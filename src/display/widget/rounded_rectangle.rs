use embedded_graphics::geometry::Size;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::{draw_target::DrawTarget, primitives::Rectangle};

use crate::display::Percentage;
use crate::display::adjust_volume::RenderApplication;
use crate::display::style::{Align, Bitmap, Flexbox, Style};
use crate::display::widget::{
    IconWidget, MuteIndicator, PercentageLabel, SliderAlign, VerticalSlider,
};

pub fn rounded_rectangle<D>(
    display: &mut D,
    area: Rectangle,
    render: RenderApplication,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let style = Style::new(BinaryColor::On)
        .padding_all(1)
        .margin_all(2)
        .background(BinaryColor::Off)
        .border(1, BinaryColor::On)
        .radius_all(3)
        .align(Align::Center);

    let content_area = style.paint(display, area)?;

    let flexbox = Flexbox::new(content_area, 3i32);
    let flex_area = flexbox.vertical(&[1, 2, 3, 1, 1]);

    let data = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x1f, 0x80, 0x3f, 0x80, 0x7e, 0x18, 0x7f,
        0xc8, 0x3f, 0xcc, 0x3b, 0xcc, 0x38, 0x9e, 0x3e, 0x3c, 0x3f, 0xf8, 0x1f, 0xe0, 0x07, 0xe0,
        0x02, 0x00,
    ];

    let percentage = Percentage::from_float(render.volume);

    let font_5x8 = embedded_graphics::mono_font::ascii::FONT_5X8;
    let font_4x6 = embedded_graphics::mono_font::ascii::FONT_4X6;

    for (i, area) in flex_area.into_iter().enumerate() {
        match i {
            0 => {
                let element_style = Style::new(BinaryColor::On)
                    .background(BinaryColor::Off)
                    .align(Align::Center);
                let area = element_style.paint(display, area)?;
                element_style.draw_text(display, area, &render.name, &font_5x8)?
            }
            1 => {
                let bitmap = Bitmap::new(&data, 16, 16);
                IconWidget::new(bitmap, BinaryColor::Off).render(display, area)?
            }
            2 => {
                let style = Style::new(BinaryColor::On).align(Align::Center);

                let target_size = Size::new(10, area.size.height);
                let area = style.align_element_x(area, target_size, Align::Center);
                let area = style.paint(display, area)?;

                VerticalSlider::default().render(
                    display,
                    area,
                    &percentage,
                    SliderAlign::Vertical,
                )?
            }
            3 => PercentageLabel::new(&font_4x6).render(display, area, &percentage)?,
            4 => MuteIndicator::new(&font_4x6).render(display, area, true)?,
            _ => {}
        }
    }

    Ok(())
}
