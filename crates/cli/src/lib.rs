use std::sync::Arc;

use anyhow::Result;
use kavionics_krpc::collect_telemetry;
use kavionics_mission::{MissionConfig, MissionExecutor};
use tokio::sync::watch;

mod connect;

pub async fn run() -> Result<()> {
    kavionics_telemetry::init_tracing();

    let mission = MissionConfig::load(
        std::path::Path::new(std::env::args().nth(1).as_deref().unwrap_or("mission.pkl")),
    )
        .inspect_err(|error| tracing::error!(%error, "Failed to load mission config"))?;

    let vehicle = Arc::new(
        connect::connect_vehicle().await
            .inspect_err(|error| tracing::error!(%error, "Failed to connect to vehicle"))?
    );

    let (tx, rx) = watch::channel(None);

    let telemetry_vehicle = Arc::clone(&vehicle);

    let collector = tokio::spawn(async move {
        collect_telemetry(telemetry_vehicle, tx).await;
    });

    let mut mission_executor = MissionExecutor::new(
        vehicle.clone(),
        rx.clone(),
    );
    let result = mission_executor.run(&mission.phases).await;

    collector.abort();

    result
}
