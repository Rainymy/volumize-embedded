use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{
        MonoTextStyleBuilder,
        ascii::FONT_5X8,
    },
    pixelcolor::BinaryColor,
    text::Text,
};
use ssd1306::{
    Ssd1306,
    mode::{BufferedGraphicsMode, TerminalMode},
    prelude::WriteOnlyDataCommand,
    size::{DisplaySize, DisplaySize128x64},
};

use ufmt::uWrite;
use core::fmt::Write;

use crate::{RenderDisplay, log};

impl<DI, SIZE> RenderDisplay for Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
    Self: DrawTarget<Color = BinaryColor>,
{
    fn render<W: uWrite>(&mut self, _serial: &mut W, _value: i32, _button_pressed: bool) -> Result<(), u16> {
        if self.clear(BinaryColor::Off).is_err() {
            return Err(100);
        }

        arduino_hal::delay_ms(50);

        let top = FONT_5X8.character_size.height;
        // let _ = serial.write_str("writing text to screen\n");
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_5X8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        if Text::new("Volume: ", Point::new(0, top as i32), style).draw(self).is_err() {
            return Err(500);
        }

        if self.flush().is_err() {
            return Err(200);
        }

        Ok(())
    }
}

// ---- Terminal mode ----
impl<DI> RenderDisplay for Ssd1306<DI, DisplaySize128x64, TerminalMode>
where
    DI: WriteOnlyDataCommand,
{
    fn render<W: uWrite>(&mut self, serial: &mut W, _value: i32, _button_pressed: bool) -> Result<(), u16> {
        let (w, h) = self.dimensions();

        log!(serial, "{}x{}\n", w, h);

        for c in 97..123 {
            let _ = self.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
        }

        Ok(())
    }
}