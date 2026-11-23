use anyhow::{Context as _, Result};
use cel::{Context, Program};
use kavionics_core::vessel_state::VesselState;

pub struct Condition {
    pub source: String,
    program: Program,
}

impl Condition {
    pub fn compile(source: &str) -> Result<Self> {
        let program = Program::compile(source)
            .with_context(|| format!("Failed to compile condition expression: {source}"))?;

        Ok(Self {
            source: source.to_owned(),
            program,
        })
    }

    pub fn evaluate(&self, state: &VesselState) -> anyhow::Result<bool> {
        let mut context = Context::default();
        context.add_variable("vehicle", state)?;

        let result = self
            .program
            .execute(&context)
            .with_context(|| format!("Failed to evaluate condition expression: {}", self.source))?;

        match result {
            cel::Value::Bool(b) => Ok(b),
            _ => anyhow::bail!(
                "Condition expression did not evaluate to a boolean: {}",
                self.source
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use kavionics_core::vessel_state::{
        FlightState, OrbitState, PropulsionState, ResourceState, VesselState,
    };

    use crate::condition::Condition;

    fn fixture() -> VesselState {
        VesselState {
            ut: 1000.0,
            flight: FlightState {
                altitude_asl: 1000.0,
                altitude_agl: 500.0,
                vertical_speed: 10.0,
                surface_speed: 100.0,
                dynamic_pressure: 50.0,
            },

            orbit: OrbitState {
                apoapsis: 10000.0,
                periapsis: 5000.0,
                eccentricity: 0.1,
                inclination: 45.0,
                time_to_apoapsis: 200.0,
            },

            propulsion: PropulsionState {
                mass: 1000.0,
                available_thrust: 50000.0,
                thrust: 40000.0,
                throttle: 0.8,
                current_stage: 1,
            },

            resources: ResourceState {
                electric_charge: 100.0,
                liquid_fuel: 50.0,
                oxidizer: 50.0,
            },
        }
    }

    #[test]
    fn test_condition_works_as_expected() {
        let fixture = fixture();
        let condition = Condition::compile("vehicle.flight.altitude_asl > 500.0").unwrap();
        let result = condition.evaluate(&fixture).unwrap();
        assert!(result);
    }

    #[test]
    fn test_condition_works_as_expected_2() {
        let fixture = fixture();
        let condition = Condition::compile("vehicle.flight.altitude_asl < 500.0").unwrap();
        let result = condition.evaluate(&fixture).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_condition_works_as_expected_3() {
        let fixture = fixture();
        let condition = Condition::compile("vehicle.resources.electric_charge > 500.0").unwrap();
        let result = condition.evaluate(&fixture).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_condition_fails_to_evaluate_bad_condition() {
        let fixture = fixture();
        let condition = Condition::compile("vehicle.resources.nonexistent").unwrap();
        let result = condition.evaluate(&fixture);
        assert!(result.is_err());
    }

    #[test]
    fn test_condition_fails_to_evaluate_non_bool() {
        let fixture = fixture();
        let condition = Condition::compile("vehicle.resources.liquid_fuel + 1").unwrap();
        let result = condition.evaluate(&fixture);
        assert!(result.is_err());
    }

    #[test]
    fn test_condition_fails_to_compile_invalid_cel() {
        let condition = Condition::compile("vehicle.");
        assert!(condition.is_err());
    }
}
