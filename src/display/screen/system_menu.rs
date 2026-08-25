use alloc::{format, vec};
use embedded_graphics::{
    draw_target::DrawTarget, geometry::OriginDimensions, pixelcolor::BinaryColor,
};

use crate::{
    InputEvent, RotationEvent,
    display::{
        Screen::{ApplicationList, Settings},
        Transition,
        application_menu::ApplicationMenuState,
        get_applications, get_devices,
        settings::SettingsState,
        style::{Align, Flexbox, Insets, Style},
        text_style::TextStyle,
        util::WrappingInt,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct SystemMenuState {
    pub value: WrappingInt,
    pub count: i32,
}

impl SystemMenuState {
    pub fn new(menu_item_count: i32) -> Self {
        let menu_count = menu_item_count + 1; // 1 is for settings index.
        Self {
            value: WrappingInt::new(0, menu_count),
            count: menu_count,
        }
    }
}

pub async fn handle_system_menu(state: &mut SystemMenuState, event: InputEvent) -> Transition {
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
            if state.value.value() == state.count - 1 {
                Transition::Push(Settings(SettingsState::new()))
            } else {
                let devices = get_devices().await;
                let device_id = devices
                    .get(state.value.value() as usize)
                    .map(|d| d.id.clone());
                let applications = get_applications(device_id).await;

                let menu_state = ApplicationMenuState::new(applications.len());
                Transition::Push(ApplicationList(menu_state))
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
    let devices = get_devices().await;
    let font_style = TextStyle::Medium.value();

    let flexbox = Flexbox::new(display.bounding_box(), 2i32);
    let allocated_flexbox = flexbox.vertical(&vec![1; 3]);
    let window_size = allocated_flexbox.len();

    let current_selected = state.value.value() as usize;
    let total = state.count as usize;

    let max_offset = total.saturating_sub(window_size);
    let offset = current_selected
        .saturating_sub(window_size.saturating_sub(1))
        .min(max_offset);

    let selected_local = current_selected - offset;

    let style = Style::new(BinaryColor::On)
        .color(BinaryColor::On)
        .margin(Insets::new(0, 0, 2, 2))
        .padding(Insets::all(2))
        .align(Align::Center);

    let style_active = style
        .clone()
        .color(BinaryColor::Off)
        .background(BinaryColor::On)
        .radius_all(4)
        .margin(Insets::new(0, 0, 4, 4))
        .border(2, BinaryColor::On);

    for (i, area) in allocated_flexbox.into_iter().enumerate() {
        let absolute_index = i + offset;

        if let Some(device) = devices.get(absolute_index) {
            let text = format!("{}", device.friendly_name);

            if i == selected_local {
                let font_style = TextStyle::BoldMedium.value();
                let area = style_active.paint(display, area)?;
                style_active.draw_text(display, area, &text, font_style.font)?;
            } else {
                style.draw_text(display, area, &text, font_style.font)?;
            }
        }

        // Last index is the settings entry
        if absolute_index + 1 == total {
            if current_selected + 1 == total {
                let font_style = TextStyle::BoldMedium.value();
                let area = style_active.paint(display, area)?;
                style_active.draw_text(display, area, "Settings", font_style.font)?;
            } else {
                style.draw_text(display, area, "Settings", font_style.font)?;
            }
        }
    }

    Ok(())
}
