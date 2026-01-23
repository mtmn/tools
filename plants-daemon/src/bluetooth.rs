use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use bluer::{AdapterEvent, Address};
use common::status::{BatteryStatus, GenericDeviceStatus, Status};
use futures::{StreamExt, pin_mut};
use tokio::time;
use zbus::object_server::InterfaceRef;

use crate::{
    config::Config,
    daemon_impl::{PlantsDaemon, PlantsDaemonSignals},
};

pub async fn run(interface: InterfaceRef<PlantsDaemon>, state: Arc<Mutex<Status>>) -> Result<()> {
    let config = match crate::config::load_config().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            tracing::error!("Failed to load config: {}", e);
            Config {
                devices: std::collections::HashMap::default(),
                buds: None,
            }
        }
    };

    if config.devices.is_empty() {
        tracing::info!("No generic devices configured.");
        return Ok(());
    }

    let session = bluer::Session::new().await?;
    let adapter = session.default_adapter().await?;
    adapter.set_powered(true).await?;

    let zbus_conn = zbus::Connection::system().await.ok();

    // Initial update
    update_devices(&adapter, &config, &state, &interface, zbus_conn.as_ref()).await;

    // Listen for events
    let events = adapter.events().await?;
    pin_mut!(events);

    // Simple debounce: don't update more often than once per second
    let mut last_update = time::Instant::now();

    loop {
        tokio::select! {
             // Polling interval
            () = time::sleep(Duration::from_secs(30)) => {
                 update_devices(&adapter, &config, &state, &interface, zbus_conn.as_ref()).await;
            }
            Some(event) = events.next() => {
                 match event {
                    AdapterEvent::DeviceAdded(_) | AdapterEvent::DeviceRemoved(_) | AdapterEvent::PropertyChanged(_) => {
                        if last_update.elapsed() > Duration::from_millis(500) {
                            update_devices(&adapter, &config, &state, &interface, zbus_conn.as_ref()).await;
                            last_update = time::Instant::now();
                        }
                    }
                 }
            }
        }
    }
}

async fn update_devices(
    adapter: &bluer::Adapter,
    config: &Config,
    state: &Arc<Mutex<Status>>,
    interface: &InterfaceRef<PlantsDaemon>,
    zbus_conn: Option<&zbus::Connection>,
) {
    let mut new_devices = Vec::new();

    for (name, device_cfg) in &config.devices {
        if device_cfg.device_type != "bluetooth" {
            continue;
        }

        let address: Address = match device_cfg.mac.parse() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::error!("Invalid MAC address for {}: {}", name, e);
                continue;
            }
        };

        // Check if device is available in adapter
        if let Ok(device) = adapter.device(address) {
            let is_connected = time::timeout(Duration::from_secs(2), device.is_connected())
                .await
                .map(|r| r.unwrap_or(false))
                .unwrap_or(false);

            if is_connected {
                let battery_pct = if let Some(conn) = zbus_conn {
                    get_battery_percentage(adapter.name(), address, conn).await
                } else {
                    None
                };

                if let Some(pct) = battery_pct {
                    new_devices.push(GenericDeviceStatus {
                        name: name.clone(),
                        battery: pct,
                        text: device_cfg.text.clone(),
                        status: BatteryStatus::Discharging,
                    });
                }
            }
        }
    }

    {
        let mut status = state.lock().unwrap();
        status.devices = new_devices;
    }

    let status = {
        let status = state.lock().unwrap();
        status.clone()
    };

    if let Err(e) = interface.update(status).await {
        tracing::error!("Failed to update waybar: {}", e);
    }
}

async fn get_battery_percentage(
    adapter_name: &str,
    address: bluer::Address,
    conn: &zbus::Connection,
) -> Option<u8> {
    // Construct object path: /org/bluez/{adapter}/dev_{mac_with_underscores}
    let addr_str = address.to_string().replace(':', "_");
    let path_str = format!("/org/bluez/{adapter_name}/dev_{addr_str}");
    let path = zbus::zvariant::ObjectPath::try_from(path_str).ok()?;

    // Create a proxy for the Battery1 interface on the device path
    let proxy = zbus::Proxy::new(conn, "org.bluez", &path, "org.bluez.Battery1")
        .await
        .ok()?;

    match proxy.get_property::<u8>("Percentage").await {
        Ok(pct) => Some(pct),
        Err(e) => {
            tracing::debug!("Failed to get battery percentage for {}: {}", address, e);
            None
        }
    }
}
