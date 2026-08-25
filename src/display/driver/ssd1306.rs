use alloc::format;

use defmt::info;
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

use ssd1306::{
    Ssd1306Async,
    mode::{BufferedGraphicsModeAsync, TerminalDisplaySizeAsync, TerminalModeAsync},
    size::DisplaySizeAsync,
};

use crate::{
    display::Screen,
    display::{RenderDisplay, screen},
};

impl<DI, SIZE> RenderDisplay for Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
    Self: DrawTarget<Color = BinaryColor>,
{
    async fn render(&mut self, screen: &mut Screen) -> Result<(), u16> {
        if self.clear(BinaryColor::Off).is_err() {
            return Err(100);
        }

        match screen {
            Screen::ApplicationList(state) => screen::application_menu::render(self, state).await,
            Screen::Settings(state) => screen::settings::render(self, state).await,
            Screen::VolumeAdjust(state) => screen::adjust_volume::render(self, state).await,
            Screen::SystemMenu(state) => screen::system_menu::render(self, state).await,
        }
        .map_err(|_| 400u16)?;

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
    async fn render(&mut self, screen: &mut Screen) -> Result<(), u16> {
        let value = match &screen {
            Screen::VolumeAdjust(volume) => volume.value,
            Screen::ApplicationList(selected) => selected.selected.value(),
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
