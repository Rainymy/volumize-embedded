mod driver;
mod percentage;
mod render;
mod screen;
pub mod style;
mod util;
mod widget;

pub mod text_style;

use alloc::format;
pub use percentage::*;
pub use render::*;
pub use screen::*;

use alloc::vec::Vec;
use alloc::{collections::BTreeMap, string::String};
use shared_types::{AudioVolume, ProcessInfo, SessionDirection, SessionType};

use core::cell::RefCell;
use critical_section::Mutex;

use shared_types::{
    AppIdentifier, AudioApplication, AudioDevice, ChangeType, DeviceIdentifier, Identifier,
    UpdateChange,
    protocol::{Envelope, Response},
};

type Refs<T> = Mutex<RefCell<T>>;
type Dict<V> = BTreeMap<DeviceIdentifier, V>;

pub static DEVICES_LIST: Refs<Vec<AudioDevice>> = Mutex::new(RefCell::new(Vec::new()));
pub static APPLICATIONS_LIST: Refs<Dict<Vec<AudioApplication>>> =
    Mutex::new(RefCell::new(BTreeMap::new()));
pub static APPLICATIONS_ID_LIST: Refs<Dict<Vec<AppIdentifier>>> =
    Mutex::new(RefCell::new(BTreeMap::new()));

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

fn find_application_device(app_id: AppIdentifier) -> Option<String> {
    critical_section::with(|cs| {
        let current = APPLICATIONS_ID_LIST.borrow_ref_mut(cs);
        let item = current
            .iter()
            .find(|entry| entry.1.iter().find(|id| **id == app_id).is_some());

        if let Some(item) = item {
            Some(item.0.clone())
        } else {
            None
        }
    })
}

