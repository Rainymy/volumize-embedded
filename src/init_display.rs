use embassy_time::Delay;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};
use lcd_async::{
    Builder,
    interface::SpiInterface,
    models::ILI9341Rgb565,
    options::{ColorOrder, Orientation, Rotation},
};
use ssd1306::{
    I2CDisplayInterface, Ssd1306Async,
    mode::{BufferedGraphicsModeAsync, DisplayConfigAsync},
    prelude::I2CInterface,
    rotation::DisplayRotation,
    size::DisplaySize128x64,
};

pub type DisplayI2c = esp_hal::i2c::master::I2c<'static, esp_hal::Async>;
pub type DisplayInterface = I2CInterface<DisplayI2c>;

pub type GraphicsMode = BufferedGraphicsModeAsync<DisplaySize128x64>;
// pub type GraphicsMode = TerminalMode;
pub type Ssd1306DisplayType = Ssd1306Async<DisplayInterface, DisplaySize128x64, GraphicsMode>;

#[allow(dead_code)]
static SSD1306_DISPLAY: StaticCell<Ssd1306DisplayType> = StaticCell::new();
#[allow(dead_code)]
static ILI9341_DISPLAY: StaticCell<Ili9341DisplayType> = StaticCell::new();

#[allow(dead_code)]
pub async fn init_display(i2c: DisplayI2c) -> Option<&'static mut Ssd1306DisplayType> {
    init_and_store(&SSD1306_DISPLAY, || async {
        let interface = I2CDisplayInterface::new(i2c);
        let mut display = Ssd1306Async::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
            .into_buffered_graphics_mode();
        display.init().await.ok()?;
        Some(display)
    })
    .await
}

pub type DisplaySPI = esp_hal::spi::master::Spi<'static, esp_hal::Async>;

pub type Ili9341DisplayType = lcd_async::Display<
    SpiInterface<ExclusiveDevice<DisplaySPI, Output<'static>, Delay>, Output<'static>>,
    ILI9341Rgb565,
    Output<'static>,
>;

#[allow(dead_code)]
pub async fn init_spi_display(
    spi: DisplaySPI,
    cs: AnyPin<'static>,
    dc: AnyPin<'static>,
    rst: AnyPin<'static>,
) -> Option<&'static mut Ili9341DisplayType> {
    init_and_store(&ILI9341_DISPLAY, || async move {
        let config = OutputConfig::default();
        let cs_output = Output::new(cs, Level::High, config);
        let dc_output = Output::new(dc, Level::Low, config);
        let rst_output = Output::new(rst, Level::High, config);

        let spi_device = ExclusiveDevice::new(spi, cs_output, Delay).ok()?;
        let interface = SpiInterface::new(spi_device, dc_output);

        Builder::new(ILI9341Rgb565, interface)
            .reset_pin(rst_output)
            .display_size(240, 320)
            .orientation(Orientation::new().rotate(Rotation::Deg90).flip_horizontal())
            .color_order(ColorOrder::Rgb)
            .init(&mut Delay)
            .await
            .ok()
    })
    .await
}

use core::future::Future;
use static_cell::StaticCell;

async fn init_and_store<T, F, Fut>(cell: &'static StaticCell<T>, init: F) -> Option<&'static mut T>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Option<T>>,
{
    defmt::info!("init_and_store: ");
    let value = init().await?;
    Some(cell.init(value))
}
