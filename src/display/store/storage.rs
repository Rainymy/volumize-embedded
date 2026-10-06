use alloc::{collections::BTreeMap, string::String, vec::Vec};
use shared_types::{
    AppIdentifier, AudioApplication, AudioDevice, AudioVolume, DeviceIdentifier, Identifier,
};

pub struct AudioStore {
    devices: Vec<AudioDevice>,
    apps: BTreeMap<DeviceIdentifier, Vec<AudioApplication>>,
}

#[allow(dead_code)]
impl AudioStore {
    pub const fn new() -> Self {
        Self {
            devices: Vec::new(),
            apps: BTreeMap::new(),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::new();
    }

    // ---- queries ----

    pub fn devices(&self) -> &[AudioDevice] {
        &self.devices
    }

    pub fn device(&self, id: &str) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| d.id == id)
    }

    pub fn default_device(&self) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| d.is_default)
    }

    pub fn apps_of(&self, device_id: &str) -> &[AudioApplication] {
        self.apps.get(device_id).map(Vec::as_slice).unwrap_or(&[])
    }

    // ---- bulk replace / update-insert ----

    pub fn replace_devices(&mut self, devices: Vec<AudioDevice>) {
        self.devices = devices;
    }

    pub fn replace_apps(&mut self, device_id: DeviceIdentifier, apps: Vec<AudioApplication>) {
        self.apps.insert(device_id, apps);
    }

    pub fn update_or_insert_app(&mut self, app: AudioApplication) {
        let list = self.apps.entry(app.device_id.clone()).or_default();
        match list.iter_mut().find(|a| a.process.id == app.process.id) {
            Some(slot) => *slot = app,
            None => list.push(app),
        }
    }

    pub fn sync_app_list(&mut self, device_id: &str, ids: &[AppIdentifier]) -> Vec<AppIdentifier> {
        let list = self.apps.entry(device_id.into()).or_default();
        list.retain(|a| ids.contains(&a.process.id));
        list.sort_by_key(|a| ids.iter().position(|id| *id == a.process.id));

        ids.iter()
            .copied()
            .filter(|id| !list.iter().any(|a| a.process.id == *id))
            .collect()
    }

    // ---- apps and devices updates ----

    pub fn set_volume(&mut self, target: &Identifier, volume: AudioVolume) {
        match target {
            Identifier::App(id) => self
                .apps_with_id_mut(*id)
                .for_each(|a| a.volume = volume.clone()),
            Identifier::Device(id) => {
                if let Some(d) = self.device_mut(id) {
                    d.volume = volume;
                }
            }
        }
    }

    pub fn set_name(&mut self, target: &Identifier, name: String) {
        match target {
            Identifier::App(id) => self
                .apps_with_id_mut(*id)
                .for_each(|a| a.process.name = name.clone()),
            Identifier::Device(id) => {
                if let Some(d) = self.device_mut(id) {
                    d.friendly_name = name.clone();
                    d.name = name;
                }
            }
        }
    }

    /// Devices have no icon path, so this only affects apps.
    pub fn set_icon_path(&mut self, target: &Identifier, path: String) {
        if let Identifier::App(id) = target {
            self.apps_with_id_mut(*id)
                .for_each(|a| a.process.path = Some(path.clone()));
        }
    }

    pub fn remove(&mut self, target: &Identifier) {
        match target {
            Identifier::App(id) => self
                .apps
                .values_mut()
                .for_each(|l| l.retain(|a| a.process.id != *id)),
            Identifier::Device(id) => {
                self.devices.retain(|d| &d.id != id);
                self.apps.remove(id);
            }
        }
    }

    // ---- internals ----

    fn device_mut(&mut self, id: &str) -> Option<&mut AudioDevice> {
        self.devices.iter_mut().find(|d| d.id == id)
    }

    fn apps_with_id_mut(
        &mut self,
        id: AppIdentifier,
    ) -> impl Iterator<Item = &mut AudioApplication> {
        self.apps
            .values_mut()
            .flatten()
            .filter(move |a| a.process.id == id)
    }
}
