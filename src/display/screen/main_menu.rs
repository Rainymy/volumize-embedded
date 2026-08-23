use alloc::{format, vec};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::BinaryColor,
    primitives::Rectangle,
};

use crate::{
    InputEvent, RotationEvent,
    display::{
        Screen,
        adjust_volume::VolumeAdjustState,
        screen::Transition,
        style::{Align, Flexbox, Insets, Style},
        text_style::TextStyle,
    },
};

const MENU_ITEM_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MainMenuState {
    pub selected: usize,
}

pub fn handle_main_menu(state: &mut MainMenuState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.selected = state.selected.saturating_add(1) % MENU_ITEM_COUNT;
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            if state.selected == 0 {
                state.selected = MENU_ITEM_COUNT - 1;
            } else {
                state.selected = state.selected.saturating_sub(1);
            }
            Transition::Stay
        }
        InputEvent::SingleClick => match state.selected {
            0 => Transition::Push(Screen::VolumeAdjust(VolumeAdjustState::default())),
            _ => Transition::Stay,
        },
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: MainMenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let display_width = display.size().width as u32;

    let text_style = TextStyle::Small.value();
    let font_height = text_style.font.character_size.height;

    let width = display_width / 3;
    let height = font_height * 2;

    let flexbox = Flexbox::new(display.bounding_box(), 0i32);
    let items = flexbox.vertical(&vec![1; MENU_ITEM_COUNT]);

    let style = Style::new(BinaryColor::On)
        .padding(Insets::all(2))
        .margin(Insets::all(2))
        .border(2, BinaryColor::On)
        .align(Align::Center);

    let style_selected = style
        .clone()
        .background(BinaryColor::On)
        .color(BinaryColor::Off);

    for (i, size) in items.into_iter().enumerate() {
        let point = Point::new((display_width / 2 - width / 2) as i32, size.top_left.y);
        let area = Rectangle::new(point, Size::new(width, height));
        // let area = style.paint(display, area)?;

        if i == state.selected {
            let area = style_selected.paint(display, area)?;
            style_selected.draw_text(display, area, &format!("Menu {}", i + 1), text_style.font)?;
        } else {
            style.draw_text(display, area, &format!("Menu {}", i + 1), text_style.font)?;
        }

        // style.draw_text(display, area, &format!("Menu {}", i + 1), text_style.font)?;
    }

    Ok(())
}
