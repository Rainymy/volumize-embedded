use shared_types::protocol::CommandResponse;
use shared_types::{
    AudioVolume, ChangeType, EntityState, UpdateChange,
    protocol::{Envelope, Response},
};

pub fn update_information(envelope: Envelope) {
    match envelope {
        Envelope::Command(_) => { /* Outgoing only. */ }
        Envelope::Response(response) => update_response(response),
        Envelope::Event(change) => update_event(change),
    }
}

fn update_response(response: CommandResponse) {
    super::with(|s| match response.response {
        Response::Volume { id, volume } => s.set_volume(&id, volume),
        Response::DeviceList(list) => s.replace_devices(list),
        Response::ApplicationList { id, apps } => {
            let missing = s.sync_app_list(&id, &apps);
            if !missing.is_empty() {
                defmt::debug!("{} app(s) listed without details", missing.len());
            }
        }
        Response::Application(app) => s.update_or_insert_app(app),
        // No icon storage yet.
        Response::Icon { .. } => {}
        Response::ACK => defmt::info!("ACK received"),
        Response::Error { message } => defmt::error!("Error: {}", message.as_str()),
    });
}

fn update_event(event: UpdateChange) {
    let UpdateChange { id, change } = event;

    super::with(|s| match change {
        ChangeType::AudioVolume { volume, mute } => s.set_volume(
            &id,
            AudioVolume {
                current: volume,
                muted: mute,
            },
        ),
        ChangeType::NameChange { name } => s.set_name(&id, name),
        ChangeType::IconPathChange { path } => s.set_icon_path(&id, path),
        ChangeType::StateChange {
            state: EntityState::Disconnect,
        } => s.remove(&id),
        // The event has no payload; the new entity arrives via a later
        // DeviceList / ApplicationList / Application response.
        ChangeType::StateChange {
            state: EntityState::Created,
        } => defmt::debug!("Entity created, waiting for details"),
    });
}
