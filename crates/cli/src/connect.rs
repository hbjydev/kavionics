use std::sync::Arc;

use anyhow::Result;
use kavionics_krpc::KspVehicle;
use stayputnik::{
    Client,
    ClientRef,
    services::{
        krpc::KRPC,
        ksa::KSA,
        space_center::SpaceCenter,
    },
};
use tracing::info;

pub async fn connect_vehicle() -> Result<KspVehicle> {
    let client_name = format!("kavionics {}", env!("CARGO_PKG_VERSION"));
    let client = Client::connect(&client_name, "127.0.0.1", 50000)
        .await?
        .into_shared();

    info!("kRPC connected");

    match detect_game(client.clone()).await? {
        Game::KSP => {
            info!("Detected KSP");
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

            let vehicle = KspVehicle::new(vessel.clone(), Arc::new(space_center));

            Ok(vehicle)
        }
        Game::KSA => {
            info!("Detected KSA");
            let ksa = KSA::new(client);

            let mut vessel = ksa.active_vehicle().await?;
            while vessel.is_none() {
                info!("Waiting for active vehicle...");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                vessel = ksa.active_vehicle().await?;
            }
            let vessel = vessel.unwrap();

            info!(
                vessel = %vessel.name().await?,
                "Active vessel found"
            );

            anyhow::bail!("KSA is not yet supported");
        }
    }
}

enum Game {
    KSP,
    KSA,
}

async fn detect_game(client: ClientRef) -> Result<Game> {
    let krpc = KRPC::new(client.clone());
    let svcs = krpc.get_services().await?;

    if let Some(_) = svcs.services.iter().find(|s| s.name == "KSA") {
        Ok(Game::KSA)
    } else if let Some(_) = svcs.services.iter().find(|s| s.name == "SpaceCenter") {
        Ok(Game::KSP)
    } else {
        anyhow::bail!("Could not detect game type");
    }
}
