use alloc::format;
use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::{draw_target::DrawTarget, primitives::Rectangle};

use super::{Align, Bitmap, Corners, Flexbox, Percentage, Style};

pub fn rounded_rectangle<D>(
    display: &mut D,
    start: Point,
    size: Size,
    percentage: &mut Percentage,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    let bounding_box = display.bounding_box();
    let coordinate = bounding_box.top_left + start;

    let style = Style::new(BinaryColor::On)
        .padding_all(1)
        .margin_all(2)
        .background(BinaryColor::Off)
        .border(1, BinaryColor::On)
        .radius_all(3)
        .align(Align::Center);

    let allocated_area = Rectangle::new(coordinate, size);
    let content_area = style.paint(display, allocated_area)?;

    let flexbox = Flexbox::new(content_area, 3i32);
    let flex_area = flexbox.vertical(&[1, 2, 3, 1, 1]);

    for (i, area) in flex_area.into_iter().enumerate() {
        let color = BinaryColor::Off;

        // println!("{}: Area - {} ", i, area.size);

        let element_style = Style::new(color.invert())
            .background(color)
            // .border(1, color.invert())
            .align(Align::Center);

        let area = element_style.paint(display, area)?;

        if i == 0 {
            let font = embedded_graphics::mono_font::ascii::FONT_5X8;
            element_style.draw_text(display, area, "ok", &font)?;
        }

        if i == 1 {
            let data = [
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0e, 0x00, 0x1f, 0x80, 0x3f, 0x80, 0x7e, 0x18,
                0x7f, 0xc8, 0x3f, 0xcc, 0x3b, 0xcc, 0x38, 0x9e, 0x3e, 0x3c, 0x3f, 0xf8, 0x1f, 0xe0,
                0x07, 0xe0, 0x02, 0x00,
            ];

            // Implement a bitmap resizing function if needed.
            element_style.draw_bitmap(
                display,
                area,
                &Bitmap::new(&data, 16, 16),
                color.invert(),
                None,
            )?;
        }

        if i == 2 {
            let slider_style = Style::new(BinaryColor::On)
                .margin_all(1)
                .padding_all(1)
                .border(1, BinaryColor::On)
                .background(BinaryColor::Off)
                .radius_all(3)
                .align(Align::Center);

            let inner_style = {
                let mut style = Style::new(BinaryColor::On)
                    // .radius_all(3)
                    .padding_all(1)
                    .background(BinaryColor::On)
                    .border(1, BinaryColor::On);

                if percentage.is_max() {
                    style = style.radius_all(3);
                } else {
                    style = style.radius(Corners::new(0, 0, 3, 3));
                }

                style
            };

            let slider_size = Size::new(10, area.size.height);
            let area = slider_style.align_element_x(area, slider_size, Align::Center);

            let working_area = slider_style.paint(display, area)?;

            // println!("Percentage {:.2}", percentage.to_percentage());

            let aligned_area = inner_style.align_element_y(
                working_area,
                Size::new(
                    working_area.size.width,
                    (working_area.size.height as f32 * percentage.to_float()) as u32,
                ),
                Align::End,
            );

            let _working_area = inner_style.paint(display, aligned_area)?;
        }

        if i == 3 {
            let font = embedded_graphics::mono_font::ascii::FONT_4X6;

            let formatted_progress = format!("{:.1}%", percentage.to_percentage());
            element_style.draw_text(display, area, &formatted_progress, &font)?;
        }

        if i == 4 {
            let is_muted = true;
            let font = embedded_graphics::mono_font::ascii::FONT_4X6;
            element_style.draw_text(display, area, if is_muted { "M" } else { "U" }, &font)?;
        }
    }

    Ok(())
}
