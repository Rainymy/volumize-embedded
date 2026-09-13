use alloc::format;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::OriginDimensions,
    mono_font::ascii::{FONT_6X12, FONT_7X14},
    pixelcolor::BinaryColor,
};
use shared_types::protocol::{Command, CommandRequest, Envelope};

use crate::{
    InputEvent, OUT_CHANNEL,
    display::{
        Transition, is_waiting_for_data,
        style::{Align, Flexbox, Style},
    },
};

static mut TIME: u32 = 0;

#[derive(Default, Debug, Clone)]
pub struct WaitForDataState {
    sent: bool,
}

pub async fn handle_wait_for_data(state: &mut WaitForDataState, event: InputEvent) -> Transition {
    match event {
        InputEvent::SingleClick => {
            match OUT_CHANNEL.try_send(Envelope::Command(CommandRequest {
                id: 12,
                command: Command::GetPlaybackDevices,
            })) {
                Ok(_) => {
                    defmt::info!("sent");
                    state.sent = true;
                }
                Err(e) => {
                    let msg = format!("Error msg: {:?}", e);
                    defmt::info!("{}", msg);
                }
            }

            // Click to initiate sending data to host, and wait for receive.
            // After receiving, transition to Pop else Stay.
            if is_waiting_for_data() {
                Transition::Stay
            } else {
                Transition::Pop
            }
        }
        _ => Transition::Stay,
    }
}

pub async fn render<D>(display: &mut D, state: &mut WaitForDataState) -> Result<(), D::Error>
where
    D: DrawTarget<Color = BinaryColor> + OriginDimensions,
{
    unsafe {
        TIME += 1;
    }

    let style = Style::new(BinaryColor::On)
        .border(1, BinaryColor::On)
        .align(Align::Center)
        .radius_all(4)
        .margin_all(4);

    let allocated_area = style.paint(display, display.bounding_box())?;

    let flexbox = Flexbox::new(allocated_area, 2);
    let areas = flexbox.vertical(&[1, 2, 1]);

    for (i, area) in areas.into_iter().enumerate() {
        match i {
            0 => {
                if state.sent {
                    style.draw_text(display, area, "Data sent", &FONT_6X12)?
                }
            }
            1 => style.draw_text(display, area, "Click to send", &FONT_7X14)?,
            2 => {
                let time_text = format!("Time: {}", unsafe { TIME });
                style.draw_text(display, area, &time_text, &FONT_6X12)?
            }
            _ => {}
        }
    }

    Ok(())
}
