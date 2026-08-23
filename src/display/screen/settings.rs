use embedded_graphics::{
    draw_target::DrawTarget, geometry::OriginDimensions, pixelcolor::BinaryColor,
};

use crate::{InputEvent, display::screen::Transition};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SettingsState {
    pub selected: usize,
}

pub fn handle_settings(_state: &mut SettingsState, event: InputEvent) -> Transition {
    match event {
        InputEvent::LongPress => Transition::Pop,
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, _state: SettingsState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let _display = display;
    Ok(())
}
