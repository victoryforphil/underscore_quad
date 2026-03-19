use eframe::egui;

use super::theme;

/// Display scaling settings.
pub struct DisplaySettings {
    pub fit_window: bool,
    pub image_scale: f32,
}

impl DisplaySettings {
    pub fn new(fit_window: bool, scale: f32) -> Self {
        Self {
            fit_window,
            image_scale: scale.clamp(0.25, 4.0),
        }
    }

    /// Draw the fit-window checkbox and scale slider.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "Display");

        ui.checkbox(&mut self.fit_window, "Fit image to window");
        ui.add(
            egui::Slider::new(&mut self.image_scale, 0.25..=4.0)
                .text("Scale")
                .trailing_fill(true),
        );
    }
}
