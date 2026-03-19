use eframe::egui::{self, RichText};

use super::worker::FrameMessage;

/// Manages the camera texture and renders the video feed.
#[derive(Default)]
pub struct VideoView {
    texture: Option<egui::TextureHandle>,
}

impl VideoView {
    /// Clear the current texture (e.g. on capture restart).
    pub fn reset(&mut self) {
        self.texture = None;
    }

    /// Upload a new frame to the GPU texture.
    pub fn update(&mut self, ctx: &egui::Context, frame: FrameMessage) {
        let image = egui::ColorImage::from_rgb([frame.width, frame.height], &frame.rgb);
        match &mut self.texture {
            Some(tex) => tex.set(image, egui::TextureOptions::LINEAR),
            None => {
                self.texture =
                    Some(ctx.load_texture("camera_frame", image, egui::TextureOptions::LINEAR));
            }
        }
    }

    /// Draw the video image or a "waiting" spinner.
    pub fn draw(&self, ui: &mut egui::Ui, width: u32, height: u32, fit: bool, scale: f32) {
        let Some(texture) = &self.texture else {
            draw_waiting(ui);
            return;
        };

        let base = egui::vec2(width as f32, height as f32);
        let available = ui.available_size();
        let fit_scale = if fit {
            let sx = available.x / base.x;
            let sy = available.y / base.y;
            sx.min(sy)
        } else {
            1.0
        };
        let effective = (fit_scale * scale).max(0.05);
        let size = egui::vec2(base.x * effective, base.y * effective);

        ui.centered_and_justified(|ui| {
            ui.image((texture.id(), size));
        });
    }
}

fn draw_waiting(ui: &mut egui::Ui) {
    ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(ui.available_height() / 3.0);
            ui.label(
                RichText::new("Waiting for camera frames...")
                    .size(18.0)
                    .weak()
                    .italics(),
            );
            ui.add_space(8.0);
            ui.spinner();
        });
    });
}
