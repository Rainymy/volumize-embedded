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
    display::{
        Screen::Settings, Transition, settings::SettingsState, text_style::TextStyle,
        util::WrappingInt,
    },
};

const SYSTEM_MENU_COUNT: i32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct SystemMenuState {
    pub value: WrappingInt,
}

impl SystemMenuState {
    pub fn new() -> Self {
        Self {
            value: WrappingInt::new(0, SYSTEM_MENU_COUNT),
        }
    }
}

pub fn handle_system_menu(state: &mut SystemMenuState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.value.next();
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            state.value.prev();
            Transition::Stay
        }
        InputEvent::SingleClick => {
            if state.value.value() == SYSTEM_MENU_COUNT - 1 {
                Transition::Push(Settings(SettingsState::new()))
            } else {
                Transition::Stay
            }
        }
        InputEvent::LongPress => Transition::Pop,
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: SystemMenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let font_style = TextStyle::Small.value();
    let height = font_style.font.character_size.height;
    let text = format!("System Menu: {}", state.value.value());
    let text2 = format!("Settings menu at: {}", SYSTEM_MENU_COUNT - 1);

    Text::new(&text, Point::new(0, height as i32), font_style).draw(display)?;
    Text::new(&text2, Point::new(0, 3 * height as i32), font_style).draw(display)?;

    Ok(())
}
