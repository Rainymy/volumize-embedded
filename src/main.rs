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
    interrupt::{InterruptHandler, Priority, software::SoftwareInterruptControl},
    otg_fs::Usb,
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

extern "C" fn rotary_handler() {
    rotary::update_encoder();
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 3 * 32 * 1024);

    // Create peripherals and configure CPU clock.
    let config = MCUConfig::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // Setup RTOS.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    // Setup rotary encoder.
    let dt_pin = peripherals.GPIO36.degrade();
    let clk_pin = peripherals.GPIO40.degrade();
    rotary::init_rotary(dt_pin, clk_pin);

    // Real Time Clock
    let rtc = rtc_cntl::Rtc::new(peripherals.LPWR);

    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    match usb_task(usb) {
        Ok(task) => spawner.spawn(task),
        Err(e) => {
            panic!("Failed to spawn usb task: {:?}", e);
        }
    };
    info!("Embassy initialized!");

    // Enable GPIO interrupts.
    {
        use esp_hal::{interrupt, peripherals as phrs};
        let rotary_handler = InterruptHandler::new(rotary_handler, Priority::min());
        interrupt::bind_handler(phrs::Interrupt::GPIO, rotary_handler);
        // interrupt::enable(phrs::Interrupt::GPIO, Priority::min());
    }

    // Setup I2C communication.
    let i2c_config = I2cConfig::default();
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("I2C Failed")
        .with_scl(peripherals.GPIO4)
        .with_sda(peripherals.GPIO5);

    // Initialize the display with I2C communication.
    let display = defmt::expect!(
        init_display::init_display(i2c),
        "Display not initialized or Not Connected"
    );

    let mut button_tracker = ButtonTracker::new();
    let mut last_button_state = button_tracker.poll(false, 0);
    let (rx, _tx) = unsafe { peripherals.GPIO35.split() };

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
            last_button_state = button_state.clone();
        }

        if let Err(delay) = render(display, value.into(), button_state) {
            info!("render error from render: {}", delay);
        }

        embassy_time::Timer::after_millis(20).await;
    }
}

#[embassy_executor::task]
async fn usb_task(usb: Usb<'static>) {
    use embassy_usb::{
        Builder, Config as UsbConfig,
        class::cdc_acm::{CdcAcmClass, State},
    };
    use esp_hal::otg_fs::asynch::{Config as OtgConfig, Driver};
    use static_cell::StaticCell;

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

    let usb_future = usb_device.run();
    let echo_future = async {
        loop {
            class.wait_connection().await;
            info!("USB connected");

            let mut buf = [0u8; 64];
            loop {
                match class.read_packet(&mut buf).await {
                    Ok(count) => {
                        info!("read {} bytes", count);
                        if count > 0 {
                            let data = &buf[..count];
                            info!("data: {}", data);
                        }
                    }
                    Err(error) => {
                        info!("read error: {}", error);
                        break;
                    }
                }
            }
            info!("USB disconnected");
        }
    };

    embassy_futures::join::join(usb_future, echo_future).await;
}
