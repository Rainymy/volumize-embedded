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
    interrupt::{InterruptHandler, software::SoftwareInterruptControl},
    otg_fs::Usb,
    rtc_cntl,
    timer::timg::TimerGroup,
};

mod button_handling;
mod display;
mod init_display;
mod rotary;
mod usb;

use button_handling::ButtonTracker;
use display::render;

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

    // Setup rotary encoder.
    let dt_pin = peripherals.GPIO36.degrade();
    let clk_pin = peripherals.GPIO40.degrade();
    rotary::init_rotary(dt_pin, clk_pin);
    info!("Rotary encoder initialized!");

    // Real Time Clock
    let rtc = rtc_cntl::Rtc::new(peripherals.LPWR);

    // Enable GPIO interrupts.
    enable_gpio_interrupts();

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

    let mut button_tracker = ButtonTracker::new();
    let mut last_button_state = button_tracker.poll(false, 0);
    let (rx, _tx) = unsafe { peripherals.GPIO35.split() };

    info!("Entering main loop");
    loop {
        let now = rtc.current_time_us();
        let button_pressed = !rx.is_input_high();
        let value = rotary::read_rotation_value() / 100.0;

        let button_state = button_tracker.poll(button_pressed, now);
        if button_state.event != last_button_state.event {
            info!(
                "Button pressed: {} - event: {:?}",
                button_state.is_pressed, button_state.event
            );
            // info!("{}", esp_alloc::HEAP.stats());
            last_button_state = button_state.clone();
        }

        if let Err(delay) = render(display, value, button_state).await {
            info!("render error from render: {}", delay);
        }

        embassy_time::Timer::after_millis(20).await;
    }
}

// Enable GPIO interrupts.
fn enable_gpio_interrupts() {
    use esp_hal::{interrupt, interrupt::Priority, peripherals::Interrupt};
    use rotary::update_encoder;

    interrupt::bind_handler(
        Interrupt::GPIO,
        InterruptHandler::new(update_encoder, Priority::min()),
    );
}
