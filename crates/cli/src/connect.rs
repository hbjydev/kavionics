use std::sync::Arc;

use anyhow::Result;
use kavionics_krpc::KrpcVehicle;
use stayputnik::{Client, services::space_center::SpaceCenter};
use tracing::info;

pub async fn connect_vehicle() -> Result<KrpcVehicle> {
    let client_name = format!("kavionics {}", env!("CARGO_PKG_VERSION"));
    let client = Client::connect(&client_name, "127.0.0.1", 50000)
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

    Ok(vehicle)
}
