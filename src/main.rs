#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

use esp_hal::interrupt::software::SoftwareInterruptControl;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::{clock::CpuClock, rtc_cntl};

use embassy_executor::Spawner;
// use embassy_time::{Duration, Timer};

use defmt::info;
use esp_println as _;

use esp_backtrace as _;

mod button_handling;
mod display;
mod helper;
mod rotary;

use display::{init_display, render};

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(dead_code)]
#[esp_hal::handler]
fn rotary_handler() {
    rotary::update_encoder();
}

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

    // Setting up interrupt GPIO pins.
    use esp_hal::gpio::Pin;
    let dt_pin = peripherals.GPIO16.degrade();
    let clk_pin = peripherals.GPIO17.degrade();
    rotary::init_rotary(dt_pin, clk_pin);

    // Real Time Clock
    let rtc = rtc_cntl::Rtc::new(peripherals.LPWR);
    info!("Embassy initialized!");

    use esp_hal::interrupt;
    let gpio = esp_hal::peripherals::Interrupt::GPIO;
    interrupt::enable(gpio, interrupt::Priority::min());

    use esp_hal::i2c::master::{Config as I2cConfig, I2c};

    let i2c_config = I2cConfig::default();
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("I2C Failed")
        .with_scl(peripherals.GPIO0)
        .with_sda(peripherals.GPIO1);

    // use esp_hal::interrupt::{InterruptHandler, Priority, bind_handler};
    // let handler = InterruptHandler::new(|| rotary::handle_rotary_interrupt(), Priority::min());
    // bind_handler(gpio, handler);

    let display = init_display(i2c).expect("Display failed to initialize!!");

    let mut button_tracker = button_handling::ButtonTracker::new();
    let (rx, _tx) = unsafe { peripherals.GPIO13.split() };

    loop {
        let now = rtc.current_time_us();
        let button_pressed = !rx.is_input_high();

        let (button_pressed, _button_event) = button_tracker.poll(button_pressed, now);
        let value = rotary::read_rotation_value() as u16;

        if let Err(delay) = render(display, value, button_pressed) {
            info!("render error from render: {}", delay);
        }
    }
}
