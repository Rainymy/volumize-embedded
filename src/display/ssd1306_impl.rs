use alloc::format;
use defmt::info;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    text::Text,
};
use ssd1306::{
    I2CDisplayInterface, Ssd1306,
    mode::{BufferedGraphicsMode, DisplayConfig, TerminalMode},
    prelude::{DisplayRotation, I2CInterface, WriteOnlyDataCommand},
    size::{DisplaySize, DisplaySize128x64},
};

// use crate::{digits::digit_to_str, helper::digits::number_to_vec};

use core::{fmt::Write, mem::MaybeUninit};
use esp_hal::Blocking;

pub type DisplayI2c<'a> = esp_hal::i2c::master::I2c<'a, Blocking>;
pub type DisplayInterface<'a> = I2CInterface<DisplayI2c<'a>>;

pub type GraphicsMode = BufferedGraphicsMode<DisplaySize128x64>;
// pub type GraphicsMode = TerminalMode;
pub type DisplayType = Ssd1306<DisplayInterface<'static>, DisplaySize128x64, GraphicsMode>;

use super::ButtonEvent;
use super::RenderDisplay;

static mut DISPLAY: MaybeUninit<DisplayType> = MaybeUninit::zeroed();

#[allow(dead_code)]
enum TextStyle {
    Small,
    Medium,
    Large,
}

impl TextStyle {
    const fn value(&self) -> MonoTextStyle<'_, BinaryColor> {
        use embedded_graphics::mono_font::ascii::{FONT_6X10, FONT_8X13, FONT_9X15};

        match self {
            TextStyle::Small => MonoTextStyleBuilder::new()
                .font(&FONT_6X10)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
            TextStyle::Medium => MonoTextStyleBuilder::new()
                .font(&FONT_8X13)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
            TextStyle::Large => MonoTextStyleBuilder::new()
                .font(&FONT_9X15)
                .text_color(BinaryColor::On)
                .background_color(BinaryColor::Off)
                .build(),
        }
    }
}

pub fn init_display(i2c: DisplayI2c<'static>) -> Option<&'static mut DisplayType> {
    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    // .into_terminal_mode();

    if display.init().is_err() {
        return None;
    }

    #[allow(static_mut_refs)]
    unsafe {
        // TODO: Release the old display before initializing the new one.
        DISPLAY.write(display);
        Some(DISPLAY.assume_init_mut())
    }
}

impl<DI, SIZE> RenderDisplay for Ssd1306<DI, SIZE, BufferedGraphicsMode<SIZE>>
where
    DI: WriteOnlyDataCommand,
    SIZE: DisplaySize,
    Self: DrawTarget<Color = BinaryColor>,
{
    fn render(&mut self, value: u16, button_state: ButtonEvent) -> Result<(), u16> {
        if self.clear(BinaryColor::Off).is_err() {
            return Err(100);
        }

        let small_style = TextStyle::Small.value();
        let small_top = small_style.font.character_size.height;

        let big_style = TextStyle::Large.value();
        let big_top = big_style.font.character_size.height;

        let _ = Text::new("Volume", Point::new(0, big_top as i32), big_style).draw(self);
        let _ = Text::new(
            &format!("value: {}", value),
            Point::new(0, (big_top + small_top * 2) as i32),
            small_style,
        )
        .draw(self);

        let is_pressed_text = if button_state.is_pressed {
            "DOWN"
        } else {
            "UP"
        };
        let _ = Text::new(
            &format!("Button: {}", is_pressed_text),
            Point::new(0, (big_top * 2 + small_top * 2) as i32),
            small_style,
        )
        .draw(self);

        let _ = Text::new(
            &format!("State: {:?}", button_state.event),
            Point::new(0, (big_top * 2 + small_top * 3) as i32),
            small_style,
        )
        .draw(self);

        if self.flush().is_err() {
            return Err(200);
        }

        Ok(())
    }
}

static mut LAST_VALUE: Option<u16> = None;

// ---- Terminal mode ----
impl<DI> RenderDisplay for Ssd1306<DI, DisplaySize128x64, TerminalMode>
where
    DI: WriteOnlyDataCommand,
{
    fn render(&mut self, value: u16, _button_state: ButtonEvent) -> Result<(), u16> {
        if unsafe { LAST_VALUE } == Some(value) {
            return Ok(());
        }
        unsafe {
            LAST_VALUE = Some(value);
        }

        let _ = self.clear();

        let digits = format!("volume: {}", value);
        info!("Volume: {}", value);

        // Write full string does not work, it only displays the last character.
        // let _ = self.write_str(&_digits);
        // Instead, we write each character individually. Works around the issue.
        for c in digits.as_bytes() {
            let bind = &[*c];
            let _ = self.write_str(unsafe { core::str::from_utf8_unchecked(bind) });
        }
        let _ = self.set_position(0, 2);

        for c in 33..123 {
            let bind = &[c];
            let _ = self.write_str(unsafe { core::str::from_utf8_unchecked(bind) });
        }

        Ok(())
    }
}
