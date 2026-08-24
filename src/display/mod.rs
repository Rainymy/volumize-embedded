#![allow(dead_code)]
mod driver;
mod percentage;
mod render;
mod screen;
pub mod style;
mod util;
mod widget;

pub mod text_style;

pub use percentage::*;
pub use render::*;
pub use screen::*;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use core::cell::RefCell;
use critical_section::Mutex;

use shared_types::{
    AppIdentifier, AudioApplication, AudioDevice, ChangeType, DeviceIdentifier, UpdateChange,
    protocol::{Envelope, Response},
};

type Refs<T> = Mutex<RefCell<T>>;
type Dict<V> = BTreeMap<DeviceIdentifier, V>;

pub static DEVICES_LIST: Refs<Vec<AudioDevice>> = Mutex::new(RefCell::new(Vec::new()));
pub static APPLICATIONS_LIST: Refs<Dict<Vec<AudioApplication>>> =
    Mutex::new(RefCell::new(BTreeMap::new()));
pub static APPLICATIONS_ID_LIST: Refs<Dict<Vec<AppIdentifier>>> =
    Mutex::new(RefCell::new(BTreeMap::new()));

#[allow(dead_code, unused_variables)]
pub async fn update_information(envelope: Envelope) {
    match envelope {
        // Ignore command, it's outgoing data only.
        Envelope::Command(_) => {}
        // Received response.
        Envelope::Response(response) => update_response(response),
        // Received event.
        Envelope::Event(update_change) => update_event(update_change),
    }
}

fn update_response(response: Response) {
    critical_section::with(|cs| match response {
        Response::Volume { .. } => {}
        Response::ApplicationList { device_id, apps } => {
            let mut current = APPLICATIONS_ID_LIST.borrow_ref_mut(cs);
            let _ = current.remove(&device_id);
            current.insert(device_id, apps);
        }
        Response::Application(application) => {
            let current = APPLICATIONS_ID_LIST.borrow_ref(cs);
            // Find the device ID associated with the application
            // Reverse lookup: value to key
            let item = current.iter().find(|entry| {
                entry
                    .1
                    .iter()
                    .find(|id| **id == application.process.id)
                    .is_some()
            });

            // ID must exist in APPLICATIONS_ID_LIST
            // Otherwise, the application is not associated with a device
            if let Some(entry) = item {
                let device_id = entry.0.clone();
                let mut current = APPLICATIONS_LIST.borrow_ref_mut(cs);
                let old_list = current.remove(&device_id).unwrap_or_default();

                let new_list = old_list
                    .into_iter()
                    .filter(|entry| entry.process.id != application.process.id)
                    .chain(core::iter::once(application.clone()))
                    .collect::<Vec<_>>();

                current.insert(device_id, new_list);
            }
        }
        Response::Icon { .. } => {}
        Response::DeviceList(device_list) => {
            DEVICES_LIST.replace_with(cs, |_old| device_list);
        }
        Response::Error { .. } => {}
    });
}

fn update_event(event: UpdateChange) {
    match event.change {
        ChangeType::AudioVolume { .. } => {}
        ChangeType::IconPathChange { .. } => {}
        ChangeType::NameChange { .. } => {}
        ChangeType::StateChange { .. } => {}
    }
}

pub async fn get_devices() -> Vec<AudioDevice> {
    Vec::new()
}
