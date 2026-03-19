use std::collections::HashMap;
use std::time::Duration;

use anyhow::{anyhow, Result};
use gilrs::{Event, EventType, GamepadId, Gilrs};

use crate::gamepad::types::{
    GamepadAxis, GamepadButton, GamepadEvent, GamepadEventKind, GamepadSnapshot,
};

#[derive(Debug, Clone, Copy)]
pub struct GamepadCollectorOptions {
    pub axis_deadzone: f32,
    pub trigger_deadzone: f32,
}

impl Default for GamepadCollectorOptions {
    fn default() -> Self {
        Self {
            axis_deadzone: 0.15,
            trigger_deadzone: 0.05,
        }
    }
}

pub struct GamepadCollector {
    gilrs: Gilrs,
    options: GamepadCollectorOptions,
    snapshots: HashMap<GamepadId, GamepadSnapshot>,
    active_gamepad: Option<GamepadId>,
}

impl GamepadCollector {
    pub fn new() -> Result<Self> {
        Self::with_options(GamepadCollectorOptions::default())
    }

    pub fn with_options(options: GamepadCollectorOptions) -> Result<Self> {
        let gilrs = Gilrs::new().map_err(|err| {
            anyhow!(
                "failed to initialize gamepad backend; on Linux make sure libudev is available: {err}"
            )
        })?;

        let mut collector = Self {
            gilrs,
            options,
            snapshots: HashMap::new(),
            active_gamepad: None,
        };

        let ids = collector
            .gilrs
            .gamepads()
            .map(|(id, _)| id)
            .collect::<Vec<_>>();

        for id in ids {
            collector.refresh_snapshot(id);
            collector.active_gamepad.get_or_insert(id);
        }

        Ok(collector)
    }

    pub fn poll(&mut self) -> Vec<GamepadEvent> {
        let mut events = Vec::new();

        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            self.active_gamepad = Some(id);

            match event {
                EventType::Connected => {
                    self.refresh_snapshot(id);
                    events.push(GamepadEvent {
                        gamepad_id: id,
                        kind: GamepadEventKind::Connected,
                    });
                }
                EventType::Disconnected => {
                    if let Some(snapshot) = self.snapshots.get_mut(&id) {
                        snapshot.is_connected = false;
                    }
                    events.push(GamepadEvent {
                        gamepad_id: id,
                        kind: GamepadEventKind::Disconnected,
                    });
                }
                EventType::AxisChanged(axis, value, _) => {
                    if let Some(axis) = GamepadAxis::from_gilrs(axis) {
                        let value = self.filter_axis_value(axis, value);
                        self.refresh_snapshot(id);
                        if let Some(snapshot) = self.snapshots.get_mut(&id) {
                            snapshot.axes.insert(axis, value);
                        }
                        events.push(GamepadEvent {
                            gamepad_id: id,
                            kind: GamepadEventKind::AxisChanged { axis, value },
                        });
                    }
                }
                EventType::ButtonPressed(button, _) => {
                    if let Some(button) = GamepadButton::from_gilrs(button) {
                        self.set_button_value(id, button, 1.0);
                        events.push(GamepadEvent {
                            gamepad_id: id,
                            kind: GamepadEventKind::ButtonPressed { button },
                        });
                    }
                }
                EventType::ButtonReleased(button, _) => {
                    if let Some(button) = GamepadButton::from_gilrs(button) {
                        self.set_button_value(id, button, 0.0);
                        events.push(GamepadEvent {
                            gamepad_id: id,
                            kind: GamepadEventKind::ButtonReleased { button },
                        });
                    }
                }
                EventType::ButtonChanged(button, value, _) => {
                    if let Some(button) = GamepadButton::from_gilrs(button) {
                        let value = self.filter_button_value(button, value);
                        self.set_button_value(id, button, value);
                        events.push(GamepadEvent {
                            gamepad_id: id,
                            kind: GamepadEventKind::ButtonChanged { button, value },
                        });
                    }
                }
                EventType::Dropped | EventType::ButtonRepeated(_, _) => {}
                _ => {}
            }
        }

        self.gilrs.inc();
        events
    }

    pub fn poll_blocking(&mut self, timeout: Option<Duration>) -> Vec<GamepadEvent> {
        match self.gilrs.next_event_blocking(timeout) {
            Some(event) => {
                self.gilrs.insert_event(event);
                self.poll()
            }
            None => Vec::new(),
        }
    }

    pub fn active_gamepad(&self) -> Option<GamepadId> {
        self.active_gamepad.or_else(|| {
            self.snapshots
                .iter()
                .find_map(|(id, snapshot)| snapshot.is_connected.then_some(*id))
        })
    }

    pub fn snapshot(&self, id: GamepadId) -> Option<&GamepadSnapshot> {
        self.snapshots.get(&id)
    }

    pub fn active_snapshot(&self) -> Option<&GamepadSnapshot> {
        self.active_gamepad().and_then(|id| self.snapshot(id))
    }

    pub fn snapshots(&self) -> impl Iterator<Item = &GamepadSnapshot> {
        self.snapshots.values()
    }

    fn set_button_value(&mut self, id: GamepadId, button: GamepadButton, value: f32) {
        self.refresh_snapshot(id);
        if let Some(snapshot) = self.snapshots.get_mut(&id) {
            snapshot.buttons.insert(button, value);
        }
    }

    fn refresh_snapshot(&mut self, id: GamepadId) {
        let gamepad = self.gilrs.gamepad(id);
        let mut snapshot = GamepadSnapshot::new(
            id,
            gamepad.name().to_string(),
            gamepad.os_name().to_string(),
        );
        snapshot.vendor_id = gamepad.vendor_id();
        snapshot.product_id = gamepad.product_id();
        snapshot.is_connected = gamepad.is_connected();

        for axis in GamepadAxis::ALL {
            let value = gamepad
                .axis_data(axis.as_gilrs())
                .map(|data| data.value())
                .unwrap_or_default();
            let value = self.filter_axis_value(axis, value);
            snapshot.axes.insert(axis, value);
        }

        for button in GamepadButton::ALL {
            let value = gamepad
                .button_data(button.as_gilrs())
                .map(|data| data.value())
                .unwrap_or_else(|| {
                    if gamepad.is_pressed(button.as_gilrs()) {
                        1.0
                    } else {
                        0.0
                    }
                });
            let value = self.filter_button_value(button, value);
            snapshot.buttons.insert(button, value);
        }

        self.snapshots.insert(id, snapshot);
    }

    fn filter_axis_value(&self, axis: GamepadAxis, value: f32) -> f32 {
        let threshold = match axis {
            GamepadAxis::LeftZ | GamepadAxis::RightZ => self.options.trigger_deadzone,
            _ => self.options.axis_deadzone,
        };

        if value.abs() < threshold {
            0.0
        } else {
            value.clamp(-1.0, 1.0)
        }
    }

    fn filter_button_value(&self, button: GamepadButton, value: f32) -> f32 {
        match button {
            GamepadButton::LeftTrigger2 | GamepadButton::RightTrigger2 => {
                if value < self.options.trigger_deadzone {
                    0.0
                } else {
                    value.clamp(0.0, 1.0)
                }
            }
            _ => value.clamp(0.0, 1.0),
        }
    }
}
