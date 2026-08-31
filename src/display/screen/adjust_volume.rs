use alloc::{format, string::String};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    mono_font::MonoTextStyle,
    pixelcolor::BinaryColor,
    primitives::Rectangle,
};
use shared_types::{AudioApplication, AudioDevice, Identifier};

use crate::{
    InputEvent, RotationEvent,
    display::{
        Percentage,
        screen::Transition,
        style::{Align, Flexbox, Insets, Style},
        text_style::TextStyle,
        util::WrappingInt,
        widget::{SliderAlign, VerticalSlider},
    },
};

#[derive(Debug, Clone)]
pub struct RenderApplication {
    pub id: Identifier,
    pub name: String,
    pub volume: f32,
    pub is_muted: bool,
}

impl From<&AudioDevice> for RenderApplication {
    fn from(device: &AudioDevice) -> Self {
        RenderApplication {
            id: Identifier::Device(device.id.clone()),
            is_muted: device.volume.muted,
            name: device.friendly_name.clone(),
            volume: device.volume.current,
        }
    }
}

impl From<&AudioApplication> for RenderApplication {
    fn from(app: &AudioApplication) -> Self {
        RenderApplication {
            id: Identifier::App(app.process.id),
            is_muted: app.volume.muted,
            name: app.process.name.clone(),
            volume: app.volume.current,
        }
    }
}

#[derive(Debug, Clone)]
pub struct VolumeAdjustState {
    pub value: WrappingInt,
    pub application: RenderApplication,
}

impl VolumeAdjustState {
    pub fn new(value: i32, application: RenderApplication) -> Self {
        Self {
            value: WrappingInt::new(value.min(0), 101),
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
        InputEvent::LongPress => Transition::Pop,
        InputEvent::DoubleClick => {
            state.application.is_muted = !state.application.is_muted;
            Transition::Stay
        }
        InputEvent::SingleClick => Transition::Ignored,
    }
}

pub async fn render<D>(display: &mut D, state: &mut VolumeAdjustState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    let style = Style::new(BinaryColor::Off)
        .background(BinaryColor::Off)
        .margin_all(2)
        .radius_all(3)
        .border(1, BinaryColor::On);

    let allocated_area = style.paint(display, display.bounding_box())?;

    let flexbox = Flexbox::new(allocated_area, 0);
    let flexbox_area = flexbox.vertical(&[1, 2, 1]);

    for (i, area) in flexbox_area.into_iter().enumerate() {
        match i {
            0 => {
                let area_style = Style::new(BinaryColor::Off)
                    .background(BinaryColor::Off)
                    .color(BinaryColor::On)
                    .margin(Insets::new(5, 0, 3, 7))
                    // .radius_all(3);
                    .radius_all(6);

                let font_style = TextStyle::Medium.value();

                let area = area_style.paint(display, area)?;
                let _area = area_style.draw_text(
                    display,
                    area,
                    &state.application.name,
                    font_style.font,
                )?;
            }
            1 => {
                let style = Style::new(BinaryColor::On).margin(Insets::new(10, 5, 0, 5));
                let area = style.paint(display, area)?;

                let font = embedded_graphics::mono_font::ascii::FONT_6X12;
                let percentage = Percentage::from_int(state.value.value().cast_unsigned());

                let info = format!("{:.2}%", percentage.to_percentage());

                let title_style = style.clone().align(Align::Center);

                let title_area = area.top_left - font.character_size - Size::new(0, 4);
                let title_rect = Rectangle::new(
                    title_area,
                    Size::new(area.size.width + 50, area.size.height),
                );

                title_style.draw_text(display, title_rect, &info, &font)?;

                VerticalSlider::default().render_labeled(
                    display,
                    area,
                    &percentage,
                    SliderAlign::Horizontal,
                    &info,
                    MonoTextStyle::new(&font, BinaryColor::On),
                )?;
            }
            2 => {
                let font = embedded_graphics::mono_font::ascii::FONT_6X12;
                let style = Style::new(BinaryColor::Off)
                    .background(BinaryColor::Off)
                    .align(Align::Center)
                    .color(BinaryColor::On);
                let area = style.paint(display, area)?;
                let muted_text = format!(
                    "{}",
                    if state.application.is_muted {
                        "Muted"
                    } else {
                        "Unmuted"
                    }
                );
                style.draw_text(display, area, &muted_text, &font)?;
            }
            _ => {}
        }
    }

    // let display_size = display.size();
    // let display_height = display_size.height as i32;
    // let display_width = display_size.width as i32;
    // let max_chars = ((display_width / 4).max(1)) as usize;

    // Text::with_baseline(
    //     &truncate_name(&state.application.name, max_chars),
    //     Point::new(10, display_height - 10),
    //     MonoTextStyle::new(&FONT_4X6, BinaryColor::On),
    //     Baseline::Top,
    // )
    // .draw(display)?;

    Ok(())
}

fn truncate_name(name: &str, max_chars: usize) -> String {
    let mut out: String = String::new();
    for c in name.chars().take(max_chars.max(1)) {
        out.push(c);
    }
    out
}
