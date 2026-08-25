use alloc::vec;
use alloc::vec::Vec;

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::BinaryColor,
};
use shared_types::DeviceIdentifier;

use crate::{
    InputEvent, RotationEvent,
    display::{
        Percentage, Screen, adjust_volume::VolumeAdjustState, get_applications, get_device_by_id,
        get_devices, screen::Transition, system_menu::SystemMenuState, util::WrappingInt,
        widget::rounded_rectangle,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationMenuState {
    pub selected: WrappingInt,
    pub application_count: usize,
    pub device_id: Option<DeviceIdentifier>,
}

impl ApplicationMenuState {
    pub fn new(application_count: usize, device_id: Option<DeviceIdentifier>) -> Self {
        Self {
            selected: WrappingInt::new(0, application_count as i32),
            device_id,
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
        InputEvent::SingleClick => {
            let state = VolumeAdjustState::new(state.selected.value());
            Transition::Push(Screen::VolumeAdjust(state))
        }
        InputEvent::LongPress => {
            let devices = get_devices().await;
            let state = SystemMenuState::new(devices.len() as i32);
            Transition::Push(Screen::SystemMenu(state))
        }
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: &mut ApplicationMenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let device_id = state.device_id.clone();

    // no device, no applications
    let device = get_device_by_id(device_id.clone()).await;
    let applications = get_applications(device_id.clone()).await;

    let display_height = display.size().height as u32;

    let total_count = applications.len() + if device.is_some() { 1 } else { 0 };

    #[derive(Clone, Default)]
    struct PercentageState {
        value: Percentage,
    }

    let mut percentage_state: Vec<PercentageState> = Vec::new();

    if let Some(device) = device {
        percentage_state.push(PercentageState {
            value: Percentage::from_float(device.volume.current),
        });
    }

    for application in &applications {
        percentage_state.push(PercentageState {
            value: Percentage::from_float(application.volume.current),
        });
    }

    for (i, size) in vec![Size::new(40, display_height); total_count]
        .into_iter()
        .enumerate()
    {
        let mut percentage = percentage_state.get(i).cloned().unwrap_or_default();

        let width = size.width as i32;
        let coord = Point::new(i as i32 * width, 0);

        let _ = rounded_rectangle(display, coord, size, &mut percentage.value);
    }

    Ok(())
}
