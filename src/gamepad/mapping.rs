use crate::gamepad::types::{GamepadAxis, GamepadButton, GamepadSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlValueMode {
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlInputBinding {
    None,
    Axis(GamepadAxis),
    Button(GamepadButton),
    ButtonPair {
        negative: GamepadButton,
        positive: GamepadButton,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChannelBinding {
    pub input: ControlInputBinding,
    pub invert: bool,
    pub deadzone: f32,
    pub scale: f32,
    pub mode: ControlValueMode,
}

impl ChannelBinding {
    pub fn axis(axis: GamepadAxis) -> Self {
        Self {
            input: ControlInputBinding::Axis(axis),
            invert: false,
            deadzone: 0.0,
            scale: 1.0,
            mode: ControlValueMode::Signed,
        }
    }

    pub fn button(button: GamepadButton) -> Self {
        Self {
            input: ControlInputBinding::Button(button),
            invert: false,
            deadzone: 0.0,
            scale: 1.0,
            mode: ControlValueMode::Unsigned,
        }
    }

    pub fn button_pair(negative: GamepadButton, positive: GamepadButton) -> Self {
        Self {
            input: ControlInputBinding::ButtonPair { negative, positive },
            invert: false,
            deadzone: 0.0,
            scale: 1.0,
            mode: ControlValueMode::Signed,
        }
    }

    pub fn sample(&self, snapshot: &GamepadSnapshot) -> f32 {
        let raw_value = match self.input {
            ControlInputBinding::None => 0.0,
            ControlInputBinding::Axis(axis) => snapshot.axis(axis),
            ControlInputBinding::Button(button) => snapshot.button_value(button),
            ControlInputBinding::ButtonPair { negative, positive } => {
                snapshot.button_value(positive) - snapshot.button_value(negative)
            }
        };

        normalize_value(raw_value, self.invert, self.deadzone, self.scale, self.mode)
    }
}

impl Default for ChannelBinding {
    fn default() -> Self {
        Self {
            input: ControlInputBinding::None,
            invert: false,
            deadzone: 0.0,
            scale: 1.0,
            mode: ControlValueMode::Signed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamepadControlMapping {
    pub roll: ChannelBinding,
    pub pitch: ChannelBinding,
    pub yaw: ChannelBinding,
    pub throttle: ChannelBinding,
}

impl GamepadControlMapping {
    pub fn mode2_default() -> Self {
        Self {
            roll: ChannelBinding {
                deadzone: 0.05,
                ..ChannelBinding::axis(GamepadAxis::RightStickX)
            },
            pitch: ChannelBinding {
                invert: true,
                deadzone: 0.05,
                ..ChannelBinding::axis(GamepadAxis::RightStickY)
            },
            yaw: ChannelBinding {
                deadzone: 0.05,
                ..ChannelBinding::axis(GamepadAxis::LeftStickX)
            },
            throttle: ChannelBinding {
                invert: true,
                deadzone: 0.05,
                ..ChannelBinding::axis(GamepadAxis::LeftStickY)
            },
        }
    }

    pub fn trigger_throttle_default() -> Self {
        Self {
            throttle: ChannelBinding {
                deadzone: 0.02,
                ..ChannelBinding::button_pair(
                    GamepadButton::LeftTrigger2,
                    GamepadButton::RightTrigger2,
                )
            },
            ..Self::mode2_default()
        }
    }

    pub fn map_snapshot(&self, snapshot: &GamepadSnapshot) -> GamepadControlFrame {
        GamepadControlFrame {
            gamepad_id: snapshot.id,
            roll: self.roll.sample(snapshot),
            pitch: self.pitch.sample(snapshot),
            yaw: self.yaw.sample(snapshot),
            throttle: self.throttle.sample(snapshot),
        }
    }
}

impl Default for GamepadControlMapping {
    fn default() -> Self {
        Self::mode2_default()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GamepadControlFrame {
    pub gamepad_id: gilrs::GamepadId,
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub throttle: f32,
}

fn normalize_value(
    raw_value: f32,
    invert: bool,
    deadzone: f32,
    scale: f32,
    mode: ControlValueMode,
) -> f32 {
    let mut value = raw_value;
    if invert {
        value = -value;
    }

    if matches!(mode, ControlValueMode::Unsigned) {
        value = (value + 1.0) * 0.5;
    }

    let min = if matches!(mode, ControlValueMode::Unsigned) {
        0.0
    } else {
        -1.0
    };
    let max = 1.0;
    value = value.clamp(min, max);

    if matches!(mode, ControlValueMode::Unsigned) {
        if value <= deadzone {
            0.0
        } else {
            (value * scale).clamp(0.0, 1.0)
        }
    } else if value.abs() <= deadzone {
        0.0
    } else {
        (value * scale).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{normalize_value, ChannelBinding, ControlInputBinding, ControlValueMode};

    #[test]
    fn unsigned_binding_maps_signed_axis_to_zero_to_one() {
        let value = normalize_value(-0.8, true, 0.0, 1.0, ControlValueMode::Unsigned);
        assert!((value - 0.9).abs() < f32::EPSILON);
    }

    #[test]
    fn deadzone_zeroes_small_signed_values() {
        let value = normalize_value(0.03, false, 0.05, 1.0, ControlValueMode::Signed);
        assert_eq!(value, 0.0);
    }

    #[test]
    fn button_pair_binding_defaults_to_signed_mode() {
        let binding = ChannelBinding::button_pair(
            crate::gamepad::GamepadButton::LeftTrigger2,
            crate::gamepad::GamepadButton::RightTrigger2,
        );
        assert!(matches!(
            binding.input,
            ControlInputBinding::ButtonPair { .. }
        ));
        assert!(matches!(binding.mode, ControlValueMode::Signed));
    }
}
