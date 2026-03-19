use eframe::egui::{self, RichText};

use crate::gamepad::{GamepadControlFrame, GamepadSnapshot};

use super::theme;

pub struct GamepadPanelState<'a> {
    pub config_source: &'a str,
    pub snapshot: Option<&'a GamepadSnapshot>,
    pub control_frame: Option<&'a GamepadControlFrame>,
    pub error: Option<&'a str>,
}

pub fn draw(ui: &mut egui::Ui, state: &GamepadPanelState<'_>) {
    ui.label(
        RichText::new(format!("Config: {}", state.config_source))
            .small()
            .monospace()
            .color(theme::TEXT_SECONDARY),
    );

    if let Some(error) = state.error {
        ui.add_space(4.0);
        ui.label(RichText::new(error).small().color(theme::RED));
        return;
    }

    let Some(snapshot) = state.snapshot else {
        ui.add_space(4.0);
        ui.label(RichText::new("No active gamepad detected").weak().italics());
        return;
    };

    ui.add_space(4.0);
    ui.label(RichText::new(&snapshot.name).strong());
    ui.label(
        RichText::new(format!(
            "{}  connected:{}",
            snapshot.os_name, snapshot.is_connected
        ))
        .small()
        .monospace()
        .color(theme::TEXT_SECONDARY),
    );

    if let Some(control) = state.control_frame {
        ui.add_space(6.0);
        ui.label(RichText::new("Control Channels").small().strong());
        egui::Grid::new("gamepad_controls_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                value_row(ui, "Roll", control.roll);
                value_row(ui, "Pitch", control.pitch);
                value_row(ui, "Yaw", control.yaw);
                value_row(ui, "Throttle", control.throttle);
            });
    }

    ui.add_space(6.0);
    ui.label(RichText::new("Axes").small().strong());
    egui::Grid::new("gamepad_axes_grid")
        .num_columns(2)
        .spacing([8.0, 4.0])
        .show(ui, |ui| {
            for (axis, value) in &snapshot.axes {
                value_row(ui, &format!("{:?}", axis), *value);
            }
        });

    let active_buttons = snapshot
        .buttons
        .iter()
        .filter(|(_, value)| **value > 0.0)
        .collect::<Vec<_>>();

    ui.add_space(6.0);
    ui.label(RichText::new("Active Buttons").small().strong());
    if active_buttons.is_empty() {
        ui.label(RichText::new("none pressed").small().weak().italics());
    } else {
        egui::Grid::new("gamepad_buttons_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                for (button, value) in active_buttons {
                    value_row(ui, &format!("{:?}", button), *value);
                }
            });
    }
}

fn value_row(ui: &mut egui::Ui, label: &str, value: f32) {
    ui.label(RichText::new(label).small().color(theme::TEXT_SECONDARY));
    ui.label(
        RichText::new(format!("{value:+.3}"))
            .small()
            .monospace()
            .color(value_color(value)),
    );
    ui.end_row();
}

fn value_color(value: f32) -> egui::Color32 {
    let magnitude = value.abs();
    if magnitude >= 0.75 {
        theme::ACCENT_HOVER
    } else if magnitude >= 0.2 {
        theme::ACCENT
    } else {
        theme::TEXT_PRIMARY
    }
}
