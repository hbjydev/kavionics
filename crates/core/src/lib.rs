use async_trait::async_trait;

use crate::vessel_state::VesselState;

pub mod vessel_state;

#[derive(Debug, Clone)]
pub struct VesselSnapshot {
    pub state: VesselState,
    pub received_at: std::time::Instant,
}

#[derive(Debug, thiserror::Error)]
pub enum VehicleError {
    #[error("vehicle is unavailable")]
    Unavailable,

    #[error("vehicle communication failed: {0}")]
    Communication(#[source] Box<dyn std::error::Error + Send + Sync>),

    #[error("invalid telemetry: {0}")]
    InvalidTelemetry(String),
}

#[async_trait]
pub trait Vehicle {
    async fn snapshot(&self) -> Result<VesselState, VehicleError>;
}

#[async_trait]
pub trait VehicleControl: Send + Sync {
    async fn set_throttle(&self, throttle: f32) -> Result<(), VehicleError>;
    async fn activate_next_stage(&self) -> Result<(), VehicleError>;
}
