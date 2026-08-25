#![allow(unused)]
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
        DEVICES_LIST, Screen,
        adjust_volume::VolumeAdjustState,
        get_devices,
        screen::Transition,
        style::{Align, Flexbox, Insets, Style},
        system_menu::SystemMenuState,
        text_style::TextStyle,
        util::WrappingInt,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ApplicationMenuState {
    pub selected: WrappingInt,
    pub application_count: usize,
}

impl ApplicationMenuState {
    pub const MENU_ITEM_COUNT: usize = 3;

    pub fn new(application_count: usize) -> Self {
        Self {
            selected: WrappingInt::new(0, application_count as i32),
            application_count,
        }
    }
}

pub async fn handle_main_menu(state: &mut ApplicationMenuState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.selected.next();
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            state.selected.prev();
            Transition::Stay
        }
        InputEvent::SingleClick => match state.selected.value() {
            0 => Transition::Push(Screen::VolumeAdjust(VolumeAdjustState::default())),
            _ => Transition::Stay,
        },
        InputEvent::LongPress => {
            let devices = get_devices().await;
            Transition::Push(Screen::SystemMenu(SystemMenuState::new(
                devices.len() as i32
            )))
        }
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: ApplicationMenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let display_width = display.size().width as u32;

    let text_style = TextStyle::Small.value();
    let font_height = text_style.font.character_size.height;

    let width = display_width / 3;
    let height = font_height * 2;

    // let flexbox = Flexbox::new(display.bounding_box(), 0i32);
    // let items = flexbox.vertical(&vec![1; ApplicationMenuState::MENU_ITEM_COUNT]);

    let style = Style::new(BinaryColor::On)
        .padding(Insets::all(2))
        .margin(Insets::all(2))
        .border(2, BinaryColor::On)
        .align(Align::Center);

    let style_selected = style
        .clone()
        .background(BinaryColor::On)
        .color(BinaryColor::Off);

    let devices = critical_section::with(|cs| DEVICES_LIST.borrow(cs).borrow().clone());

    for (i, device) in devices.into_iter().enumerate() {
        let point = Point::new(
            (display_width / 2 - width / 2) as i32,
            (font_height * (i as u32)) as i32,
        );
        let area = Rectangle::new(point, Size::new(width, height));
        let text = &device.friendly_name;

        if i == state.selected.value() as usize {
            let area = style_selected.paint(display, area)?;
            style_selected.draw_text(
                display,
                area,
                &format!("{text} {}", i + 1),
                text_style.font,
            )?;
        } else {
            style.draw_text(display, area, &format!("{text} {}", i + 1), text_style.font)?;
        }

        // style.draw_text(display, area, &format!("Menu {}", i + 1), text_style.font)?;
    }

    Ok(())
}
