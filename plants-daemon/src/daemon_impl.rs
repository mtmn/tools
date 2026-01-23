use common::status::Status;
use zbus::{interface, object_server::SignalEmitter};

pub struct PlantsDaemon;

#[interface(name = "org.mtmn.Plants")]
impl PlantsDaemon {
    #[zbus(signal)]
    async fn update(emitter: &SignalEmitter<'_>, status: Status) -> zbus::Result<()>;
}
