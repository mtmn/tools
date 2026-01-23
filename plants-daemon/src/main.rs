use anyhow::Result;

use zbus::conn;

use common::status::Status;
use std::sync::{Arc, Mutex};

mod airpods;
mod airpods_consts;
mod bluetooth;
mod config;
mod daemon_impl;
mod packets;
mod pbp;
mod pbp_client;

use crate::daemon_impl::{PlantsDaemon, PlantsDaemonSignals};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .init();

    let conn = conn::Builder::session()?
        .name("org.mtmn.Plants")?
        .serve_at("/org/mtmn/Plants", PlantsDaemon)?
        .build()
        .await?;
    let interface = conn.object_server().interface("/org/mtmn/Plants").await?;

    let state = Arc::new(Mutex::new(Status::default()));

    PlantsDaemonSignals::update(&interface, Status::default()).await?;

    let bt_state = state.clone();
    let bt_interface = interface.clone();
    tokio::spawn(async move {
        if let Err(e) = bluetooth::run(bt_interface, bt_state).await {
            tracing::error!("Bluetooth generic error: {}", e);
        }
    });

    let ap_state = state.clone();
    let ap_interface = interface.clone();
    tokio::spawn(async move {
        if let Err(e) = airpods::run(ap_interface, ap_state).await {
            tracing::error!("AirPods error: {}", e);
        }
    });

    let pbp_state = state.clone();
    let pbp_interface = interface.clone();
    pbp::run(pbp_interface, pbp_state).await?;

    Ok(())
}
