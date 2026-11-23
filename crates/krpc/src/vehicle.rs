use std::sync::Arc;

use async_trait::async_trait;
use kavionics_core::{Vehicle, VehicleControl, VehicleError, vessel_state::{FlightState, OrbitState, PropulsionState, ResourceState, VesselState}};
use stayputnik::services::space_center::{SpaceCenter, Vessel};

#[derive(Clone)]
pub struct KrpcVehicle {
    space_center: Arc<SpaceCenter>,
    vessel: stayputnik::services::space_center::Vessel,
}

impl KrpcVehicle {
    pub fn new(vessel: Vessel, space_center: Arc<SpaceCenter>) -> Self {
        Self { vessel, space_center }
    }

    async fn universal_time(&self) -> Result<f64, VehicleError> {
        self.space_center.ut().await.map_err(|e| VehicleError::Communication(Box::new(e)))
    }

    async fn flight_state(&self) -> Result<FlightState, VehicleError> {
        let orbit = self
            .vessel
            .orbit()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        let body = orbit
            .body()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        // Measure speeds relative to the rotating surface. The vessel's own
        // reference frame moves with it, so speeds in that frame are zero.
        let reference_frame = body
            .reference_frame()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        let flight = self
            .vessel
            .flight(Some(Some(reference_frame)))
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        Ok(FlightState {
            altitude_asl: flight.mean_altitude().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            altitude_agl: flight.surface_altitude().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            vertical_speed: flight.vertical_speed().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            surface_speed: flight.speed().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            dynamic_pressure: flight.dynamic_pressure().await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
        })
    }

    async fn orbit_state(&self) -> Result<OrbitState, VehicleError> {
        let orbit = self
            .vessel
            .orbit()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        Ok(OrbitState {
            apoapsis: orbit.apoapsis().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            periapsis: orbit.periapsis().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            eccentricity: orbit.eccentricity().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            inclination: orbit.inclination().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
            time_to_apoapsis: orbit.time_to_apoapsis().await.map_err(|e| VehicleError::Communication(Box::new(e)))?,
        })
    }

    async fn resource_state(&self) -> Result<ResourceState, VehicleError> {
        let resources = self
            .vessel
            .resources()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        Ok(ResourceState {
            electric_charge: resources.amount("ElectricCharge").await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
            liquid_fuel: resources.amount("LiquidFuel").await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
            oxidizer: resources.amount("Oxidizer").await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
        })
    }

    async fn propulsion_state(&self) -> Result<PropulsionState, VehicleError> {
        let control = self
            .vessel
            .control()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        Ok(PropulsionState {
            mass: self.vessel.mass().await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
            available_thrust: self.vessel.available_thrust().await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
            thrust: self.vessel.thrust().await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
            throttle: control.throttle().await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
            current_stage: control.current_stage().await.map_err(|e| VehicleError::Communication(Box::new(e)))?.into(),
        })
    }
}

#[async_trait]
impl Vehicle for KrpcVehicle {
    async fn snapshot(&self) -> Result<VesselState, VehicleError> {
        let ut = self.universal_time().await?;

        let (
            flight,
            orbit,
            resources,
            propulsion,
        ) = tokio::try_join!(
            self.flight_state(),
            self.orbit_state(),
            self.resource_state(),
            self.propulsion_state(),
        )?;

        Ok(VesselState {
            ut,
            flight,
            orbit,
            resources,
            propulsion,
        })
    }
}

#[async_trait]
impl VehicleControl for KrpcVehicle {
    async fn set_throttle(&self, throttle: f32) -> Result<(), VehicleError> {
        let control = self
            .vessel
            .control()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        control.set_throttle(throttle).await.map_err(|e| VehicleError::Communication(Box::new(e)))?;
        Ok(())
    }

    async fn activate_next_stage(&self) -> Result<(), VehicleError> {
        let control = self
            .vessel
            .control()
            .await
            .map_err(|e| VehicleError::Communication(Box::new(e)))?;

        control.activate_next_stage().await.map_err(|e| VehicleError::Communication(Box::new(e)))?;
        Ok(())
    }
}
