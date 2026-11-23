use std::sync::Arc;

use anyhow::Result;
use kavionics_core::{Vehicle, VehicleControl, VehicleError, VesselSnapshot};
use kavionics_krpc::KrpcVehicle;
use stayputnik::{Client, services::space_center::SpaceCenter};
use tokio::sync::watch;
use tracing::info;

pub struct MissionRuntime {
    vehicle: Arc<dyn Vehicle>,
}

impl MissionRuntime {
    pub fn new(vehicle: Arc<dyn Vehicle>) -> Self {
        Self { vehicle }
    }

    pub async fn tick(&self) -> Result<(), VehicleError> {
        let state = self.vehicle.snapshot().await?;

        tracing::debug!(
            ut = state.ut,
            altitude = state.flight.altitude_asl,
            apoapsis = state.orbit.apoapsis,
            "Vehicle state snapshot",
        );

        Ok(())
    }
}

pub async fn run() -> Result<()> {
    kavionics_telemetry::init_tracing();

    let client_name = format!("kavionics {}", env!("CARGO_PKG_VERSION"));
    let client = Client::connect(
        &client_name,
        "127.0.0.1",
        50000,
    )
        .await?
        .into_shared();

    info!("kRPC connected");

    let space_center = SpaceCenter::new(client);

    info!("Finding active vessel...");
    let mut vessel = space_center.active_vessel().await?;
    while vessel.is_none() {
        info!("Waiting for active vessel...");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        vessel = space_center.active_vessel().await?;
    }

    let vessel = vessel.unwrap();
    info!(
        vessel = %vessel.name().await?,
        "Active vessel found"
    );

    let vehicle = KrpcVehicle::new(vessel.clone(), Arc::new(space_center));
    let (tx, rx) = tokio::sync::watch::channel(None::<VesselSnapshot>);
    let vehicle = Arc::new(vehicle);
    let vehicle_telemetry = vehicle.clone();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(
            std::time::Duration::from_millis(200),
        );

        loop {
            interval.tick().await;

            match vehicle_telemetry.snapshot().await {
                Ok(state) => {
                    tx.send_replace(Some(VesselSnapshot {
                        state,
                        received_at: std::time::Instant::now(),
                    }));
                }
                Err(error) => {
                    tracing::warn!(%error, "Failed to acquire vehicle telemetry");
                }
            }
        }
    });

    Ok(launch(vehicle, rx).await?)
}

async fn launch<V>(
    vehicle: Arc<V>,
    mut telemetry: watch::Receiver<Option<VesselSnapshot>>,
) -> anyhow::Result<()>
where
    V: VehicleControl,
{
    tracing::info!("Starting launch sequence...");

    vehicle.set_throttle(1.0).await?;
    vehicle.activate_next_stage().await?;

    tracing::info!("Launch sequence initiated. Monitoring telemetry...");

    loop {
        telemetry.changed().await?;

        let Some(snapshot) = telemetry.borrow_and_update().clone() else {
            continue;
        };

        let altitude = snapshot.state.flight.altitude_asl;

        tracing::debug!(
            altitude,
            vertical_speed = snapshot.state.flight.vertical_speed,
            "Ascent telemetry",
        );

        if altitude >= 10_000.0 {
            vehicle.set_throttle(0.0).await?;

            tracing::info!("10 km reached; engines cut");

            return Ok(());
        }
    }
}
