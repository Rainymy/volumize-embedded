use alloc::format;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point},
    pixelcolor::BinaryColor,
    text::Text,
};

use crate::{
    InputEvent, RotationEvent,
    display::{screen::Transition, text_style::TextStyle, util::WrappingInt},
};

#[derive(Debug, Clone, PartialEq)]
pub struct SettingsState {
    pub selected: WrappingInt,
}

impl SettingsState {
    const SETTINGS_ITEM_COUNT: usize = 4;

    pub fn new() -> Self {
        Self {
            selected: WrappingInt::new(0, Self::SETTINGS_ITEM_COUNT as i32),
        }
    }
}

pub async fn handle_settings(_state: &mut SettingsState, event: InputEvent) -> Transition {
    match event {
        InputEvent::LongPress => Transition::Pop,
        InputEvent::Rotation(RotationEvent::Next) => {
            _state.selected.next();
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            _state.selected.prev();
            Transition::Stay
        }
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: SettingsState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let font_style = TextStyle::Small.value();
    let height = font_style.font.character_size.height;

    let text = format!("Settings Menu: {}", state.selected.value());

    Text::new(&text, Point::new(0, height as i32), font_style).draw(display)?;

    Ok(())
}
