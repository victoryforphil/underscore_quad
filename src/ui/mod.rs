mod app;
mod capture_settings;
mod device_picker;
mod display_settings;
mod latency;
mod latency_table;
mod status_bar;
mod theme;
mod top_bar;
mod video_view;
mod worker;

use anyhow::Result;
use eframe::egui;

use crate::Cli;

pub fn run(cli: Cli) -> Result<()> {
    let initial_size = egui::vec2(cli.width as f32 + 360.0, cli.height as f32 + 120.0);
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("underscore_quad")
            .with_inner_size(initial_size),
        ..Default::default()
    };

    eframe::run_native(
        "underscore_quad",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::CameraApp::new(cc, cli)))),
    )
    .map_err(|err| anyhow::anyhow!("failed to run egui app: {err}"))?;

    Ok(())
}
