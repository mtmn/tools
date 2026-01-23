use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use bluer::{
    DeviceEvent, DeviceProperty, ErrorKind,
    rfcomm::{Profile, Role, Stream},
};
use futures::StreamExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time,
};
use zbus::object_server::InterfaceRef;

use crate::{
    airpods_consts::{
        AIRPODS_SERVICE, FEATURES_ACK, HANDSHAKE, HANDSHAKE_ACK, REQUEST_NOTIFICATIONS,
        SET_SPECIFIC_FEATURES,
    },
    daemon_impl::{PlantsDaemon, PlantsDaemonSignals},
    packets::{
        battery::{BatteryPacket, Pod},
        in_ear::InEarPacket,
        metadata::MetadataPacket,
    },
};
use common::status::Status;

#[derive(Default)]
struct LocalState {
    primary: Pod,
    // Keep a local copy to track changes and hash,
    // but we also sync to the global shared state.
    status: Status,
}

pub async fn run(interface: InterfaceRef<PlantsDaemon>, state: Arc<Mutex<Status>>) -> Result<()> {
    // Use a separate session for AirPods scanning
    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let profile = Profile {
        uuid: AIRPODS_SERVICE,
        role: Some(Role::Client),
        service: Some(AIRPODS_SERVICE),
        ..Default::default()
    };
    let mut profile = session.register_profile(profile).await?;

    tracing::info!("Scanning for AirPods");
    let device = get_airpods(&adapter).await?;
    tracing::info!("Found AirPods [{}]", device.address());

    // Connection loop
    let device_addr = device.address();
    let device = adapter.device(device_addr)?; // Re-get device to ensure we have valid handle

    // Spawn a task to maintain connection
    let device_cl = device.clone();
    tokio::spawn(async move {
        // Initial connect attempt
        if let Ok(true) = device_cl.is_connected().await {
            let _ = device_cl.connect_profile(&AIRPODS_SERVICE).await;
        }

        if let Ok(mut events) = device_cl.events().await {
            while let Some(event) = events.next().await {
                if let DeviceEvent::PropertyChanged(DeviceProperty::Connected(true)) = event {
                    // Retry connecting profile until success or error is not InProgress
                    while let Err(err) = device_cl.connect_profile(&AIRPODS_SERVICE).await {
                        if err.kind == ErrorKind::InProgress {
                            time::sleep(Duration::from_millis(500)).await;
                        } else {
                            // Backoff slightly
                            time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
    });

    while let Some(handle) = profile.next().await {
        tracing::debug!("AirPods profile connected");
        let mut stream = handle.accept()?;
        match handle_connection(&interface, &state, &mut stream).await {
            Ok(()) => {
                tracing::info!("AirPods connection closed normally");
            }
            Err(e) => {
                tracing::warn!("AirPods connection error: {}", e);
            }
        }

        // Clear status on disconnect
        {
            let mut gs = state.lock().unwrap();
            gs.components = common::status::Components::default();
            gs.ear = common::status::InEar::default();
            gs.metadata = None;
        }

        let status = {
            let gs = state.lock().unwrap();
            gs.clone()
        };
        interface.update(status).await?;
    }

    Ok(())
}

async fn handle_connection(
    interface: &InterfaceRef<PlantsDaemon>,
    global_state: &Arc<Mutex<Status>>,
    stream: &mut Stream,
) -> Result<()> {
    stream.write_all(HANDSHAKE).await?;

    let mut local_state = LocalState::default();

    loop {
        let mut data = Vec::new();

        loop {
            let mut buffer = vec![0; 1024];
            let bytes = stream.read(&mut buffer).await?;
            if bytes == 0 {
                anyhow::bail!("Stream ended");
            }
            data.extend_from_slice(&buffer[..bytes]);

            if bytes < buffer.len() {
                break;
            }
        }

        if data.starts_with(HANDSHAKE_ACK) {
            stream.write_all(SET_SPECIFIC_FEATURES).await?;
        } else if data.starts_with(FEATURES_ACK) {
            stream.write_all(REQUEST_NOTIFICATIONS).await?;
        } else {
            let hash = local_state.status.hash();
            got_packet(&mut local_state, &data);

            if hash != local_state.status.hash() {
                // Update global state
                {
                    let mut gs = global_state.lock().unwrap();
                    gs.components = local_state.status.components.clone(); // Assuming Clone is derived
                    gs.ear = local_state.status.ear.clone(); // Assuming Clone
                    // Metadata?
                    if let Some(m) = &local_state.status.metadata {
                        gs.metadata = Some(common::status::Metadata {
                            name: m.name.clone(),
                            model: m.model.clone(),
                        });
                    }
                }

                let status = {
                    let gs = global_state.lock().unwrap();
                    gs.clone()
                };
                interface.update(status).await?;
            }
        }
    }
}

fn got_packet(state: &mut LocalState, data: &[u8]) {
    if let Some(metadata) = MetadataPacket::parse(data) {
        tracing::debug!("Got Metadata: {:?}", metadata);
        state.status.metadata = Some(metadata.into());
    } else if let Some(battery) = BatteryPacket::parse(data) {
        tracing::debug!("Got Battery: {:?}", battery);
        state.primary = battery.primary;

        // Sync local components
        if let Some(l) = battery.left {
            state.status.components.left = Some(l.into());
        }
        if let Some(r) = battery.right {
            state.status.components.right = Some(r.into());
        }
        if let Some(c) = battery.case {
            state.status.components.case = Some(c.into());
        }
    } else if let Some(in_ear) = InEarPacket::parse(data) {
        tracing::debug!("Got InEar: {:?}", in_ear);

        if let Some([left, right]) = in_ear.get(state.primary) {
            state.status.ear.left = left.into();
            state.status.ear.right = right.into();
        }
    }
}

async fn get_airpods(adapter: &bluer::Adapter) -> Result<bluer::Device> {
    loop {
        let connected = adapter.device_addresses().await?;
        for addr in connected {
            let device = adapter.device(addr)?;
            // Can't always get UUIDs immediately, handling error gracefully
            let uuids = device.uuids().await.ok().flatten().unwrap_or_default();
            if uuids.contains(&AIRPODS_SERVICE) {
                return Ok(device);
            }
        }

        time::sleep(Duration::from_secs(5)).await;
    }
}
