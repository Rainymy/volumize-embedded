use alloc::string::String;
use serde::{Deserialize, Serialize};

pub type VolumePercent = f32;
pub type AppIdentifier = u32;
pub type DeviceIdentifier = alloc::string::String;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    Application,
    Device,
    System,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionDirection {
    Render,
    Capture,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub id: AppIdentifier,
    pub name: String,
    pub path: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioApplication {
    pub process: ProcessInfo,
    pub session_type: SessionType,
    pub direction: SessionDirection,
    pub volume: AudioVolume,
    pub device_id: DeviceIdentifier,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: DeviceIdentifier,
    pub name: String,
    pub friendly_name: String,
    pub direction: SessionDirection,
    pub is_default: bool,
    pub volume: AudioVolume,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVolume {
    pub current: VolumePercent,
    pub muted: bool,
}
