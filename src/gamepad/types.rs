use std::collections::BTreeMap;

use gilrs::{Axis, Button, GamepadId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    LeftZ,
    RightStickX,
    RightStickY,
    RightZ,
    DPadX,
    DPadY,
}

impl GamepadAxis {
    pub const ALL: [Self; 8] = [
        Self::LeftStickX,
        Self::LeftStickY,
        Self::LeftZ,
        Self::RightStickX,
        Self::RightStickY,
        Self::RightZ,
        Self::DPadX,
        Self::DPadY,
    ];

    pub fn from_gilrs(axis: Axis) -> Option<Self> {
        match axis {
            Axis::LeftStickX => Some(Self::LeftStickX),
            Axis::LeftStickY => Some(Self::LeftStickY),
            Axis::LeftZ => Some(Self::LeftZ),
            Axis::RightStickX => Some(Self::RightStickX),
            Axis::RightStickY => Some(Self::RightStickY),
            Axis::RightZ => Some(Self::RightZ),
            Axis::DPadX => Some(Self::DPadX),
            Axis::DPadY => Some(Self::DPadY),
            Axis::Unknown => None,
        }
    }

    pub fn as_gilrs(self) -> Axis {
        match self {
            Self::LeftStickX => Axis::LeftStickX,
            Self::LeftStickY => Axis::LeftStickY,
            Self::LeftZ => Axis::LeftZ,
            Self::RightStickX => Axis::RightStickX,
            Self::RightStickY => Axis::RightStickY,
            Self::RightZ => Axis::RightZ,
            Self::DPadX => Axis::DPadX,
            Self::DPadY => Axis::DPadY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    C,
    Z,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
}

impl GamepadButton {
    pub const ALL: [Self; 19] = [
        Self::South,
        Self::East,
        Self::North,
        Self::West,
        Self::C,
        Self::Z,
        Self::LeftTrigger,
        Self::LeftTrigger2,
        Self::RightTrigger,
        Self::RightTrigger2,
        Self::Select,
        Self::Start,
        Self::Mode,
        Self::LeftThumb,
        Self::RightThumb,
        Self::DPadUp,
        Self::DPadDown,
        Self::DPadLeft,
        Self::DPadRight,
    ];

    pub fn from_gilrs(button: Button) -> Option<Self> {
        match button {
            Button::South => Some(Self::South),
            Button::East => Some(Self::East),
            Button::North => Some(Self::North),
            Button::West => Some(Self::West),
            Button::C => Some(Self::C),
            Button::Z => Some(Self::Z),
            Button::LeftTrigger => Some(Self::LeftTrigger),
            Button::LeftTrigger2 => Some(Self::LeftTrigger2),
            Button::RightTrigger => Some(Self::RightTrigger),
            Button::RightTrigger2 => Some(Self::RightTrigger2),
            Button::Select => Some(Self::Select),
            Button::Start => Some(Self::Start),
            Button::Mode => Some(Self::Mode),
            Button::LeftThumb => Some(Self::LeftThumb),
            Button::RightThumb => Some(Self::RightThumb),
            Button::DPadUp => Some(Self::DPadUp),
            Button::DPadDown => Some(Self::DPadDown),
            Button::DPadLeft => Some(Self::DPadLeft),
            Button::DPadRight => Some(Self::DPadRight),
            Button::Unknown => None,
        }
    }

    pub fn as_gilrs(self) -> Button {
        match self {
            Self::South => Button::South,
            Self::East => Button::East,
            Self::North => Button::North,
            Self::West => Button::West,
            Self::C => Button::C,
            Self::Z => Button::Z,
            Self::LeftTrigger => Button::LeftTrigger,
            Self::LeftTrigger2 => Button::LeftTrigger2,
            Self::RightTrigger => Button::RightTrigger,
            Self::RightTrigger2 => Button::RightTrigger2,
            Self::Select => Button::Select,
            Self::Start => Button::Start,
            Self::Mode => Button::Mode,
            Self::LeftThumb => Button::LeftThumb,
            Self::RightThumb => Button::RightThumb,
            Self::DPadUp => Button::DPadUp,
            Self::DPadDown => Button::DPadDown,
            Self::DPadLeft => Button::DPadLeft,
            Self::DPadRight => Button::DPadRight,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GamepadSnapshot {
    pub id: GamepadId,
    pub name: String,
    pub os_name: String,
    pub vendor_id: Option<u16>,
    pub product_id: Option<u16>,
    pub is_connected: bool,
    pub axes: BTreeMap<GamepadAxis, f32>,
    pub buttons: BTreeMap<GamepadButton, f32>,
}

impl GamepadSnapshot {
    pub fn new(id: GamepadId, name: String, os_name: String) -> Self {
        Self {
            id,
            name,
            os_name,
            vendor_id: None,
            product_id: None,
            is_connected: true,
            axes: BTreeMap::new(),
            buttons: BTreeMap::new(),
        }
    }

    pub fn axis(&self, axis: GamepadAxis) -> f32 {
        self.axes.get(&axis).copied().unwrap_or_default()
    }

    pub fn button_value(&self, button: GamepadButton) -> f32 {
        self.buttons.get(&button).copied().unwrap_or_default()
    }

    pub fn is_pressed(&self, button: GamepadButton) -> bool {
        self.button_value(button) > 0.5
    }
}

#[derive(Debug, Clone)]
pub struct GamepadEvent {
    pub gamepad_id: GamepadId,
    pub kind: GamepadEventKind,
}

#[derive(Debug, Clone)]
pub enum GamepadEventKind {
    Connected,
    Disconnected,
    AxisChanged { axis: GamepadAxis, value: f32 },
    ButtonPressed { button: GamepadButton },
    ButtonReleased { button: GamepadButton },
    ButtonChanged { button: GamepadButton, value: f32 },
}

#[cfg(test)]
mod tests {
    use super::{GamepadAxis, GamepadButton};

    #[test]
    fn all_axes_round_trip_to_gilrs() {
        for axis in GamepadAxis::ALL {
            assert_eq!(GamepadAxis::from_gilrs(axis.as_gilrs()), Some(axis));
        }
    }

    #[test]
    fn all_buttons_round_trip_to_gilrs() {
        for button in GamepadButton::ALL {
            assert_eq!(GamepadButton::from_gilrs(button.as_gilrs()), Some(button));
        }
    }
}
