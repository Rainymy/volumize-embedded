#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

use esp_hal::clock::CpuClock;
use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;

use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};

use defmt::info;
use esp_println as _;

use esp_backtrace as _;

mod button_handling;
mod display;
mod helper;
// mod rotary;

use display::{init_display, render};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// #[allow(
//     clippy::large_stack_frames,
//     reason = "it's not unusual to allocate larger buffers etc. in main"
// )]
#[esp_rtos::main]
async fn main(_spawner: Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 3 * 32 * 1024);

    // Create peripherals and configure CPU clock.
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Setup timers and software interrupts.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");
    // rotary::init_rotary(_dt, _clk);

    use esp_hal::i2c::master::{Config as I2cConfig, I2c};

    let i2c_config = I2cConfig::default();
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("I2C Failed")
        .with_scl(peripherals.GPIO0)
        .with_sda(peripherals.GPIO1);

    let debug_blink = move |_delay_ms: u16, _count: u16| -> ! {
        loop {}
        // debug::blink_forever(peripherals.GPIO13, delay_ms, count);
    };

    let display = match init_display(i2c) {
        Some(d) => d,
        None => debug_blink(100, 2),
    };

    let mut button_tracker = button_handling::ButtonTracker::new();
    let value = 0;
    let (rx, _tx) = unsafe { peripherals.GPIO13.split() };

    loop {
        // let now = millis::millis();
        let button_pressed = !rx.is_input_high();
        let now = 0;

        let (button_pressed, _button_event) = button_tracker.update(button_pressed, now);

        // let value = rotary::read_rotation_value() as u32;
        // let _ = ufmt::uwriteln!(serial, "value: {}", value);

        if let Err(delay) = render(display, value, button_pressed) {
            debug_blink(delay, 4);
        }
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
