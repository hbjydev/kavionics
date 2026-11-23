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
            .with_context(|| {
                format!("Failed to compile condition expression: {source}")
            })?;

        Ok(Self {
            source: source.to_owned(),
            program,
        })
    }

    pub fn evaluate(
        &self,
        state: &VesselState,
    ) -> anyhow::Result<bool> {
        let mut context = Context::default();
        context.add_variable("vehicle", state)?;

        let result = self.program.execute(&context)
            .with_context(|| {
                format!("Failed to evaluate condition expression: {}", self.source)
            })?;

        match result {
            cel::Value::Bool(b) => Ok(b),
            _ => anyhow::bail!("Condition expression did not evaluate to a boolean: {}", self.source),
        }
    }
}
