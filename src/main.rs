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
    rtc_cntl,
    timer::timg::TimerGroup,
};

mod button_handling;
mod display;
mod init_display;
mod rotary;

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
    match usb_task(usb) {
        Ok(task) => spawner.spawn(task),
        Err(e) => defmt::panic!("Failed to spawn usb task: {:?}", e),
    };

    // Setup I2C communication.
    info!("Setup I2C communication");
    let i2c_config = I2cConfig::default();
    let i2c = defmt::expect!(I2c::new(peripherals.I2C0, i2c_config), "I2C Failed")
        .with_scl(peripherals.GPIO4)
        .with_sda(peripherals.GPIO5);

    // Initialize the display with I2C communication.
    info!("Initialize the display");
    let display = defmt::expect!(
        init_display::init_display(i2c),
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

        if let Err(delay) = render(display, value, button_state) {
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

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use esp_hal::otg_fs::Usb;

#[embassy_executor::task]
async fn usb_task(usb: Usb<'static>) {
    use embassy_usb::{
        Builder, Config as UsbConfig,
        class::cdc_acm::{CdcAcmClass, State},
    };
    use esp_hal::otg_fs::asynch::{Config as OtgConfig, Driver};
    use static_cell::StaticCell;

    info!("USB Task started");

    static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static STATE: StaticCell<State> = StaticCell::new();

    let mut config = UsbConfig::new(0x1209, 0x0001);
    config.manufacturer = Some("Volumize");
    config.product = Some("Volumize Hardware");
    config.serial_number = Some("VMZE:01");

    let mut builder = Builder::new(
        Driver::new(usb, EP_OUT_BUFFER.init([0u8; 256]), OtgConfig::default()),
        config,
        CONFIG_DESC.init([0u8; 256]),
        BOS_DESC.init([0u8; 256]),
        &mut [],
        CONTROL_BUF.init([0u8; 64]),
    );

    let mut class = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);
    let mut usb_device = builder.build();

    // let (mut rx, mut tx) = class.split();

    let usb_future = usb_device.run();
    let echo_future = async {
        loop {
            class.wait_connection().await;
            info!("USB connected");

            let raw_frame = match read_frame(&mut class).await {
                Ok(frame) => frame,
                Err(error) => {
                    info!("{}", error);
                    continue;
                }
            };

            info!("read {} bytes", raw_frame.len());

            if raw_frame.is_empty() {
                info!("Payload is empty");
                continue;
            }

            let payload = match decode_message(&raw_frame) {
                Ok(payload) => payload,
                Err(error) => {
                    info!(
                        "Decode error: [read bytes: {}] {}",
                        raw_frame.len(),
                        error.as_str()
                    );
                    continue;
                }
            };

            let hello = alloc::format!("{:?}", payload);
            info!("{}", hello.as_str());
            info!("USB disconnected");
        }
    };

    embassy_futures::join::join(usb_future, echo_future).await;

    info!("USB Task finished");
}

use shared_types::protocol::{Envelope, read_frame};

fn decode_message(payload: &Vec<u8>) -> Result<Envelope, String> {
    let data = payload.as_slice();
    ciborium::from_reader(data).map_err(|e| e.to_string())
}

#[allow(dead_code)]
fn encode_message(envelope: &Envelope) -> Vec<u8> {
    use shared_types::protocol::encode_frame;

    match encode_frame(envelope) {
        Ok(frame) => frame.build(),
        Err(err) => {
            defmt::warn!("Encode error: {}", err.as_str());
            Vec::new()
        }
    }
}
