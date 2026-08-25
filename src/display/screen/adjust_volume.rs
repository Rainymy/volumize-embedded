use alloc::vec;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    pixelcolor::BinaryColor,
};

use crate::{
    InputEvent, RotationEvent,
    display::{Percentage, screen::Transition, widget::rounded_rectangle},
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct VolumeAdjustState {
    pub value: i32,
}

impl VolumeAdjustState {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

pub async fn handle_volume_adjust(state: &mut VolumeAdjustState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.value = (state.value + 1).min(100);
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            state.value = state.value.saturating_sub(1);
            Transition::Stay
        }
        InputEvent::LongPress => Transition::Pop,
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: &mut VolumeAdjustState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let display_height = display.size().height as u32;
    let mut percentage = Percentage::from_int(state.value as u32);

    // let text_style = TextStyle::Small.value();
    // let font_height = text_style.font.character_size.height;

    for (i, size) in vec![Size::new(40, display_height); 1]
        .into_iter()
        .enumerate()
    {
        let width = size.width as i32;
        let coord = Point::new(i as i32 * width, 0);

        let _ = rounded_rectangle(display, coord, size, &mut percentage);
    }

    Ok(())
}
