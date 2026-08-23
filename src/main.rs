#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

use defmt::info;
use esp_backtrace as _;
use esp_println as _;

use esp_hal::{
    Config as MCUConfig,
    clock::CpuClock,
    gpio::Pin,
    i2c::master::{Config as I2cConfig, I2c},
    interrupt::software::SoftwareInterruptControl,
    otg_fs::Usb,
    time::Instant,
    timer::timg::TimerGroup,
};

mod display;
mod init_display;
mod input;
mod interrupt_handler;
// mod navigation;
mod screens;
mod usb;

use display::render;
pub use input::{ButtonTracker, InputEvent, RotaryTracker, RotationEvent};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 3 * 32 * 1024);

    info!("Embassy initialized!");

    // Create peripherals and configure CPU clock.
    let config = MCUConfig::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    info!("CPU clock configured!");

    // Setup RTOS.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    info!("RTOS scheduler started!");

    // Setup interrupt pins.
    let dt_pin = peripherals.GPIO36.degrade();
    let clk_pin = peripherals.GPIO40.degrade();
    let btn_pin = peripherals.GPIO35.degrade();

    // Initialize interrupt handlers.
    interrupt_handler::init_rotary_interrupt(dt_pin, clk_pin);
    interrupt_handler::init_button_interrupt(btn_pin);
    interrupt_handler::enable_gpio_interrupts();
    info!("Interrupt handlers initialized!");

    // USB CDC-ACM - Serial over USB
    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    match usb::usb_task(usb, spawner) {
        Ok(task) => spawner.spawn(task),
        Err(e) => defmt::panic!("Failed to spawn usb task: {:?}", e),
    };

    // Setup I2C communication.
    info!("Setup I2C communication");
    let i2c_config = I2cConfig::default();
    let i2c = defmt::expect!(I2c::new(peripherals.I2C0, i2c_config), "I2C Failed")
        .with_scl(peripherals.GPIO4)
        .with_sda(peripherals.GPIO5)
        .into_async();

    // Initialize the display with I2C communication.
    info!("Initialize the display");
    let display = defmt::expect!(
        init_display::init_display(i2c).await,
        "Display not initialized or Not Connected"
    );

    let mut button_tracker = ButtonTracker::default();
    let mut rotary_tracker = RotaryTracker::default();

    let mut ui_state = screens::UIState::new();

    info!("Entering main loop");
    loop {
        let value = interrupt_handler::read_rotation_value();
        if let Some(event) = rotary_tracker.poll(value as i16) {
            info!("Rotary event: {}", event);
            screens::handle_event(&mut ui_state, event);
        }

        interrupt_handler::with_edge_queue(|is_down, timestamp| {
            if let Some(event) = button_tracker.on_edge(is_down, timestamp) {
                info!("Edge event: {}", event);
                screens::handle_event(&mut ui_state, event);
            }
        });

        let now_ms = Instant::now().duration_since_epoch().as_millis();
        if let Some(button_state) = button_tracker.check_timeouts(now_ms) {
            info!("Timeout event: {}", button_state);
            screens::handle_event(&mut ui_state, button_state);
        }

        let current_screen: screens::Screen = ui_state.current().clone();
        if let Err(delay) = render(display, current_screen).await {
            info!("render error from render: {}", delay);
        }
    }
}
