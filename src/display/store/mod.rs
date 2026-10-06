pub mod dummy;
mod update;

use alloc::vec::Vec;
use core::cell::RefCell;
use critical_section::Mutex;
use shared_types::{AudioApplication, AudioDevice, DeviceIdentifier};

mod storage;
use storage::AudioStore;
pub use update::*;

static STORE: Mutex<RefCell<AudioStore>> = Mutex::new(RefCell::new(AudioStore::new()));

pub fn with<R>(f: impl FnOnce(&mut AudioStore) -> R) -> R {
    critical_section::with(|cs| f(&mut STORE.borrow_ref_mut(cs)))
}

pub async fn get_applications(device_id: Option<DeviceIdentifier>) -> Vec<AudioApplication> {
    with(|s| {
        let id = device_id
            .or_else(|| s.default_device().map(|d| d.id.clone()))
            .unwrap_or_default();
        s.apps_of(&id).to_vec()
    })
}

#[allow(dead_code)]
pub async fn get_default_device() -> Option<AudioDevice> {
    with(|s| s.default_device().cloned())
}

pub async fn get_device_by_id(device_id: Option<DeviceIdentifier>) -> Option<AudioDevice> {
    with(|s| {
        device_id
            .and_then(|id| s.device(&id))
            .or_else(|| s.default_device())
            .cloned()
    })
}

pub fn is_waiting_for_data() -> bool {
    with(|s| s.devices().is_empty())
}

pub async fn get_devices() -> Vec<AudioDevice> {
    with(|s| s.devices().to_vec())
}
