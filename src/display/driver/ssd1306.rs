use alloc::format;

use defmt::info;
use display_interface::AsyncWriteOnlyDataCommand;
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::BinaryColor};

use ssd1306::{
    Ssd1306Async,
    mode::{BufferedGraphicsModeAsync, TerminalDisplaySizeAsync, TerminalModeAsync},
    size::DisplaySizeAsync,
};

use crate::display::{
    RenderDisplay, Screen,
    theme::{FromTheme, Theme, ThemeColor},
};

impl<DI, SIZE> RenderDisplay for Ssd1306Async<DI, SIZE, BufferedGraphicsModeAsync<SIZE>>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: DisplaySizeAsync,
    Self: DrawTarget<Color = BinaryColor>,
{
    async fn render(&mut self, screen: &mut Screen, theme: &Theme) -> Result<(), u16> {
        let background = FromTheme::from_theme(theme, ThemeColor::Background);
        self.clear(background).map_err(|_| 100u16)?;

        super::render_screen_state(self, screen, theme)
            .await
            .map_err(|_| 400u16)?;

        self.flush().await.map_err(|_| 200u16)
    }
}

static mut LAST_VALUE: Option<i32> = None;

// ---- Terminal mode ----
impl<DI, SIZE> RenderDisplay for Ssd1306Async<DI, SIZE, TerminalModeAsync>
where
    DI: AsyncWriteOnlyDataCommand,
    SIZE: TerminalDisplaySizeAsync,
{
    async fn render(&mut self, screen: &mut Screen, _theme: &Theme) -> Result<(), u16> {
        let value = match &screen {
            Screen::VolumeAdjust(volume) => volume.value.value(),
            Screen::ApplicationList(selected) => selected.selected.value(),
            _ => 57,
        };

        if unsafe { LAST_VALUE } == Some(value) {
            return Ok(());
        }
        unsafe {
            LAST_VALUE = Some(value);
        }

        let _ = self.clear().await;

        let digits = format!("volume: {}", value);
        info!("Volume: {}", value);

        // Write full string does not work, it only displays the last character.
        // let _ = self.write_str(&_digits);
        // Instead, we write each character individually. Works around the issue.
        for c in digits.as_bytes() {
            let bind = &[*c];
            let _ = self
                .write_str(unsafe { core::str::from_utf8_unchecked(bind) })
                .await;
        }
        let _ = self.set_position(0, 2).await;

        for c in 33..123 {
            let bind = &[c];
            let _ = self
                .write_str(unsafe { core::str::from_utf8_unchecked(bind) })
                .await;
        }

        Ok(())
    }
}
