use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use common::status::Status;
use tokio::time;
use zbus::object_server::InterfaceRef;

use crate::daemon_impl::{PlantsDaemon, PlantsDaemonSignals};

pub async fn run(interface: InterfaceRef<PlantsDaemon>, state: Arc<Mutex<Status>>) -> Result<()> {
    let mut session: Option<bluer::Session> = None;

    // Load config once to check for buds
    let config = crate::config::load_config().await.ok();
    let target_mac = if let Some(c) = &config {
        if let Some(buds) = &c.buds {
            buds.mac.parse::<bluer::Address>().ok()
        } else {
            None
        }
    } else {
        None
    };

    if target_mac.is_some() {
        session = bluer::Session::new().await.ok();
    }

    loop {
        let mut should_run = true;

        if let Some(mac) = target_mac {
            should_run = false;
            if let Some(sess) = &session {
                // Check if device is connected with timeouts
                let is_connected = async {
                    let Ok(Ok(adapter)) =
                        time::timeout(Duration::from_secs(2), sess.default_adapter()).await
                    else {
                        return false;
                    };

                    let Ok(device) = adapter.device(mac) else {
                        return false;
                    };

                    matches!(
                        time::timeout(Duration::from_secs(2), device.is_connected()).await,
                        Ok(Ok(true))
                    )
                }
                .await;

                if is_connected {
                    should_run = true;
                }
            } else if let Ok(Ok(s)) =
                time::timeout(Duration::from_secs(2), bluer::Session::new()).await
            {
                session = Some(s);
            }
        }

        if should_run {
            if let (Some(sess), Some(mac)) = (&session, target_mac) {
                // Keep trying to stream as long as connected
                if let Ok(adapter) = sess.default_adapter().await {
                    let res = crate::pbp_client::stream_pbp_stats(
                        sess,
                        &adapter,
                        bluer::Address(*mac),
                        {
                            let state = state.clone();
                            let interface = interface.clone();
                            move |new_status| {
                                {
                                    let mut status = state.lock().unwrap();
                                    status.components = new_status.components;
                                    status.ear = new_status.ear;
                                }

                                let status = {
                                    let status = state.lock().unwrap();
                                    status.clone()
                                };

                                // We need to spawn this because callback is sync but update is async
                                let interface = interface.clone();
                                tokio::spawn(async move {
                                    if let Err(e) = interface.update(status).await {
                                        tracing::error!("Failed to update plants: {}", e);
                                    }
                                });
                            }
                        },
                    )
                    .await;

                    if let Err(e) = res {
                        tracing::error!("PBP stream error: {}", e);
                    }
                }
            }
        } else {
            // If we are skipping, ensure we don't show stale info
            {
                let mut status = state.lock().unwrap();
                // Only clear if metadata is None (implying it might be PBP data).
                if status.metadata.is_none() {
                    status.components = common::status::Components::default();
                    status.ear = common::status::InEar::default();
                }
            }
            // Trigger update to clear PBP info from bar if present
            let status = {
                let status = state.lock().unwrap();
                status.clone()
            };

            if let Err(e) = interface.update(status).await {
                tracing::error!("Failed to update plants: {}", e);
            }
        }

        // Wait before retrying (e.g. if disconnected or error)
        time::sleep(Duration::from_secs(5)).await;
    }
}
