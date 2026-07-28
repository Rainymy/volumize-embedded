#![feature(abi_avr_interrupt)]
#![no_std]
#![no_main]

use arduino_hal as hal;

mod debug;
mod display;
mod ssd1306_impl;

mod button_handling;
mod millis;

// Perhaps this could be in lib.rs
pub use display::RenderDisplay;
#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    millis::tick()
}

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let mut serial = hal::default_serial!(dp, pins, 57600);

    // --- display.begin(SSD1306_SWITCHCAPVCC, 0x3C) ---
    dp.TC0.tccr0a().write(|w| w.wgm0().ctc());
    dp.TC0.ocr0a().write(|w| unsafe { w.bits(249) });
    dp.TC0.tccr0b().write(|w| w.cs0().prescale_64());
    dp.TC0.timsk0().write(|w| w.ocie0a().set_bit());

    unsafe {
        avr_device::interrupt::enable();
    }

    let i2c = hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50_000,
    );

    let interface = I2CDisplayInterface::new(i2c);
    let mut display = Ssd1306::new(
            interface,
            DisplaySize128x64,
            DisplayRotation::Rotate0
        )
        .into_buffered_graphics_mode();
        // .into_terminal_mode();

    let debug_blink = move |delay_ms: u16, count: u16| -> ! {
        debug::blink_forever(pins.d13.into_output(), delay_ms, count);
    };

    // Give the interface time to initialize.
    hal::delay_ms(50);

    if display.init().is_err() {
        debug_blink(100, 2);
    }

    // Give the display time to initialize.
    hal::delay_ms(50);

    // let mut button_tracker = button_handling::ButtonTracker::new();
    // let button_pressed = pins.a3.is_low();

    // button_tracker.update(button_pressed, 0);

    loop {
        if let Err(delay) = display::render(&mut display, &mut serial, 0, false) {
            debug_blink(delay, 4);
        }
        hal::delay_ms(50);
    }
}