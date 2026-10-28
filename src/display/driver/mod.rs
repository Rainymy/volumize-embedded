use embedded_graphics::{
    draw_target::DrawTarget, geometry::OriginDimensions, pixelcolor::PixelColor,
};

use crate::display::{
    Screen,
    theme::{FromTheme, Theme},
};

mod ili9341;
mod ssd1306;

async fn render_screen_state<D, Color: PixelColor>(
    canvas: &mut D,
    screen: &mut Screen,
    theme: &Theme,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = Color> + OriginDimensions,
    D::Color: FromTheme,
{
    use crate::display::{adjust_volume, application_menu, settings, system_menu, wait_for_data};
    match screen {
        Screen::ApplicationList(state) => application_menu::render(canvas, theme, state).await,
        Screen::Settings(state) => settings::render(canvas, theme, state).await,
        Screen::VolumeAdjust(state) => adjust_volume::render(canvas, theme, state).await,
        Screen::SystemMenu(state) => system_menu::render(canvas, theme, state).await,
        Screen::WaitingForData(state) => wait_for_data::render(canvas, theme, state).await,
    }
}
