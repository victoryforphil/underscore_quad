use eframe::egui::{self, Color32, RichText};

use super::theme;

/// Capture resolution / FPS settings.
///
/// Returns `true` from [`draw`] when the user clicks Apply.
pub struct CaptureSettings {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

impl CaptureSettings {
    pub fn new(width: u32, height: u32, fps: u32) -> Self {
        Self { width, height, fps }
    }

    /// Draw the capture settings grid + Apply button.
    ///
    /// Returns `true` when the user clicks Apply.
    pub fn draw(&mut self, ui: &mut egui::Ui) -> bool {
        theme::section_heading(ui, "Capture");

        egui::Grid::new("capture_grid")
            .num_columns(2)
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Width").weak());
                ui.add(egui::Slider::new(&mut self.width, 160..=1920).trailing_fill(true));
                ui.end_row();

                ui.label(RichText::new("Height").weak());
                ui.add(egui::Slider::new(&mut self.height, 120..=1080).trailing_fill(true));
                ui.end_row();

                ui.label(RichText::new("Target FPS").weak());
                ui.add(egui::Slider::new(&mut self.fps, 5..=120).trailing_fill(true));
                ui.end_row();
            });

        ui.add_space(4.0);

        ui.add(
            egui::Button::new(RichText::new("Apply").strong().color(Color32::WHITE))
                .fill(theme::ACCENT.gamma_multiply(0.8))
                .corner_radius(egui::CornerRadius::same(4)),
        )
        .clicked()
    }
}
