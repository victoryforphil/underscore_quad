use anyhow::Result;
use eframe::egui;
use std::time::Instant;

use crate::camera::{list_video_devices, VideoDevice};
use crate::cli::resolve_device;
use crate::Cli;

use super::worker::{CaptureWorker, FrameMessage};

pub struct CameraApp {
    devices: Vec<VideoDevice>,
    selected_device: String,
    width: u32,
    height: u32,
    fps: u32,
    fit_window: bool,
    image_scale: f32,
    worker: Option<CaptureWorker>,
    texture: Option<egui::TextureHandle>,
    latest_fps: f32,
    latest_total_delay_ms: f32,
    latest_queue_delay_ms: f32,
    latest_uvc_wait_ms: f32,
    latest_decode_ms: f32,
    latest_rgb_pack_ms: f32,
    status: String,
}

impl CameraApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, cli: Cli) -> Self {
        let devices = list_video_devices().unwrap_or_default();
        let selected = resolve_device(&cli.device).unwrap_or(cli.device);

        let mut app = Self {
            devices,
            selected_device: selected,
            width: cli.width,
            height: cli.height,
            fps: cli.fps,
            fit_window: cli.fit_window,
            image_scale: cli.scale.clamp(0.25, 4.0),
            worker: None,
            texture: None,
            latest_fps: 0.0,
            latest_total_delay_ms: 0.0,
            latest_queue_delay_ms: 0.0,
            latest_uvc_wait_ms: 0.0,
            latest_decode_ms: 0.0,
            latest_rgb_pack_ms: 0.0,
            status: "idle".to_string(),
        };

        if let Err(err) = app.restart_capture() {
            app.status = format!("capture start failed: {err}");
        }
        app
    }

    fn restart_capture(&mut self) -> Result<()> {
        if let Some(worker) = &mut self.worker {
            worker.stop();
        }
        self.worker = None;
        self.texture = None;

        let worker = CaptureWorker::start(
            self.selected_device.clone(),
            self.width,
            self.height,
            self.fps,
        )?;

        self.worker = Some(worker);
        self.status = format!(
            "streaming {} at {}x{}@{}",
            self.selected_device, self.width, self.height, self.fps
        );
        Ok(())
    }

    fn refresh_devices(&mut self) {
        self.devices = list_video_devices().unwrap_or_default();
        if self.devices.is_empty() {
            return;
        }

        let selected_exists = self.devices.iter().any(|d| d.path == self.selected_device);
        if !selected_exists {
            self.selected_device = self.devices[0].path.clone();
        }
    }

    fn consume_latest_frame(&mut self, ctx: &egui::Context) {
        let frame = self.worker.as_ref().and_then(CaptureWorker::latest_frame);
        if let Some(frame) = frame {
            self.latest_fps = frame.fps;
            let now = Instant::now();
            self.latest_total_delay_ms = now
                .saturating_duration_since(frame.uvc_ready_at)
                .as_secs_f32()
                * 1000.0;
            self.latest_queue_delay_ms = now
                .saturating_duration_since(frame.app_ready_at)
                .as_secs_f32()
                * 1000.0;
            self.latest_uvc_wait_ms = frame.uvc_wait_ms;
            self.latest_decode_ms = frame.decode_ms;
            self.latest_rgb_pack_ms = frame.rgb_pack_ms;
            self.update_texture(ctx, frame);
        }
    }

    fn update_texture(&mut self, ctx: &egui::Context, frame: FrameMessage) {
        let image = egui::ColorImage::from_rgb([frame.width, frame.height], &frame.rgb);
        match &mut self.texture {
            Some(texture) => {
                texture.set(image, egui::TextureOptions::LINEAR);
            }
            None => {
                self.texture =
                    Some(ctx.load_texture("camera_frame", image, egui::TextureOptions::LINEAR));
            }
        }
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Camera Controls");
        ui.separator();

        if self.devices.is_empty() {
            ui.label("No /dev/video* devices found");
        } else {
            egui::ComboBox::from_label("Device")
                .selected_text(self.selected_device.clone())
                .show_ui(ui, |ui| {
                    for device in &self.devices {
                        ui.selectable_value(
                            &mut self.selected_device,
                            device.path.clone(),
                            format!("{} ({})", device.path, device.name),
                        );
                    }
                });
        }

        ui.add(egui::Slider::new(&mut self.width, 160..=1920).text("Width"));
        ui.add(egui::Slider::new(&mut self.height, 120..=1080).text("Height"));
        ui.add(egui::Slider::new(&mut self.fps, 5..=120).text("Target FPS"));
        ui.checkbox(&mut self.fit_window, "Fit image to window");
        ui.add(egui::Slider::new(&mut self.image_scale, 0.25..=4.0).text("Image scale"));

        ui.horizontal(|ui| {
            if ui.button("Apply").clicked() {
                if let Err(err) = self.restart_capture() {
                    self.status = format!("capture restart failed: {err}");
                }
            }

            if ui.button("Refresh Devices").clicked() {
                self.refresh_devices();
            }
        });

        ui.separator();
        ui.label(format!("Status: {}", self.status));
        ui.label(format!("Live FPS: {:.1}", self.latest_fps));
        ui.separator();
        ui.label(format!(
            "Estimated app delay (UVC -> display): {:.2} ms",
            self.latest_total_delay_ms
        ));
        ui.label(format!(
            "Stage breakdown: wait {:.2} ms | decode {:.2} ms | rgb {:.2} ms | queue/ui {:.2} ms",
            self.latest_uvc_wait_ms,
            self.latest_decode_ms,
            self.latest_rgb_pack_ms,
            self.latest_queue_delay_ms
        ));
    }

    fn draw_video(&self, ui: &mut egui::Ui) {
        let Some(texture) = &self.texture else {
            ui.centered_and_justified(|ui| {
                ui.label("Waiting for camera frames...");
            });
            return;
        };

        let base = egui::vec2(self.width as f32, self.height as f32);
        let available = ui.available_size();
        let fit_scale = if self.fit_window {
            let sx = available.x / base.x;
            let sy = available.y / base.y;
            sx.min(sy)
        } else {
            1.0
        };
        let effective_scale = (fit_scale * self.image_scale).max(0.05);
        let size = egui::vec2(base.x * effective_scale, base.y * effective_scale);

        ui.centered_and_justified(|ui| {
            ui.image((texture.id(), size));
        });
    }
}

impl eframe::App for CameraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.consume_latest_frame(ctx);

        egui::SidePanel::left("controls").show(ctx, |ui| {
            self.draw_controls(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_video(ui);
        });

        ctx.request_repaint();
    }
}
