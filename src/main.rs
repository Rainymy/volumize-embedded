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

use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use esp_hal::{
    Config as MCUConfig,
    clock::CpuClock,
    gpio::Pin,
    interrupt::software::SoftwareInterruptControl,
    otg_fs::Usb,
    spi::master::{Config as SpiConfig, Spi},
    time::Rate,
    timer::timg::TimerGroup,
};

mod display;
mod init_display;
mod input;
mod usb;

use input::{ButtonTracker, RotaryTracker};
pub use input::{InputEvent, RotationEvent};
use shared_types::protocol::Envelope;

use display::store::{self, dummy, get_applications, update_information};
use display::{
    Screen, UIState, application_menu::ApplicationMenuState, wait_for_data::WaitForDataState,
};
use init_display::Ili9341DisplayType;

pub static OUT_CHANNEL: Channel<CriticalSectionRawMutex, Envelope, 16> = Channel::new();
pub static IN_CHANNEL: Channel<CriticalSectionRawMutex, Envelope, 16> = Channel::new();

esp_bootloader_esp_idf::esp_app_desc!();

esp_hal::assign_resources! {
    Resources<'d> {
        d_spi: DisplaySpi<'d> {
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
async fn main(spawner: Spawner) -> ! {
    esp_alloc::heap_allocator!(size: 3 * 32 * 1024);
    info!("Embassy initialized!");

    let display = init_hardware(spawner).await;

    // Populate dummy data to simulate applications.
    dummy::populate_dummy_data().await;

    let mut ui_state = build_initial_ui().await;
    let mut inputs = InputPoller::default();
    let in_receiver = IN_CHANNEL.receiver();

    info!("Entering main loop");
    loop {
        while let Ok(envelope) = in_receiver.try_receive() {
            update_information(envelope);
        }

        inputs.dispatch(&mut ui_state).await;

        if let Err(code) = display::render(display, ui_state.current_mut()).await {
            info!("[display::render] render error: {}", code);
        }
    }
}

async fn init_hardware(spawner: Spawner) -> &'static mut Ili9341DisplayType {
    let config = MCUConfig::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let Resources { d_spi, encoder } = split_resources!(peripherals);
    info!("CPU clock configured!");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);
    info!("RTOS scheduler started!");

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
    spawner.spawn(defmt::unwrap!(usb::usb_task(usb, spawner)));

    let spi_config = SpiConfig::default().with_frequency(Rate::from_mhz(20));
    let spi = defmt::expect!(Spi::new(peripherals.SPI2, spi_config))
        .with_sck(d_spi.clk)
        .with_mosi(d_spi.mosi)
        .with_miso(d_spi.miso)
        .into_async();

    defmt::expect!(
        init_display::init_spi_display(
            spi,
            d_spi.cs.degrade(),
            d_spi.dc.degrade(),
            d_spi.rst.degrade(),
        )
        .await,
        "Display SPI not initialized or Not Connected"
    )
}

async fn build_initial_ui() -> UIState {
    let applications = get_applications(None).await;
    let root = Screen::ApplicationList(ApplicationMenuState::new(applications.len(), None));
    let mut ui_state = UIState::new(root);

    if store::is_waiting_for_data() {
        ui_state.push(Screen::WaitingForData(WaitForDataState::default()));
    }

    ui_state
}

#[derive(Default)]
struct InputPoller {
    button: ButtonTracker,
    rotary: RotaryTracker,
}

impl InputPoller {
    async fn dispatch(&mut self, ui_state: &mut UIState) {
        if let Some(event) = self.rotary.poll(input::read_rotation_value()) {
            info!("Rotary event: {}", event);
            display::handle_event(ui_state, event).await;
        }

        for (is_down, timestamp) in input::take_edges() {
            if let Some(event) = self.button.on_edge(is_down, timestamp) {
                info!("Edge event: {}", event);
                display::handle_event(ui_state, event).await;
            }
        }

        if let Some(event) = self.button.check_timeouts(input::now_ms()) {
            info!("Timeout event: {}", event);
            display::handle_event(ui_state, event).await;
        }
    }
}
