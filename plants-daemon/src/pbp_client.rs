use anyhow::Result;
use bluer::rfcomm::{Profile, ProfileHandle, ReqError, Role, Stream};
use bluer::{Adapter, Address, Device, Session};
use common::status::{BatteryStatus, ComponentStatus, Components, EarStatus, Status};
use futures::StreamExt;
use maestro::protocol::codec::Codec;
use maestro::protocol::utils;
use maestro::pwrpc::client::Client;
use maestro::service::MaestroService;
use std::time::Duration;

pub async fn stream_pbp_stats<F>(
    session: &Session,
    adapter: &Adapter,
    mac: Address,
    callback: F,
) -> Result<()>
where
    F: Fn(Status) + Send + Sync + 'static,
{
    let dev = adapter.device(mac)?;
    tracing::debug!("Connecting to PBP RFCOMM at {}", mac);

    // Connect RFCOMM
    let stream = connect_maestro_rfcomm(session, &dev).await?;
    tracing::debug!("RFCOMM connected");

    // Setup Codec
    let codec = Codec::new();
    let stream = codec.wrap(stream);

    // Setup RPC Client
    let mut client = Client::new(stream);
    let handle = client.handle();

    // Resolve channel first - pbpctrl does this before client.run()
    let channel = utils::resolve_channel(&mut client).await?;
    tracing::debug!("Maestro channel resolved: {}", channel);

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);

    let task = async move {
        let mut service = MaestroService::new(handle, channel);
        let mut call = service.subscribe_to_runtime_info()?;
        tracing::debug!("Subscribed to RuntimeInfo");

        let mut stream = call.stream();
        while let Some(msg) = stream.next().await {
            let info = msg?;
            tracing::trace!("Received RuntimeInfo update: {:?}", info);
            if tx.send(info).await.is_err() {
                break;
            }
        }

        Ok::<_, anyhow::Error>(())
    };

    tokio::select! {
        res = client.run() => {
            tracing::warn!("Client run loop terminated: {:?}", res);
            res?;
            anyhow::bail!("client terminated unexpectedly");
        },
        res = task => {
             tracing::warn!("Subscription task terminated: {:?}", res);
            res?;
        }
        () = async {
            while let Some(info) = rx.recv().await {
                callback(runtime_info_to_status(info));
            }
        } => {}
    }

    Ok(())
}

fn runtime_info_to_status(info: maestro::protocol::types::RuntimeInfo) -> Status {
    let mut components = Components::default();
    let mut ear = common::status::InEar::default();

    // Map Case
    if let Some(c) = info.battery_info.as_ref().and_then(|b| b.case.as_ref()) {
        components.case = Some(ComponentStatus {
            level: u8::try_from(c.level).unwrap_or(0),
            status: if c.state == 2 {
                BatteryStatus::Charging
            } else {
                BatteryStatus::Discharging
            },
        });
    }

    // Map Left
    if let Some(l) = info.battery_info.as_ref().and_then(|b| b.left.as_ref()) {
        components.left = Some(ComponentStatus {
            level: u8::try_from(l.level).unwrap_or(0),
            status: if l.state == 2 {
                BatteryStatus::Charging
            } else {
                BatteryStatus::Discharging
            },
        });
        if let Some(placement) = &info.placement {
            ear.left = if placement.left_bud_in_case {
                EarStatus::InCase
            } else {
                EarStatus::InEar
            };
        }
    }

    // Map Right
    if let Some(r) = info.battery_info.as_ref().and_then(|b| b.right.as_ref()) {
        components.right = Some(ComponentStatus {
            level: u8::try_from(r.level).unwrap_or(0),
            status: if r.state == 2 {
                BatteryStatus::Charging
            } else {
                BatteryStatus::Discharging
            },
        });
        if let Some(placement) = &info.placement {
            ear.right = if placement.right_bud_in_case {
                EarStatus::InCase
            } else {
                EarStatus::InEar
            };
        }
    }

    Status {
        metadata: None,
        components,
        ear,
        devices: Vec::new(),
    }
}

async fn connect_maestro_rfcomm(session: &Session, dev: &Device) -> Result<Stream> {
    let maestro_profile = Profile {
        uuid: maestro::UUID,
        role: Some(Role::Client),
        require_authentication: Some(false),
        require_authorization: Some(false),
        auto_connect: Some(false),
        ..Default::default()
    };

    let mut handle = session.register_profile(maestro_profile).await?;

    let stream = tokio::try_join!(
        try_connect_profile(dev),
        handle_requests_for_profile(&mut handle, dev.address()),
    )?
    .1;

    Ok(stream)
}

async fn try_connect_profile(dev: &Device) -> Result<()> {
    const RETRY_TIMEOUT: Duration = Duration::from_secs(1);
    const MAX_TRIES: u32 = 3;

    let mut i = 0;
    while let Err(err) = dev.connect_profile(&maestro::UUID).await {
        if i >= MAX_TRIES {
            return Err(err.into());
        }
        i += 1;
        tokio::time::sleep(RETRY_TIMEOUT).await;
    }
    Ok(())
}

async fn handle_requests_for_profile(
    handle: &mut ProfileHandle,
    address: Address,
) -> Result<Stream> {
    while let Some(req) = handle.next().await {
        if req.device() == address {
            return Ok(req.accept()?);
        }
        req.reject(ReqError::Rejected);
    }
    anyhow::bail!("profile terminated without requests")
}
