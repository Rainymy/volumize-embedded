use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use defmt::info;
use esp_hal::otg_fs::{Usb, asynch::Driver};

use super::{IN_CHANNEL, OUT_CHANNEL};
use shared_types::protocol::{Envelope, read_frame};

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

    let class = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);
    let usb_device = builder.build();

    let (sender, receiver) = class.split();

    spawner.spawn(usb_sender_task(sender).unwrap());
    spawner.spawn(usb_receiver_task(receiver).unwrap());
    spawner.spawn(usb_run_task(usb_device).unwrap());

    #[embassy_executor::task]
    async fn usb_run_task(mut usb_device: UsbDevice<'static, Driver<'static>>) {
        usb_device.run().await;
    }
}

use embassy_usb::class::cdc_acm::{Receiver, Sender};

#[embassy_executor::task]
async fn usb_receiver_task(mut receiver: Receiver<'static, Driver<'static>>) {
    loop {
        receiver.wait_connection().await;

        let raw_frame = match read_frame(&mut receiver).await {
            Ok(raw_frame) => raw_frame,
            Err(err) => {
                info!("Error reading frame: {}", err);
                continue;
            }
        };

        let payload = match decode_message(&raw_frame) {
            Ok(payload) => payload,
            Err(error) => {
                info!("Decode error: {}", error.as_str());
                continue;
            }
        };

        let hello = alloc::format!("{:?}", payload);
        info!("{}", hello.as_str());

        IN_CHANNEL.send(payload).await;
    }
}

#[embassy_executor::task]
async fn usb_sender_task(mut class: Sender<'static, Driver<'static>>) {
    loop {
        class.wait_connection().await;
        let frame = encode_message(OUT_CHANNEL.receive().await);

        let mut chunks = frame.chunks(class.max_packet_size() as usize);
        while let Some(chunk) = chunks.next() {
            if let Err(err) = class.write_packet(&chunk).await {
                defmt::warn!("Write error: {}", err);
                break;
            }
        }
    }
}

fn decode_message(payload: &Vec<u8>) -> Result<Envelope, String> {
    let data = payload.as_slice();
    ciborium::from_reader(data).map_err(|e| e.to_string())
}

fn encode_message(envelope: Envelope) -> Vec<u8> {
    use shared_types::protocol::encode_frame;

    match encode_frame(&envelope) {
        Ok(frame) => frame.build(),
        Err(err) => {
            defmt::warn!("Encode error: {}", err.as_str());
            Vec::new()
        }
    }
}