fn update_response(response: Response) {
    critical_section::with(|cs| match response {
        Response::Volume { id, volume } => match id {
            Identifier::App(app_id) => {
                let device_id = find_application_device(app_id).unwrap_or_default();
                APPLICATIONS_LIST
                    .borrow_ref_mut(cs)
                    .iter_mut()
                    .find(|item| item.0 == &device_id)
                    .map(|item| item.1.iter_mut().find(|app| app.process.id == app_id))
                    .map(|app| {
                        if let Some(app) = app {
                            app.volume = volume;
                        }
                    });
            }
            Identifier::Device(device_id) => {
                DEVICES_LIST
                    .borrow_ref_mut(cs)
                    .iter_mut()
                    .find(|item| item.id == device_id)
                    .map(|item| item.volume = volume);
            }
        },
        Response::ApplicationList { device_id, apps } => {
            let mut current = APPLICATIONS_ID_LIST.borrow_ref_mut(cs);
            let _ = current.remove(&device_id);
            current.insert(device_id, apps);
        }
        Response::Application(application) => {
            if let Some(device_id) = find_application_device(application.process.id) {
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
        Response::Icon { app_id, data } => {
            let device_id = find_application_device(app_id).unwrap_or_default();

            let mut current = APPLICATIONS_LIST.borrow_ref_mut(cs);
            let current = current
                .iter_mut()
                .find(|item| item.0 == &device_id)
                .map(|item| item.1.iter_mut().find(|app| app.process.id == app_id))
                .flatten();

            if let Some(app) = current {
                app.process.path = Some(data);
            }
        }
        Response::DeviceList(device_list) => {
            DEVICES_LIST.replace_with(cs, |_old| device_list);
        }
        Response::Error { message, request } => {
            let text = format!("Error: {} (request: {:?})", message.as_str(), request);
            defmt::error!("{}", text.as_str());
        }
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

pub async fn get_applications(device_id: Option<DeviceIdentifier>) -> Vec<AudioApplication> {
    critical_section::with(|cs| {
        let device_id = match device_id {
            Some(id) => id,
            None => DEVICES_LIST
                .borrow_ref(cs)
                .iter()
                .find(|item| item.is_default)
                .map(|item| item.id.clone())
                .unwrap_or_default(),
        };

        defmt::dbg!("{}", device_id.as_str());

        APPLICATIONS_LIST
            .borrow_ref(cs)
            .get(&device_id)
            .cloned()
            .unwrap_or_default()
    })
}

pub async fn get_default_device() -> Option<AudioDevice> {
    critical_section::with(|cs| {
        DEVICES_LIST
            .borrow_ref(cs)
            .iter()
            .find(|item| item.is_default)
            .cloned()
    })
}

pub async fn get_device_by_id(device_id: Option<DeviceIdentifier>) -> Option<AudioDevice> {
    match device_id {
        Some(id) => {
            let device = critical_section::with(|cs| {
                DEVICES_LIST
                    .borrow_ref(cs)
                    .iter()
                    .find(|item| item.id == id)
                    .cloned()
            });
            match device {
                Some(device) => Some(device),
                None => get_default_device().await,
            }
        }
        None => get_default_device().await,
    }
}

pub async fn get_devices() -> Vec<AudioDevice> {
    critical_section::with(|cs| DEVICES_LIST.borrow_ref(cs).to_vec())
}

#[allow(dead_code)]
pub async fn populate_dummy_data() {
    critical_section::with(|cs| {
        use alloc::string::ToString;

        let default_device_id = "headphones".to_string();

        let dummy_devices = Vec::from([
            AudioDevice {
                id: "speaker".to_string(),
                name: "Speaker".to_string(),
                friendly_name: "Speaker".to_string(),
                direction: SessionDirection::Render,
                is_default: false,
                volume: AudioVolume::new(0.3),
            },
            AudioDevice {
                id: default_device_id.clone(),
                name: default_device_id.clone(),
                friendly_name: default_device_id.clone(),
                direction: SessionDirection::Render,
                is_default: true,
                volume: AudioVolume::new(0.5),
            },
            AudioDevice {
                id: "asus_v231".to_string(),
                name: "ASUS V231".to_string(),
                friendly_name: "ASUS V231".to_string(),
                direction: SessionDirection::Render,
                is_default: false,
                volume: AudioVolume::new(0.7),
            },
            AudioDevice {
                id: "asus_v232".to_string(),
                name: "Samsung Tv".to_string(),
                friendly_name: "Samsung Tv".to_string(),
                direction: SessionDirection::Render,
                is_default: false,
                volume: AudioVolume::new(0.7),
            },
            AudioDevice {
                id: "asus_v233".to_string(),
                name: "Green Apple".to_string(),
                friendly_name: "Green Apple".to_string(),
                direction: SessionDirection::Render,
                is_default: false,
                volume: AudioVolume::new(0.7),
            },
        ]);

        let application = Vec::from([
            AudioApplication {
                device_id: default_device_id.clone(),
                process: ProcessInfo {
                    id: 100,
                    name: "Steam".to_string(),
                    path: None,
                },
                session_type: SessionType::Application,
                direction: SessionDirection::Render,
                volume: AudioVolume::new(0.5),
            },
            AudioApplication {
                device_id: default_device_id.clone(),
                process: ProcessInfo {
                    id: 100,
                    name: "Firefox".to_string(),
                    path: None,
                },
                session_type: SessionType::Application,
                direction: SessionDirection::Render,
                volume: AudioVolume::new(0.5),
            },
            AudioApplication {
                device_id: default_device_id.clone(),
                process: ProcessInfo {
                    id: 100,
                    name: "Discord".to_string(),
                    path: None,
                },
                session_type: SessionType::Application,
                direction: SessionDirection::Render,
                volume: AudioVolume::new(0.5),
            },
            AudioApplication {
                device_id: default_device_id.clone(),
                process: ProcessInfo {
                    id: 100,
                    name: "RPG.exe".to_string(),
                    path: None,
                },
                session_type: SessionType::Application,
                direction: SessionDirection::Render,
                volume: AudioVolume::new(0.5),
            },
        ]);

        let application_ids = application
            .iter()
            .map(|app| app.process.id)
            .collect::<Vec<_>>();

        let mut applications_list = APPLICATIONS_LIST.borrow_ref_mut(cs);
        applications_list.clear();
        applications_list.insert(default_device_id.clone(), application);

        let mut applications_id_list = APPLICATIONS_ID_LIST.borrow_ref_mut(cs);
        applications_id_list.clear();
        applications_id_list.insert(default_device_id.clone(), application_ids);

        let mut vef = DEVICES_LIST.borrow_ref_mut(cs);
        vef.clear();
        vef.extend(dummy_devices);
    });
}
