use anyhow::Result;
use eframe::egui;
use egui::{Color32, RichText};

use crate::camera::list_video_devices;
use crate::cli::resolve_device;
use crate::gamepad::{GamepadControlFrame, GamepadSnapshot, GamepadWorker, ResolvedGamepadConfig};
use crate::Cli;

use super::capture_settings::CaptureSettings;
use super::device_picker::DevicePicker;
use super::display_settings::DisplaySettings;
use super::gamepad_panel::{self, GamepadPanelState};
use super::latency::LatencyTracker;
use super::latency_table;
use super::status_bar::{self, StatusBarState};
use super::theme;
use super::top_bar::{self, TopBarState};
use super::video_view::VideoView;
use super::worker::CaptureWorker;

/// Top-level eframe application.  Owns the sub-widgets and capture worker.
pub struct CameraApp {
    device_picker: DevicePicker,
    capture: CaptureSettings,
    display: DisplaySettings,
    latency: LatencyTracker,
    video: VideoView,
    worker: Option<CaptureWorker>,
    gamepad_worker: Option<GamepadWorker>,
    gamepad_snapshot: Option<GamepadSnapshot>,
    gamepad_control: Option<GamepadControlFrame>,
    gamepad_config_source: String,
    gamepad_error: Option<String>,
    fps: f32,
    frame_count: u64,
    status: String,
}

impl CameraApp {
    pub fn new(cc: &eframe::CreationContext<'_>, cli: Cli) -> Self {
        theme::apply_theme(&cc.egui_ctx);

        let devices = list_video_devices().unwrap_or_default();
        let selected = resolve_device(&cli.device).unwrap_or(cli.device);

        let mut app = Self {
            device_picker: DevicePicker::new(devices, selected),
            capture: CaptureSettings::new(cli.width, cli.height, cli.fps),
            display: DisplaySettings::new(cli.fit_window, cli.scale),
            latency: LatencyTracker::default(),
            video: VideoView::default(),
            worker: None,
            gamepad_worker: None,
            gamepad_snapshot: None,
            gamepad_control: None,
            gamepad_config_source: "built-in defaults".to_string(),
            gamepad_error: None,
            fps: 0.0,
            frame_count: 0,
            status: "idle".to_string(),
        };

        app.start_gamepad(cli.gamepad_config.as_deref());

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
        self.video.reset();
        self.frame_count = 0;

        let device = &self.device_picker.selected;
        let worker = CaptureWorker::start(
            device.clone(),
            self.capture.width,
            self.capture.height,
            self.capture.fps,
        )?;

        self.status = format!(
            "streaming {} at {}x{}@{}",
            device, self.capture.width, self.capture.height, self.capture.fps,
        );
        self.worker = Some(worker);
        Ok(())
    }

    fn consume_latest_frame(&mut self, ctx: &egui::Context) {
        let frame = self.worker.as_ref().and_then(CaptureWorker::latest_frame);
        if let Some(frame) = frame {
            self.fps = frame.fps;
            self.frame_count = self.frame_count.wrapping_add(1);
            self.latency.record(&frame);
            self.video.update(ctx, frame);
        }
    }

