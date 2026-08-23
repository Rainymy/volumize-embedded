use alloc::{format, vec};

use defmt::info;
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::BinaryColor,
    text::Text,
};

use ssd1306::{
    Ssd1306Async,
    mode::{BufferedGraphicsModeAsync, TerminalDisplaySizeAsync, TerminalModeAsync},
    size::DisplaySizeAsync,
};

use crate::{display::text_style::TextStyle, screens::Screen};

use super::Percentage;
use super::RenderDisplay;
use super::rounded_rectangle;

impl<DI, SIZE> RenderDisplay for Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
    Self: DrawTarget<Color = BinaryColor>,
{
    async fn render(&mut self, _screen: Screen) -> Result<(), u16> {
        if self.clear(BinaryColor::Off).is_err() {
            return Err(100);
        }

        let value = match &_screen {
            // Screen::VolumeAdjust(volume) => volume.value,
            Screen::MainMenu(selected) => selected.selected as u32,
            _ => 57,
        };

        let display_height = SIZE::HEIGHT as u32;
        let mut percentage = Percentage::from_int(value);

        let text_style = TextStyle::Small.value();
        let font_height = text_style.font.character_size.height;

        let text = format!("{:?}", _screen);
        // info!("Screen: {}", text.as_str());
        Text::new(&text, Point::new(0, font_height as i32), text_style)
            .draw(self)
            .ok();

        for (i, size) in vec![Size::new(40, display_height); 1]
            .into_iter()
            .enumerate()
        {
            let width = size.width as i32;
            let coord = Point::new(i as i32 * width, 0);

            let _ = rounded_rectangle(self, coord, size, &mut percentage);
        }

        if self.flush().await.is_err() {
            return Err(200);
        }

        Ok(())
    }
}

static mut LAST_VALUE: Option<i32> = None;

// ---- Terminal mode ----
impl<DI, SIZE> RenderDisplay for Ssd1306Async<DI, SIZE, TerminalModeAsync>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: TerminalDisplaySizeAsync,
{
    async fn render(&mut self, _screen: Screen) -> Result<(), u16> {
        let value = match &_screen {
            Screen::VolumeAdjust(volume) => volume.value,
            Screen::MainMenu(selected) => selected.selected as i32,
            _ => 57,
        };

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
