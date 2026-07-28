use core::{fmt::Write, mem::MaybeUninit};
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{ascii::FONT_5X8, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    text::Text,
    Drawable,
};
use ssd1306::{
    mode::{BufferedGraphicsMode, DisplayConfig, TerminalMode},
    prelude::{DisplayRotation, I2CInterface, WriteOnlyDataCommand},
    size::{DisplaySize, DisplaySize128x64},
    I2CDisplayInterface, Ssd1306,
};
use ufmt::{uWrite, uwrite};

use crate::{
    digits::{digit_to_str, number_to_vec},
    RenderDisplay,
};

pub type DisplayI2c = arduino_hal::I2c;
pub type DisplayInterface = I2CInterface<DisplayI2c>;

// pub type GraphicsMode = ssd1306::mode::BufferedGraphicsMode<DisplaySize128x64>;
pub type GraphicsMode = ssd1306::mode::TerminalMode;
pub type DisplayType = Ssd1306<DisplayInterface, DisplaySize128x64, GraphicsMode>;

static mut DISPLAY: MaybeUninit<DisplayType> = MaybeUninit::uninit();

pub fn init_display(i2c: DisplayI2c) -> Option<&'static mut DisplayType> {
    let interface = I2CDisplayInterface::new(i2c);
    arduino_hal::delay_ms(100);

    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        // .into_buffered_graphics_mode();
        .into_terminal_mode();

    // Give the interface time to initialize.
    arduino_hal::delay_ms(100);

    if display.init().is_err() {
        return None;
    }

    // Give the display time to initialize.
    arduino_hal::delay_ms(50);

    // This should change soon. ASAP
    // This is safe because only place it is getting modified is in "display::render"
    #[allow(static_mut_refs)]
    unsafe {
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
    fn render<W: uWrite>(
        &mut self,
        _serial: &mut W,
        _value: u32,
        _button_pressed: bool,
    ) -> Result<(), u16> {
        if self.clear(BinaryColor::Off).is_err() {
            return Err(100);
        }

        arduino_hal::delay_ms(50);

        let top = FONT_5X8.character_size.height + 12;
        let style = MonoTextStyleBuilder::new()
            .font(&FONT_5X8)
            .text_color(BinaryColor::On)
            .background_color(BinaryColor::Off)
            .build();

        use heapless::String;

        let mut value_str: String<2> = String::new();
        let _ = uwrite!(value_str, "Volume: {}", _value);

        let _ = _serial.write_str(&value_str);
        let _ = _serial.write_str("\n");

        // This is making the display "crash". Not sure why. Anything after this line is not executed.
        // if Text::new(&value_str, Point::new(0, top as i32), style)
        //     .draw(self)
        //     .is_err()
        // {
        //     return Err(500);
        // }

        // This is for sanity check.
        if Text::new("volume 6", Point::new(0, 2 * top as i32), style)
            .draw(self)
            .is_err()
        {
            return Err(500);
        }

        arduino_hal::delay_ms(50);
        if self.flush().is_err() {
            return Err(200);
        }

        Ok(())
    }
}

// ---- Terminal mode ----
impl<DI> RenderDisplay for Ssd1306<DI, DisplaySize128x64, TerminalMode>
where
    DI: WriteOnlyDataCommand,
{
    fn render<W: uWrite>(
        &mut self,
        _serial: &mut W,
        _value: u32,
        _button_pressed: bool,
    ) -> Result<(), u16> {
        use heapless::String;

        // use core::mem::MaybeUninit;
        // #[allow(invalid_value)]
        // let random_init = unsafe { MaybeUninit::<u8>::uninit().assume_init() };

        let _digits: String<4> = number_to_vec::<4, u8>(_value as u8)
            .into_iter()
            .map(digit_to_str)
            .collect();

        // let mut value_str: String<8> = String::new();
        // let _ = uwriteln!(value_str, "volume: {}", random_init + 48);
        // let _ = serial.write_str(&value_str);

        // let ascii_char = unsafe { ascii::Char::digit_unchecked(random_init) };
        // let _ = serial.write_char(ascii_char.to_char());

        {
            // let _ = serial.write_str("\n");
            // for d in digits {
            //     let _ = serial.write_char(d);
            //     let _ = serial.write_str("\n");
            // }
            // let _ = serial.write_str("--------");
        }
        // let _ = serial.write_str(&digits);
        // let _ = serial.write_str("\n");

        for c in 33..123 {
            let _ = self.write_str(unsafe { core::str::from_utf8_unchecked(&[c]) });
        }

        Ok(())
    }
}