    fn start_gamepad(&mut self, config_path: Option<&str>) {
        self.gamepad_worker = None;
        self.gamepad_snapshot = None;
        self.gamepad_control = None;
        self.gamepad_error = None;

        match ResolvedGamepadConfig::from_optional_path(config_path) {
            Ok(config) => {
                self.gamepad_config_source = config
                    .path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "built-in defaults".to_string());

                match GamepadWorker::start(config.worker) {
                    Ok(worker) => {
                        self.gamepad_worker = Some(worker);
                    }
                    Err(err) => {
                        self.gamepad_error = Some(format!("gamepad worker failed: {err}"));
                    }
                }
            }
            Err(err) => {
                self.gamepad_config_source = config_path.unwrap_or("built-in defaults").to_string();
                self.gamepad_error = Some(format!("gamepad config failed: {err}"));
            }
        }
    }

    fn consume_latest_gamepad_update(&mut self) {
        let update = self
            .gamepad_worker
            .as_ref()
            .and_then(GamepadWorker::latest_update);

        if let Some(update) = update {
            self.gamepad_snapshot = update.active_snapshot;
            self.gamepad_control = update.control_frame;
        }
    }

    // -- Panel drawing helpers ------------------------------------------------

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.add_space(4.0);

                egui::CollapsingHeader::new(RichText::new("Device").strong())
                    .default_open(true)
                    .show(ui, |ui| self.device_picker.draw(ui));

                egui::CollapsingHeader::new(RichText::new("Capture").strong())
                    .default_open(true)
                    .show(ui, |ui| {
                        if self.capture.draw(ui) {
                            if let Err(err) = self.restart_capture() {
                                self.status = format!("capture restart failed: {err}");
                            }
                        }
                    });

                egui::CollapsingHeader::new(RichText::new("Display").strong())
                    .default_open(true)
                    .show(ui, |ui| self.display.draw(ui));

                egui::CollapsingHeader::new(RichText::new("Gamepad").strong())
                    .default_open(true)
                    .show(ui, |ui| {
                        gamepad_panel::draw(
                            ui,
                            &GamepadPanelState {
                                config_source: &self.gamepad_config_source,
                                snapshot: self.gamepad_snapshot.as_ref(),
                                control_frame: self.gamepad_control.as_ref(),
                                error: self.gamepad_error.as_deref(),
                            },
                        );
                    });

                egui::CollapsingHeader::new(RichText::new("Latency").strong())
                    .default_open(true)
                    .show(ui, |ui| latency_table::draw(ui, &self.latency));

                ui.add_space(8.0);
            });
    }
}

impl eframe::App for CameraApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.consume_latest_frame(ctx);
        self.consume_latest_gamepad_update();

        let is_streaming = self.worker.is_some() && self.fps > 0.0;
        let gamepad_label = self.gamepad_snapshot.as_ref().map(|snapshot| {
            format!(
                "{}:{}",
                snapshot.id,
                snapshot.name.chars().take(12).collect::<String>()
            )
        });

        // -- Top bar --
        egui::TopBottomPanel::top("top_bar")
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_DARK)
                    .inner_margin(egui::Margin::symmetric(12, 6))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 55))),
            )
            .show(ctx, |ui| {
                top_bar::draw(
                    ui,
                    &TopBarState {
                        is_streaming,
                        gamepad_active: self.gamepad_snapshot.is_some(),
                        gamepad_label,
                        fps: self.fps,
                        total_delay_ms: self.latency.total_ms,
                        width: self.capture.width,
                        height: self.capture.height,
                        frame_count: self.frame_count,
                    },
                );
            });

        // -- Bottom bar --
        egui::TopBottomPanel::bottom("bottom_bar")
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_DARK)
                    .inner_margin(egui::Margin::symmetric(12, 4))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 55))),
            )
            .show(ctx, |ui| {
                status_bar::draw(
                    ui,
                    &StatusBarState {
                        status: &self.status,
                        uvc_wait_ms: self.latency.uvc_wait_ms,
                        decode_ms: self.latency.decode_ms,
                        rgb_pack_ms: self.latency.rgb_pack_ms,
                        queue_ms: self.latency.queue_ms,
                    },
                );
            });

        // -- Left side panel --
        egui::SidePanel::left("controls")
            .resizable(true)
            .default_width(280.0)
            .min_width(220.0)
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_PANEL)
                    .inner_margin(egui::Margin::same(10))
                    .stroke(egui::Stroke::new(1.0, Color32::from_rgb(40, 40, 55))),
            )
            .show(ctx, |ui| {
                self.draw_controls(ui);
            });

        // -- Central panel: video feed --
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_DARK)
                    .inner_margin(egui::Margin::same(0)),
            )
            .show(ctx, |ui| {
                self.video.draw(
                    ui,
                    self.capture.width,
                    self.capture.height,
                    self.display.fit_window,
                    self.display.image_scale,
                );
            });

        ctx.request_repaint();
    }
}
