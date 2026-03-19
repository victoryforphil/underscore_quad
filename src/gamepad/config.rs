use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::gamepad::{
    ChannelBinding, GamepadCollectorOptions, GamepadControlMapping, GamepadWorkerOptions,
};

#[derive(Debug, Clone)]
pub struct ResolvedGamepadConfig {
    pub path: Option<PathBuf>,
    pub collector: GamepadCollectorOptions,
    pub control_mapping: GamepadControlMapping,
    pub worker: GamepadWorkerOptions,
}

impl ResolvedGamepadConfig {
    pub fn from_optional_path(path: Option<&str>) -> Result<Self> {
        match path {
            Some(path) => GamepadConfig::load_from_path(path).map(|config| config.resolve()),
            None => Ok(GamepadConfig::default().resolve()),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GamepadConfig {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(default)]
    pub collector: GamepadCollectorConfig,
    #[serde(default)]
    pub worker: GamepadWorkerConfig,
    #[serde(default)]
    pub mapping: GamepadMappingConfig,
}

impl GamepadConfig {
    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .with_context(|| format!("failed to read gamepad config: {}", path.display()))?;
        let mut config: Self = toml::from_str(&contents)
            .with_context(|| format!("failed to parse gamepad config: {}", path.display()))?;
        config.path = Some(path.to_path_buf());
        Ok(config)
    }

    pub fn resolve(&self) -> ResolvedGamepadConfig {
        let collector = self.collector.resolve();
        let control_mapping = self.mapping.resolve();
        let worker = self.worker.resolve(collector, control_mapping);

        ResolvedGamepadConfig {
            path: self.path.clone(),
            collector,
            control_mapping,
            worker,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct GamepadCollectorConfig {
    pub axis_deadzone: Option<f32>,
    pub trigger_deadzone: Option<f32>,
}

impl GamepadCollectorConfig {
    pub fn resolve(&self) -> GamepadCollectorOptions {
        let mut options = GamepadCollectorOptions::default();
        if let Some(axis_deadzone) = self.axis_deadzone {
            options.axis_deadzone = axis_deadzone;
        }
        if let Some(trigger_deadzone) = self.trigger_deadzone {
            options.trigger_deadzone = trigger_deadzone;
        }
        options
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
pub struct GamepadWorkerConfig {
    pub poll_timeout_ms: Option<u64>,
}

impl GamepadWorkerConfig {
    pub fn resolve(
        &self,
        collector: GamepadCollectorOptions,
        control_mapping: GamepadControlMapping,
    ) -> GamepadWorkerOptions {
        let mut options = GamepadWorkerOptions::default();
        options.collector = collector;
        options.control_mapping = Some(control_mapping);
        if let Some(poll_timeout_ms) = self.poll_timeout_ms {
            options.poll_timeout = Duration::from_millis(poll_timeout_ms.max(1));
        }
        options
    }
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum GamepadConfigMappingPreset {
    #[default]
    Mode2Default,
    TriggerThrottleDefault,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GamepadMappingConfig {
    pub preset: Option<GamepadConfigMappingPreset>,
    pub roll: Option<ChannelBinding>,
    pub pitch: Option<ChannelBinding>,
    pub yaw: Option<ChannelBinding>,
    pub throttle: Option<ChannelBinding>,
}

impl GamepadMappingConfig {
    pub fn resolve(&self) -> GamepadControlMapping {
        let preset = self.preset.unwrap_or_default();
        let mut mapping = match preset {
            GamepadConfigMappingPreset::Mode2Default => GamepadControlMapping::mode2_default(),
            GamepadConfigMappingPreset::TriggerThrottleDefault => {
                GamepadControlMapping::trigger_throttle_default()
            }
        };

        if let Some(roll) = self.roll {
            mapping.roll = roll;
        }
        if let Some(pitch) = self.pitch {
            mapping.pitch = pitch;
        }
        if let Some(yaw) = self.yaw {
            mapping.yaw = yaw;
        }
        if let Some(throttle) = self.throttle {
            mapping.throttle = throttle;
        }

        mapping
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{GamepadConfig, GamepadConfigMappingPreset};
    use crate::gamepad::{ControlInputBinding, ControlValueMode, GamepadAxis, GamepadButton};

    #[test]
    fn parses_toml_gamepad_config() {
        let config: GamepadConfig = toml::from_str(
            r#"
            [collector]
            axis_deadzone = 0.1
            trigger_deadzone = 0.02

            [worker]
            poll_timeout_ms = 12

            [mapping]
            preset = "trigger-throttle-default"

            [mapping.roll]
            kind = "axis"
            axis = "right-stick-x"
            deadzone = 0.04
            scale = 0.8
            invert = false
            mode = "signed"

            [mapping.throttle]
            kind = "button-pair"
            negative = "left-trigger2"
            positive = "right-trigger2"
            deadzone = 0.03
            scale = 1.0
            invert = false
            mode = "signed"
            "#,
        )
        .expect("config should parse");

        assert!(matches!(
            config.mapping.preset,
            Some(GamepadConfigMappingPreset::TriggerThrottleDefault)
        ));

        let roll = config.mapping.roll.expect("roll override");
        assert!(matches!(
            roll.input,
            ControlInputBinding::Axis {
                axis: GamepadAxis::RightStickX
            }
        ));
        assert!(matches!(roll.mode, ControlValueMode::Signed));

        let throttle = config.mapping.throttle.expect("throttle override");
        assert!(matches!(
            throttle.input,
            ControlInputBinding::ButtonPair {
                negative: GamepadButton::LeftTrigger2,
                positive: GamepadButton::RightTrigger2
            }
        ));

        let resolved = config.resolve();
        assert!((resolved.collector.axis_deadzone - 0.1).abs() < f32::EPSILON);
        assert_eq!(resolved.worker.poll_timeout, Duration::from_millis(12));
    }
}
