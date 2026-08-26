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
        util::WrappingInt,
        widget::{ScrollState, ScrollableList},
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct SystemMenuState {
    pub value: WrappingInt,
    pub count: i32,
    pub scroll: ScrollState,
}

impl SystemMenuState {
    pub fn new(menu_item_count: i32) -> Self {
        let menu_count = menu_item_count + 1; // 1 is for settings index.
        Self {
            value: WrappingInt::new(0, menu_count),
            count: menu_count,
            scroll: ScrollState::default(),
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
                let applications = get_applications(device_id.clone()).await;

                let menu_state = ApplicationMenuState::new(applications.len(), device_id);
                Transition::Push(ApplicationList(menu_state))
            }
        }
        InputEvent::LongPress => Transition::Pop,
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: &mut SystemMenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    // items per window
    let window_size = 3;

    let devices = get_devices().await;
    let scrollable = ScrollableList::new(&devices, |device| &device.friendly_name, window_size)
        .with_trailing("Settings");

    scrollable.render(
        display,
        display.bounding_box(),
        &mut state.scroll,
        state.value.value() as usize,
    )
}
