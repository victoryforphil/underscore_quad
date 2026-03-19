use eframe::egui::{self, RichText};

use super::theme;

/// Data needed to render the top status bar.
pub struct TopBarState {
    pub is_streaming: bool,
    pub fps: f32,
    pub total_delay_ms: f32,
    pub width: u32,
    pub height: u32,
    pub frame_count: u64,
}

/// Draws the top bar: app name, LIVE/IDLE badge, quick stats, resolution.
pub fn draw(ui: &mut egui::Ui, state: &TopBarState) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("_quad")
                .size(16.0)
                .strong()
                .color(theme::ACCENT),
        );
        ui.separator();

        if state.is_streaming {
            theme::badge(ui, "LIVE", theme::GREEN);
        } else {
            theme::badge(ui, "IDLE", theme::TEXT_SECONDARY);
        }

        ui.separator();

        ui.label(
            RichText::new(format!("{:.0} fps", state.fps))
                .monospace()
                .color(fps_color(state.fps)),
        );

        ui.separator();

        ui.label(
            RichText::new(format!("{:.1} ms", state.total_delay_ms))
                .monospace()
                .color(theme::latency_color(state.total_delay_ms)),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{}x{}", state.width, state.height))
                    .monospace()
                    .weak(),
            );
            ui.label(
                RichText::new(format!("#{}", state.frame_count))
                    .monospace()
                    .weak(),
            );
        });
    });
}

fn fps_color(fps: f32) -> egui::Color32 {
    if fps >= 24.0 {
        theme::GREEN
    } else if fps >= 10.0 {
        theme::YELLOW
    } else {
        theme::RED
    }
}
