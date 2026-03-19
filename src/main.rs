mod camera;
mod cli;
mod fps;
pub mod gamepad;
mod logging;
mod picker;
mod terminal_ui;
mod tui_arg_select;

mod ui;

use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about = "Display a UVC camera feed with FPS")]
pub struct Cli {
    #[arg(long, default_value_t = false)]
    list_devices: bool,
    #[arg(long, default_value_t = false)]
    list_gamepads: bool,
    #[arg(long, default_value_t = false)]
    gamepad_debug: bool,
    #[arg(long, default_value_t = 10)]
    gamepad_debug_seconds: u64,
    #[arg(long, env = "UQ_GAMEPAD_CONFIG")]
    gamepad_config: Option<String>,
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
    logging::init_logger();
    let cli = Cli::parse();

    if cli.list_devices {
        cli::print_devices(cli.verbose)?;
        return Ok(());
    }

    if cli.list_gamepads {
        cli::print_gamepads(cli.verbose)?;
        return Ok(());
    }

    if cli.gamepad_debug {
        cli::debug_gamepads(cli.gamepad_config.as_deref(), cli.gamepad_debug_seconds)?;
        return Ok(());
    }

    ui::run(cli)
}
