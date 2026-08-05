use alloc::format;
use defmt::info;
use embedded_graphics::{
    Drawable,
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{MonoTextStyleBuilder, ascii::FONT_5X8},
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

use super::RenderDisplay;

static mut DISPLAY: MaybeUninit<DisplayType> = MaybeUninit::zeroed();

pub fn init_display(i2c: DisplayI2c<'static>) -> Option<&'static mut DisplayType> {
    let interface = I2CDisplayInterface::new(i2c);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    // .into_terminal_mode();

    // Give the interface time to initialize.
    esp_hal::delay::Delay::new().delay_millis(20);

    if display.init().is_err() {
        return None;
    }

    // Give the display time to initialize.
    esp_hal::delay::Delay::new().delay_millis(20);

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
    fn render(&mut self, _value: u16, _button_pressed: bool) -> Result<(), u16> {
        if self.clear(BinaryColor::Off).is_err() {
            return Err(100);
        }

        // arduino_hal::delay_ms(50);

        let top = FONT_5X8.character_size.height;
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_5X8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        // use heapless::String;

        // let mut value_str: String<2> = String::new();
        // let _ = uwriteln!(value_str, "Volume: {}", _value);

        // let _ = _serial.write_str(&value_str);
        // let _ = _serial.write_str("\n");

        // This is making the display "crash". Not sure why. Anything after this line is not executed.
        // if Text::new(&value_str, Point::new(0, top as i32), style)
        //     .draw(self)
        //     .is_err()
        // {
        //     return Err(500);
        // }

        // This is for sanity check.
        if Text::new("volume 6", Point::new(0, top as i32), style)
            .draw(self)
            .is_err()
        {
            return Err(500);
        }

        // arduino_hal::delay_ms(50);

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
    fn render(&mut self, value: u16, _button_pressed: bool) -> Result<(), u16> {
        if unsafe { LAST_VALUE } == Some(value) {
            return Ok(());
        }
        unsafe {
            LAST_VALUE = Some(value);
        }

        let _ = self.clear();

        let _digits = format!("volume: {}", value);
        info!("[info] volume: {}", value);

        let _ = self.write_str(&_digits);
        let _ = self.set_position(2, 2);

        for c in 33..123 {
            let bind = &[c];
            let _ = self.write_str(unsafe { core::str::from_utf8_unchecked(bind) });
        }

        Ok(())
    }
}
