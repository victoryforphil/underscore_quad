mod camera;
mod cli;
mod fps;

mod ui;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about = "Display a UVC camera feed with FPS")]
pub struct Cli {
    #[arg(long, default_value_t = false)]
    list_devices: bool,
    #[arg(long, default_value_t = false)]
    verbose: bool,
    #[arg(long, default_value = "auto")]
    device: String,
    #[arg(long, default_value_t = 640)]
    width: u32,
    #[arg(long, default_value_t = 480)]
    height: u32,
    #[arg(long, default_value_t = 30)]
    fps: u32,
    #[arg(long, default_value_t = true)]
    fit_window: bool,
    #[arg(long, default_value_t = 1.0)]
    scale: f32,
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let cli = Cli::parse();

    if cli.list_devices {
        cli::print_devices(cli.verbose)?;
        return Ok(());
    }

    ui::run(cli)
}
