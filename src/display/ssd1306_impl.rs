#![allow(dead_code)]

use alloc::{format, vec};
use core::fmt::Write;

use defmt::info;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
};
use ssd1306::{
    Ssd1306,
    mode::{BufferedGraphicsMode, TerminalMode},
    prelude::WriteOnlyDataCommand,
    size::{DisplaySize, DisplaySize128x64},
};

// use crate::{digits::digit_to_str, helper::digits::number_to_vec};

use super::ButtonEvent;
use super::RenderDisplay;
use super::{Align, Bitmap, Corners, Flexbox, Percentage, Style};

#[allow(dead_code)]
enum TextStyle {
    Small,
    Medium,
    Large,
}

impl TextStyle {
    const fn value(&self) -> MonoTextStyle<'_, BinaryColor> {
        use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_8X13, FONT_9X15};

        match self {
            TextStyle::Small => MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
            TextStyle::Medium => MonoTextStyleBuilder::new()
                .font(&FONT_8X13)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
            TextStyle::Large => MonoTextStyleBuilder::new()
                .font(&FONT_9X15)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
        }
    }
}

fn rounded_rectangle<D>(
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

    // percentage.increment_percentage(0.1);
    // percentage.increment_percentage(5);

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

impl<DI, SIZE> RenderDisplay for Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
    Self: DrawTarget<Color = BinaryColor>,
{
    fn render(&mut self, value: f32, _button_state: ButtonEvent) -> Result<(), u16> {
        if self.clear(BinaryColor::Off).is_err() {
            return Err(100);
        }

        let display_height = self.size().height;

        let mut percentage = Percentage::from_float(value);

        // let small_style = TextStyle::Small.value();
        // let small_top = small_style.font.character_size.height;

        // let big_style = TextStyle::Large.value();
        // let big_top = big_style.font.character_size.height;

        for (i, size) in vec![Size::new(40, display_height); 1]
            .into_iter()
            .enumerate()
        {
            let width = size.width as i32;
            let coord = Point::new(i as i32 * width, 0);

            let _ = rounded_rectangle(self, coord, size, &mut percentage);
        }

        if self.flush().is_err() {
            return Err(200);
        }

        Ok(())
    }
}

static mut LAST_VALUE: Option<f32> = None;

// ---- Terminal mode ----
impl<DI> RenderDisplay for Ssd1306<DI, DisplaySize128x64, TerminalMode>
where
    DI: WriteOnlyDataCommand,
{
    fn render(&mut self, value: f32, _button_state: ButtonEvent) -> Result<(), u16> {
        if unsafe { LAST_VALUE } == Some(value) {
            return Ok(());
        }
        unsafe {
            LAST_VALUE = Some(value);
        }

        let _ = self.clear();

        let digits = format!("volume: {}", value);
        info!("Volume: {}", value);

        // Write full string does not work, it only displays the last character.
        // let _ = self.write_str(&_digits);
        // Instead, we write each character individually. Works around the issue.
        for c in digits.as_bytes() {
            let bind = &[*c];
            let _ = self.write_str(unsafe { core::str::from_utf8_unchecked(bind) });
        }
        let _ = self.set_position(0, 2);

        for c in 33..123 {
            let bind = &[c];
            let _ = self.write_str(unsafe { core::str::from_utf8_unchecked(bind) });
        }

        Ok(())
    }
}
