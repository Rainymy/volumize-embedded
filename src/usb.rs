use defmt::info;
use embassy_futures::select::{Either, select};
use esp_hal::otg_fs::{Usb, asynch::Driver as OtgDriver};

use super::{IN_CHANNEL, OUT_CHANNEL};
use shared_types::{info, protocol::RawFrame, reader::read_frame};

use embassy_usb::{
    class::cdc_acm::{Receiver, Sender},
    driver::EndpointError,
};

#[embassy_executor::task]
pub async fn usb_task(usb: Usb<'static>, spawner: embassy_executor::Spawner) {
    use embassy_usb::{
        Builder, Config as UsbConfig, UsbDevice,
        class::cdc_acm::{CdcAcmClass, State},
    };
    use esp_hal::otg_fs::asynch::Config as OtgConfig;
    use static_cell::StaticCell;

    info!("USB Task started");

    static EP_OUT_BUFFER: StaticCell<[u8; 256]> = StaticCell::new();
    static CONFIG_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static BOS_DESC: StaticCell<[u8; 256]> = StaticCell::new();
    static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    static STATE: StaticCell<State> = StaticCell::new();

    let mut config = UsbConfig::new(0x1209, 0x0001);
    config.manufacturer = Some("Volumize");
    config.product = Some(info::PRODUCT);
    config.serial_number = Some(info::SERIAL_NUMBER);

    let mut builder = Builder::new(
        OtgDriver::new(usb, EP_OUT_BUFFER.init([0u8; 256]), OtgConfig::default()),
        config,
        CONFIG_DESC.init([0u8; 256]),
        BOS_DESC.init([0u8; 256]),
        &mut [],
        CONTROL_BUF.init([0u8; 64]),
    );

    let class = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);
    let usb_device = builder.build();

    let (sender, receiver) = class.split();

    spawner.spawn(usb_sender_task(sender).unwrap());
    spawner.spawn(usb_receiver_task(receiver).unwrap());
    spawner.spawn(usb_run_task(usb_device).unwrap());

    #[embassy_executor::task]
    async fn usb_run_task(mut usb_device: UsbDevice<'static, OtgDriver<'static>>) {
        usb_device.run().await;
    }
}

#[embassy_executor::task]
async fn usb_receiver_task(mut receiver: Receiver<'static, OtgDriver<'static>>) {
    loop {
        receiver.wait_connection().await;

        let raw_frame = match read_frame(&mut receiver).await {
            Ok(raw_frame) => raw_frame,
            Err(err) => {
                info!("Error reading frame: {}", err);
                continue;
            }
        };

        let payload = match RawFrame::decode(&raw_frame) {
            Ok(payload) => payload,
            Err(error) => {
                info!("Decode error: {}", error);
                continue;
            }
        };

        info!("{}", alloc::format!("{:?}", payload));
        IN_CHANNEL.send(payload).await;
    }
}

#[embassy_executor::task]
async fn usb_sender_task(mut class: Sender<'static, OtgDriver<'static>>) {
    loop {
        // Disconnected: throw away anything the app produces until the host is back.
        let drain = async {
            loop {
                let _ = OUT_CHANNEL.receive().await;
            }
        };

        match select(class.wait_connection(), drain).await {
            Either::First(()) => {}         // connected
            Either::Second(never) => never, // drain never completes
        }

        defmt::info!("USB connected");

        loop {
            let envelope = OUT_CHANNEL.receive().await;
            let frame = RawFrame::encode(&envelope).build();

            match send_frame(&mut class, &frame).await {
                Ok(_) => {}
                Err(EndpointError::Disabled) => {
                    defmt::info!("USB disconnected");
                    break;
                }
                Err(EndpointError::BufferOverflow) => {
                    // This should never happen unless `max_packet_size`
                    // is larger than serial buffer size.
                    defmt::warn!("[USB buffer overflow]: Should not happen");
                }
            }
        }
    }
}

async fn send_frame(
    class: &mut Sender<'static, OtgDriver<'static>>,
    frame: &[u8],
) -> Result<(), EndpointError> {
    let mps = class.max_packet_size() as usize;

    for chunk in frame.chunks(mps) {
        class.write_packet(chunk).await?;
    }

    // USB bulk transfers signal completion via a short packet (len < max_packet_size).
    // If our last chunk happened to be exactly max_packet_size, the host can't tell
    // the transfer ended, so we send a ZLP (zero-length packet) to force closure.
    if frame.len() % mps == 0 {
        class.write_packet(&[]).await?;
    }

    Ok(())
}
