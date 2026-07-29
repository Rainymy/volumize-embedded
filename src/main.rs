#![feature(abi_avr_interrupt)]
#![no_std]
#![no_main]

use arduino_hal as hal;

mod debug;
mod display;
mod ssd1306_impl;

mod button_handling;
mod millis;

use ssd1306_impl::init_display;

// Perhaps this could be in lib.rs
pub use display::RenderDisplay;
pub mod digits;

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    millis::tick()
}

#[avr_device::interrupt(atmega328p)]
fn INT0() {
    rotary::update_encoder();
}

#[avr_device::interrupt(atmega328p)]
fn INT1() {
    rotary::update_encoder();
}

// TODO:
//  - rotary encoder/button

#[hal::entry]
fn main() -> ! {
    let dp = hal::Peripherals::take().unwrap();
    let pins = hal::pins!(dp);

    let mut serial = hal::default_serial!(dp, pins, 57600);

    dp.TC0.tccr0a().write(|w| w.wgm0().ctc());
    dp.TC0.ocr0a().write(|w| unsafe { w.bits(249) });
    dp.TC0.tccr0b().write(|w| w.cs0().prescale_64());
    dp.TC0.timsk0().write(|w| w.ocie0a().set_bit());

    // ========== External Interrupts ========
    dp.EXINT.eicra().write(|w| {
        w.isc0().val_0x01();
        w.isc1().val_0x01();
        w
    });

    dp.EXINT
        .eimsk()
        .write(|w| w.int0().set_bit().int1().set_bit());
    // =======================================

    let _clk = pins.d2.into_pull_up_input();
    let _dt = pins.d3.into_pull_up_input();
    let digital_d4 = pins.d4.into_floating_input();

    rotary::init_rotary(_dt, _clk);
    unsafe {
        avr_device::interrupt::enable();
    }

    let i2c = hal::I2c::new(
        dp.TWI,
        pins.a4.into_pull_up_input(),
        pins.a5.into_pull_up_input(),
        50_000,
    );

    let debug_blink = move |delay_ms: u16, count: u16| -> ! {
        debug::blink_forever(pins.d13.into_output(), delay_ms, count);
    };

    let display = match init_display(i2c) {
        Some(d) => d,
        None => debug_blink(100, 2),
    };

    hal::delay_ms(100);

    let mut button_tracker = button_handling::ButtonTracker::new();
    let mut value = 0u32;

    let digital_d4 = pins.d4.into_floating_input();

    loop {
        let button_pressed = digital_d4.is_low();
        let now = millis::millis();

        let (button_pressed, _button_event) = button_tracker.update(button_pressed, now);

        if let Err(delay) = display::render(display, &mut serial, value, button_pressed) {
            debug_blink(delay, 4);
        }
        hal::delay_ms(50);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Steal the peripherals — we don't care about soundness here,
    // we're crashing anyway, we just want to get a message out.
    let dp = unsafe { hal::Peripherals::steal() };
    let pins = hal::pins!(dp);
    let mut serial = hal::default_serial!(dp, pins, 57600);

    let _ = ufmt::uwriteln!(&mut serial, "PANIC!");

    if let Some(_location) = _info.location() {
        // let _ = ufmt::uwriteln!(&mut serial, "at {}", _location.line());
    }

    loop {}
}
