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

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_hal::{
    Config as MCUConfig,
    clock::CpuClock,
    gpio::Pin,
    interrupt::software::SoftwareInterruptControl,
    otg_fs::Usb,
    spi::master::{Config as SpiConfig, Spi},
    time::{Instant, Rate},
    timer::timg::TimerGroup,
};

mod display;
mod init_display;
mod input;
mod usb;

pub use input::{ButtonTracker, InputEvent, RotaryTracker, RotationEvent};
use shared_types::protocol::Envelope;

#[allow(unused)]
use display::{
    Screen,
    adjust_volume::{RenderApplication, VolumeAdjustState},
    application_menu::ApplicationMenuState,
    wait_for_data::WaitForDataState,
};

use crate::display::store::{dummy, get_applications, update_information};

pub static OUT_CHANNEL: Channel<CriticalSectionRawMutex, Envelope, 16> = Channel::new();
pub static IN_CHANNEL: Channel<CriticalSectionRawMutex, Envelope, 16> = Channel::new();

esp_bootloader_esp_idf::esp_app_desc!();

esp_hal::assign_resources! {
    Resources<'d> {
        spi: DisplaySpi<'d> {
             cs: GPIO17,
             rst: GPIO16,
             dc: GPIO15,
             mosi: GPIO7,
             clk: GPIO6,
             miso: GPIO5,
        },
        encoder: Encoder<'d> {
            dt: GPIO2,
            clk: GPIO1,
            sw: GPIO42
        },
    }
}

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 3 * 32 * 1024);
    info!("Embassy initialized!");

    // Create peripherals and configure CPU clock.
    let config = MCUConfig::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let resource = split_resources!(peripherals);
    let encoder = resource.encoder;
    let display_spi = resource.spi;
    info!("CPU clock configured!");

    // Setup RTOS.
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    info!("RTOS scheduler started!");

    // Initialize interrupt handlers.
    input::init_rotary_interrupt(
        peripherals.PCNT,
        encoder.dt.degrade(),
        encoder.clk.degrade(),
    );
    input::init_button_interrupt(encoder.sw.degrade());
    input::enable_gpio_interrupts();
    info!("Interrupt handlers initialized!");

    // USB CDC-ACM - Serial over USB
    let usb = Usb::new(peripherals.USB0, peripherals.GPIO20, peripherals.GPIO19);
    match usb::usb_task(usb, spawner) {
        Ok(task) => spawner.spawn(task),
        Err(e) => defmt::panic!("Failed to spawn usb task: {:?}", e),
    };

    // Setup SPI communication.
    let spi_config = SpiConfig::default().with_frequency(Rate::from_mhz(20));
    let spi = defmt::expect!(Spi::new(peripherals.SPI2, spi_config))
        .with_sck(display_spi.clk)
        .with_mosi(display_spi.mosi)
        .with_miso(display_spi.miso)
        .into_async();

    let display = defmt::expect!(
        init_display::init_spi_display(
            spi,
            display_spi.cs.degrade(),
            display_spi.dc.degrade(),
            display_spi.rst.degrade(),
        )
        .await,
        "Display SPI not initialized or Not Connected"
    );

    let mut button_tracker = ButtonTracker::default();
    let mut rotary_tracker = RotaryTracker::default();

    // Populate dummy data to simulate applications.
    dummy::populate_dummy_data().await;

    let application = get_applications(None).await;
    let state = ApplicationMenuState::new(application.len(), None);

    let root_screen = Screen::ApplicationList(state);
    let mut ui_state = display::UIState::new(root_screen);

    // ui_state.push(Screen::WaitingForData(WaitForDataState::default()));
    let in_receiver = IN_CHANNEL.receiver();

    info!("Entering main loop");
    loop {
        if let Ok(envelope) = in_receiver.try_receive() {
            update_information(envelope);
        };

        let value = input::read_rotation_value();
        if let Some(event) = rotary_tracker.poll(value as i16) {
            info!("Rotary event: {}", event);
            display::handle_event(&mut ui_state, event).await;
        }

        {
            // These 2 calls work together to handle button edge and timeout events.
            input::with_edge_queue(async |is_down, timestamp| {
                if let Some(event) = button_tracker.on_edge(is_down, timestamp) {
                    info!("Edge event: {}", event);
                    display::handle_event(&mut ui_state, event).await;
                }
            })
            .await;

            let now_ms = Instant::now().duration_since_epoch().as_millis();
            if let Some(button_state) = button_tracker.check_timeouts(now_ms) {
                info!("Timeout event: {}", button_state);
                display::handle_event(&mut ui_state, button_state).await;
            }
        }

        if let Err(delay) = display::render(display, ui_state.current_mut()).await {
            info!("[display::render] render error from render: {}", delay);
        }
    }
}
