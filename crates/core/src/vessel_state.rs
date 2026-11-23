use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct VesselState {
    /// KSP universal time, in seconds.
    pub ut: f64,

    pub flight: FlightState,
    pub orbit: OrbitState,
    pub resources: ResourceState,
    pub propulsion: PropulsionState,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlightState {
    /// Altitude above sea level, in meters.
    pub altitude_asl: f64,

    /// Altitude above terrain, in meters.
    pub altitude_agl: f64,

    /// Vertical speed, in meters per second.
    pub vertical_speed: f64,

    /// Surface-relative speed, in meters per second.
    pub surface_speed: f64,

    /// Dynamic pressure, in Pascals.
    pub dynamic_pressure: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct OrbitState {
    pub apoapsis: f64,
    pub periapsis: f64,
    pub eccentricity: f64,
    pub inclination: f64,
    pub time_to_apoapsis: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceState {
    pub electric_charge: f64,
    pub liquid_fuel: f64,
    pub oxidizer: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PropulsionState {
    /// Vessel mass, in kilograms.
    pub mass: f64,

    /// Current available thrust, in newtons.
    pub available_thrust: f64,

    /// Current actual thrust, in newtons.
    pub thrust: f64,

    /// Current throttle command, 0.0–1.0.
    pub throttle: f64,

    pub current_stage: i32,
}
