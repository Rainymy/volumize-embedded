use alloc::vec::Vec;

use embedded_graphics::{
    draw_target::DrawTarget, geometry::OriginDimensions, pixelcolor::BinaryColor,
    primitives::Rectangle,
};
use shared_types::DeviceIdentifier;

use crate::{
    InputEvent, RotationEvent,
    display::{
        Screen,
        adjust_volume::{RenderApplication, VolumeAdjustState},
        get_applications, get_device_by_id, get_devices,
        screen::Transition,
        style::{Flexbox, Style},
        system_menu::SystemMenuState,
        util::WrappingInt,
        widget::rounded_rectangle,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplicationMenuState {
    pub selected: WrappingInt,
    pub chunk_index: WrappingInt,
    pub application_count: usize,
    pub device_id: Option<DeviceIdentifier>,
}

impl ApplicationMenuState {
    pub fn new(application_count: usize, device_id: Option<DeviceIdentifier>) -> Self {
        Self {
            selected: WrappingInt::new(0, application_count as i32),
            chunk_index: WrappingInt::new(0, application_count.div_ceil(3) as i32),
            device_id,
            application_count,
        }
    }
}

pub async fn handle_main_menu(state: &mut ApplicationMenuState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.selected.next_clamped();
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            state.selected.prev_clamped();
            Transition::Stay
        }
        InputEvent::SingleClick => {
            async fn resolve_selection(state: &ApplicationMenuState) -> Option<RenderApplication> {
                let selected = state.selected.value().cast_unsigned();
                let device = get_device_by_id(state.device_id.clone()).await;

                // Index 0 is the device entry, if a device is present.
                if selected == 0 {
                    if let Some(device) = &device {
                        return Some(RenderApplication::from(device));
                    }
                }

                let applications = get_applications(state.device_id.clone()).await;
                let offset = if device.is_some() { 1 } else { 0 };
                let absolute_index = selected.saturating_sub(offset);

                applications
                    .get(absolute_index as usize)
                    .map(RenderApplication::from)
            }

            let render_application = defmt::expect!(resolve_selection(state).await);
            let state = VolumeAdjustState::new(state.selected.value(), render_application);
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

/// This is working so bad and wrong need to rework later
fn draw_shadow<D>(display: &mut D, area: Rectangle) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    use crate::display::style::Insets;

    let style = Style::new(BinaryColor::On)
        .background(BinaryColor::On)
        .margin(Insets::new(1, 0, 0, 1))
        .radius_all(3)
        .border(2, BinaryColor::On);

    let _shadow = style.paint(display, area)?;
    Ok(())
}

pub async fn render<D>(display: &mut D, state: &mut ApplicationMenuState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let device_id = state.device_id.clone();

    let device = get_device_by_id(device_id.clone()).await;
    let applications = get_applications(device_id.clone()).await;

    let render_state: Vec<RenderApplication> = device
        .iter()
        .map(RenderApplication::from)
        .chain(applications.iter().map(RenderApplication::from))
        .collect();

    let flexbox = Flexbox::new(display.bounding_box(), 0);
    let allocated_area = flexbox.horizontal(&[1, 1, 1]);

    let window_size = 3;
    let selected = state.selected.value() as usize;
    let chunk_index = selected.div_euclid(window_size);

    let window = render_state
        .chunks(window_size)
        .nth(chunk_index)
        .expect("selected index out of bounds");

    for (i, (area, render)) in allocated_area.iter().zip(window).enumerate() {
        let absolute_index = chunk_index.saturating_mul(window_size).saturating_add(i);
        if selected == absolute_index {
            draw_shadow(display, *area)?;
        }

        rounded_rectangle(display, *area, render.clone())?;
    }

    Ok(())
}
