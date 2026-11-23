use std::sync::Arc;

use kavionics_core::{Vehicle, VesselSnapshot};
use tracing::error;

pub async fn collect_telemetry<V>(
    vehicle: Arc<V>,
    telemetry_tx: tokio::sync::watch::Sender<Option<VesselSnapshot>>,
) where
    V: Vehicle + Send + Sync + 'static,
{
    let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
    loop {
        interval.tick().await;

        match vehicle.snapshot().await {
            Ok(state) => {
                let snapshot = VesselSnapshot {
                    state,
                    received_at: std::time::Instant::now(),
                };

                tracing::debug!(snapshot = ?snapshot, "Telemetry collected");

                let _ = telemetry_tx.send(Some(snapshot));
            }
            Err(e) => {
                error!(error = %e, "Failed to collect telemetry");
                let _ = telemetry_tx.send(None);
            }
        }
    }
}
