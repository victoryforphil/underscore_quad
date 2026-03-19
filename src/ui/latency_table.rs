use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};

use super::latency::LatencyTracker;
use super::theme;

/// Draws a striped latency table showing 1-second peak values per pipeline stage.
pub fn draw(ui: &mut egui::Ui, tracker: &LatencyTracker) {
    theme::section_heading(ui, "Latency (1 s peak)");

    let available_width = ui.available_width();
    TableBuilder::new(ui)
        .id_salt("latency_table")
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::exact(available_width * 0.55))
        .column(Column::remainder())
        .header(18.0, |mut header| {
            header.col(|ui| {
                ui.label(RichText::new("Stage").strong().small());
            });
            header.col(|ui| {
                ui.label(RichText::new("ms").strong().small());
            });
        })
        .body(|mut body| {
            let rows: [(&str, f32); 5] = [
                ("Total", tracker.total_ms),
                ("UVC wait", tracker.uvc_wait_ms),
                ("Decode", tracker.decode_ms),
                ("RGB pack", tracker.rgb_pack_ms),
                ("Queue/UI", tracker.queue_ms),
            ];
            for (label, ms) in rows {
                body.row(18.0, |mut row| {
                    row.col(|ui| {
                        ui.label(RichText::new(label).weak().monospace());
                    });
                    row.col(|ui| {
                        let color = theme::latency_color(ms);
                        ui.label(RichText::new(format!("{ms:.2}")).monospace().color(color));
                    });
                });
            }
        });
}
