use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::condition::Condition;

#[derive(Debug, Clone, Deserialize)]
pub struct MissionConfig {
    pub name: String,
    pub phases: Vec<MissionPhase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MissionPhase {
    pub name: String,
    pub actions: Vec<MissionAction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum MissionAction {
    Stage,
    Wait { seconds: f64 },
    WaitUntil { condition: String },
    SetThrottle { throttle: f32 },
}

impl MissionConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let value = pklr::eval_to_json(path)?;
        let config: Self = serde_json::from_value(value)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        for phase in &self.phases {
            for (idx, action) in phase.actions.iter().enumerate() {
                match action {
                    MissionAction::Wait { seconds } => {
                        anyhow::ensure!(
                            seconds.is_finite() && *seconds >= 0.0,
                            "wait seconds must be non-negative and finite in action {}:{}",
                            phase.name, idx,
                        );
                    },

                    MissionAction::WaitUntil { condition } => {
                        Condition::compile(condition)
                            .with_context(|| {
                                format!(
                                    "target altitude must be a valid CEL expression in action {}:{}",
                                    phase.name, idx,
                                )
                            })?;
                    },

                    MissionAction::SetThrottle { throttle } => {
                        anyhow::ensure!(
                            (0.0..=1.0).contains(throttle),
                            "throttle must be between 0 and 1 in action {}:{}",
                            phase.name, idx,
                        );
                    },

                    _ => ()
                }
            }
        }

        Ok(())
    }
}
