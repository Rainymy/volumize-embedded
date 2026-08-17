use core::mem::MaybeUninit;
use ssd1306::{
    I2CDisplayInterface, Ssd1306,
    mode::{BufferedGraphicsMode, DisplayConfig},
    prelude::{DisplayRotation, I2CInterface},
    size::DisplaySize128x64,
};

pub type DisplayI2c = esp_hal::i2c::master::I2c<'static, esp_hal::Blocking>;
pub type DisplayInterface = I2CInterface<DisplayI2c>;

pub type GraphicsMode = BufferedGraphicsMode<DisplaySize128x64>;
// pub type GraphicsMode = TerminalMode;
pub type DisplayType = Ssd1306<DisplayInterface, DisplaySize128x64, GraphicsMode>;

static mut DISPLAY: MaybeUninit<DisplayType> = MaybeUninit::zeroed();

pub fn init_display(i2c: DisplayI2c) -> Option<&'static mut DisplayType> {
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
