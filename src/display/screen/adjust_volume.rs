use alloc::{string::String, vec};
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Point, Size},
    mono_font::{MonoTextStyle, iso_8859_9::FONT_4X6},
    pixelcolor::BinaryColor,
    primitives::{Line, Primitive, PrimitiveStyle, Rectangle},
    text::{Baseline, Text},
};
use shared_types::{AudioApplication, SessionDirection};

use crate::{
    InputEvent, RotationEvent,
    display::{Percentage, screen::Transition, util::WrappingInt, widget::rounded_rectangle},
};

#[derive(Debug, Clone)]
pub struct VolumeAdjustState {
    pub value: WrappingInt,
    pub application: AudioApplication,
}

impl VolumeAdjustState {
    pub fn new(value: i32, application: AudioApplication) -> Self {
        Self {
            value: WrappingInt::new(value.min(0), 100),
            application: application,
        }
    }
}

pub async fn handle_volume_adjust(state: &mut VolumeAdjustState, event: InputEvent) -> Transition {
    match event {
        InputEvent::Rotation(RotationEvent::Next) => {
            state.value.next();
            Transition::Stay
        }
        InputEvent::Rotation(RotationEvent::Previous) => {
            state.value.prev();
            Transition::Stay
        }
        InputEvent::SingleClick => Transition::Stay,
        InputEvent::LongPress => Transition::Pop,
        _ => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: &mut VolumeAdjustState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let display_size = display.size();
    let display_height = display_size.height as i32;
    let display_width = display_size.width as i32;

    let label_height = 8;
    let bar_area_height = (display_height - label_height - 8).max(0);

    let text_style = MonoTextStyle::new(&FONT_4X6, BinaryColor::On);

    draw_direction_glyph(display, Point::new(2, 3), &state.application.direction)?;

    if state.application.volume.muted {
        draw_mute_glyph(display, Point::new(display_width - 9, 0))?;
    }

    Rectangle::new(
        Point::new(0, 0),
        Size::new(display_width as u32, display_height as u32),
    )
    .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
    .draw(display)?;

    let bar_area = Rectangle::new(
        Point::new(4, label_height),
        Size::new(
            (display_width as u32).saturating_sub(8),
            bar_area_height as u32,
        ),
    );

    let mut percentage = Percentage::from_float(state.application.volume.current);

    rounded_rectangle(display, bar_area, &mut percentage)?;

    let max_chars = ((display_width / 4).max(1)) as usize;
    let name = truncate_name(&state.application.process.name, max_chars);

    Text::with_baseline(
        &name,
        Point::new(2, display_height - 7),
        text_style,
        Baseline::Top,
    )
    .draw(display)?;

    Ok(())
}

fn draw_direction_glyph<D>(
    display: &mut D,
    origin: Point,
    direction: &SessionDirection,
) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    use embedded_graphics::prelude::{Drawable, Primitive};

    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    match direction {
        SessionDirection::Render => {
            Line::new(origin, origin + Point::new(4, -3))
                .into_styled(style)
                .draw(display)?;
            Line::new(origin, origin + Point::new(4, 3))
                .into_styled(style)
                .draw(display)?;
        }
        SessionDirection::Capture => {
            Line::new(origin + Point::new(4, -3), origin)
                .into_styled(style)
                .draw(display)?;
            Line::new(origin + Point::new(4, 3), origin)
                .into_styled(style)
                .draw(display)?;
        }
        SessionDirection::Unknown => {}
    }
    Ok(())
}

fn draw_mute_glyph<D>(display: &mut D, origin: Point) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor>,
{
    use embedded_graphics::prelude::{Drawable, Primitive};
    use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle};

    let style = PrimitiveStyle::with_stroke(BinaryColor::On, 1);
    Circle::new(origin, 6).into_styled(style).draw(display)?;
    Line::new(origin, origin + Point::new(6, 6))
        .into_styled(style)
        .draw(display)?;
    Ok(())
}

fn truncate_name(name: &str, max_chars: usize) -> String {
    let mut out: String = String::new();
    for c in name.chars().take(max_chars.max(1)) {
        out.push(c);
    }
    out
}
