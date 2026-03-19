use eframe::egui::{self, RichText};

use super::theme;

/// Data needed to render the bottom status bar.
pub struct StatusBarState<'a> {
    pub status: &'a str,
    pub uvc_wait_ms: f32,
    pub decode_ms: f32,
    pub rgb_pack_ms: f32,
    pub queue_ms: f32,
}

/// Draws the bottom status bar: status text + pipeline mini-summary.
pub fn draw(ui: &mut egui::Ui, state: &StatusBarState<'_>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(state.status)
                .monospace()
                .small()
                .color(theme::TEXT_SECONDARY),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!(
                    "w:{:.1}  d:{:.1}  r:{:.1}  q:{:.1}",
                    state.uvc_wait_ms, state.decode_ms, state.rgb_pack_ms, state.queue_ms,
                ))
                .monospace()
                .small()
                .color(theme::TEXT_SECONDARY),
            );
        });
    });
}
