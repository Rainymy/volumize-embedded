use alloc::string::{String, ToString};
use alloc::vec::Vec;
use shared_types::{
    AppIdentifier, AudioApplication, AudioDevice, AudioVolume, ProcessInfo, SessionDirection,
    SessionType,
};

fn device(id: &str, name: &str, volume: f32, is_default: bool) -> AudioDevice {
    AudioDevice {
        id: id.to_string(),
        name: name.to_string(),
        friendly_name: name.to_string(),
        direction: SessionDirection::Render,
        is_default,
        volume: AudioVolume::new(volume),
    }
}

fn app(device_id: &str, id: AppIdentifier, name: &str, volume: f32) -> AudioApplication {
    AudioApplication {
        device_id: String::from(device_id),
        process: ProcessInfo {
            id,
            name: name.to_string(),
            path: None,
        },
        session_type: SessionType::Application,
        direction: SessionDirection::Render,
        volume: AudioVolume::new(volume),
    }
}

#[allow(dead_code)]
pub async fn populate_dummy_data() {
    let default_id = "headphones";
    let devices = Vec::from([
        device(default_id, "Headphones", 0.5, true),
        device("speaker", "Speaker", 0.3, false),
        device("asus_v231", "ASUS V231", 0.7, false),
        device("asus_v232", "Samsung Tv", 0.7, false),
        device("asus_v233", "Green Apple", 0.7, false),
    ]);
    let apps = Vec::from([
        app(default_id, 100, "Steam", 0.5),
        app(default_id, 101, "Firefox", 0.5),
        app(default_id, 102, "Discord", 0.5),
        app(default_id, 103, "RPG.exe", 0.5),
    ]);

    super::with(|s| {
        s.clear();
        s.replace_devices(devices);
        s.replace_apps(default_id.to_string(), apps);
    });
}
