use core::cell::RefCell;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb565};
use lcd_async::raw_framebuf::RawFrameBuf;

use crate::{
    display::{
        RenderDisplay, Screen, screen,
        theme::{FromTheme, Theme, ThemeColor},
    },
    init_display::Ili9341DisplayType,
};

// const HEIGHT: usize = ILI9341Rgb565::FRAMEBUFFER_SIZE.0 as usize;
// const WIDTH: usize = ILI9341Rgb565::FRAMEBUFFER_SIZE.1 as usize;
const HEIGHT: usize = 240 as usize;
const WIDTH: usize = 320 as usize;

const FRAME_BUFFER_SIZE: usize = WIDTH * HEIGHT * 2; // 2 bytes per pixel (Rgb565)

static FRAME_BUFFER: Mutex<CriticalSectionRawMutex, RefCell<[u8; FRAME_BUFFER_SIZE]>> =
    Mutex::new(RefCell::new([0u8; FRAME_BUFFER_SIZE]));

impl RenderDisplay for Ili9341DisplayType {
    async fn render(&mut self, screen: &mut Screen, theme: &Theme) -> Result<(), u16> {
        let frame_buffer = FRAME_BUFFER.lock().await;
        let mut buffer = frame_buffer.borrow_mut();
        let mut buffer = RawFrameBuf::<Rgb565, _>::new(buffer.as_mut_slice(), WIDTH, HEIGHT);

        let background = FromTheme::from_theme(theme, ThemeColor::Foreground);
        let _ = buffer.clear(background);

        use screen::{adjust_volume, application_menu, settings, system_menu, wait_for_data};
        match screen {
            Screen::ApplicationList(state) => {
                application_menu::render(&mut buffer, theme, state).await
            }
            Screen::Settings(state) => settings::render(&mut buffer, theme, state).await,
            Screen::VolumeAdjust(state) => adjust_volume::render(&mut buffer, theme, state).await,
            Screen::SystemMenu(state) => system_menu::render(&mut buffer, theme, state).await,
            Screen::WaitingForData(state) => wait_for_data::render(&mut buffer, theme, state).await,
        }
        .map_err(|_| 400u16)?;

        defmt::info!("New Frame");

        self.show_raw_data(0, 0, WIDTH as u16, HEIGHT as u16, buffer.as_bytes())
            .await
            .map_err(|_| 100u16)
    }
}
