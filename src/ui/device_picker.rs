use eframe::egui::{self, RichText};

use crate::camera::{list_video_devices, VideoDevice};

use super::theme;

/// Device selection state and logic.
pub struct DevicePicker {
    pub devices: Vec<VideoDevice>,
    pub selected: String,
}

impl DevicePicker {
    pub fn new(devices: Vec<VideoDevice>, selected: String) -> Self {
        Self { devices, selected }
    }

    /// Re-scan `/dev/video*` devices, keeping the current selection if still valid.
    pub fn refresh(&mut self) {
        self.devices = list_video_devices().unwrap_or_default();
        if self.devices.is_empty() {
            return;
        }
        let still_valid = self.devices.iter().any(|d| d.path == self.selected);
        if !still_valid {
            self.selected = self.devices[0].path.clone();
        }
    }

    /// Draw the device combo box and refresh button.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        theme::section_heading(ui, "Device");

        if self.devices.is_empty() {
            ui.label(
                RichText::new("No /dev/video* devices found")
                    .weak()
                    .italics(),
            );
        } else {
            egui::ComboBox::from_id_salt("device_combo")
                .width(ui.available_width() - 8.0)
                .selected_text(&self.selected)
                .show_ui(ui, |ui| {
                    for device in &self.devices {
                        let label = format!("{} ({})", device.path, device.name);
                        ui.selectable_value(&mut self.selected, device.path.clone(), label);
                    }
                });
        }

        ui.add_space(2.0);
        if ui
            .small_button(
                RichText::new("Refresh Devices")
                    .small()
                    .color(theme::ACCENT),
            )
            .clicked()
        {
            self.refresh();
        }
    }
}
