use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result};
use kavionics_core::{VehicleControl, VesselSnapshot, vessel_state::VesselState};
use tokio::sync::watch;
use tracing::info;

use crate::{MissionAction, MissionPhase, condition::Condition};

pub struct MissionExecutor<V> {
    vehicle: Arc<V>,
    telemetry: watch::Receiver<Option<VesselSnapshot>>,
}

impl<V> MissionExecutor<V>
where
    V: VehicleControl + Send + Sync,
{
    pub fn new(
        vehicle: Arc<V>,
        telemetry: watch::Receiver<Option<VesselSnapshot>>,
    ) -> Self {
        Self { vehicle, telemetry }
    }

    pub async fn run(
        &mut self,
        phases: &[MissionPhase],
    ) -> Result<()> {
        for phase in phases {
            info!(phase = %phase.name, "Executing mission phase");

            for action in &phase.actions {
                self.execute_action(action).await?;
            }

            info!(phase = %phase.name, "Mission phase completed");
        }

        info!("Mission completed");
        Ok(())
    }

    async fn execute_action(
        &mut self,
        action: &MissionAction,
    ) -> Result<()> {
        match action {
            MissionAction::Stage => {
                self.vehicle.activate_next_stage().await?;
            },

            MissionAction::Wait { seconds } => {
                self.wait(*seconds).await?;
            },

            MissionAction::WaitUntil { condition } => {
                tracing::info!(
                    condition = %condition,
                    "Waiting for condition to be satisfied"
                );

                let condition = Condition::compile(condition)
                    .context("Failed to compile condition")?;

                let snapshot = self.wait_for_state(|state| {
                    condition.evaluate(state)
                }).await?;

                tracing::info!(
                    condition = %condition.source,
                    ut = snapshot.state.ut,
                    "Condition satisfied"
                )
            },

            MissionAction::SetThrottle { throttle } => {
                self.vehicle.set_throttle(*throttle).await?;
            },
        };

        Ok(())
    }

    async fn wait_for_state<F>(
        &mut self,
        predicate: F,
    ) -> Result<VesselSnapshot>
    where
        F: Fn(&VesselState) -> anyhow::Result<bool>,
    {
        const TELEMETRY_TIMEOUT: Duration = Duration::from_secs(2);

        loop {
            let snapshot = {
                self.telemetry.borrow_and_update().clone()
            };

            if let Some(snapshot) = snapshot {
                anyhow::ensure!(
                    snapshot.received_at.elapsed() <= TELEMETRY_TIMEOUT,
                    "Telemetry snapshot is stale"
                );

                if predicate(&snapshot.state)? {
                    return Ok(snapshot);
                }
            }

            tokio::time::timeout(
                TELEMETRY_TIMEOUT,
                self.telemetry.changed(),
            )
                .await
                .context("Timed out waiting for telemetry")?
                .context("Telemetry channel closed")?;
        }
    }

    async fn wait(
        &mut self,
        wait_seconds: f64,
    ) -> Result<()> {
        info!(wait_seconds, "Waiting for specified duration");
        let wait_time = Duration::from_secs_f64(wait_seconds);
        tokio::time::sleep(wait_time).await;
        Ok(())
    }
}
