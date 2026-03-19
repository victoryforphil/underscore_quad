mod camera;
mod fps;
mod overlay;

use std::time::Duration;

use anyhow::{Context, Result};
use camera::{list_video_devices, query_capabilities, CameraCapture, VideoDevice};
use clap::Parser;
use fps::FpsCounter;
use minifb::{Key, Window, WindowOptions};
use overlay::{draw_label_box, draw_text, Rect};

#[derive(Debug, Parser)]
#[command(author, version, about = "Display a UVC camera feed with FPS")]
struct Cli {
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
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let cli = Cli::parse();

    if cli.list_devices {
        print_devices(cli.verbose)?;
        return Ok(());
    }

    let selected_device = resolve_device(&cli.device)?;

    let width = cli.width as usize;
    let height = cli.height as usize;

    let mut capture = CameraCapture::open(&selected_device, cli.width, cli.height, cli.fps)?;
    let mut window = Window::new(
        "underscore_quad - camera",
        width,
        height,
        WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        },
    )
    .context("failed to create display window")?;

    window.set_target_fps(0);

    let mut pixels = vec![0_u32; width * height];
    let mut fps_counter = FpsCounter::new(Duration::from_millis(500));

    while window.is_open() && !window.is_key_down(Key::Escape) {
        capture.capture_to_u32(&mut pixels)?;

        let fps = fps_counter.tick();
        let label = format!("FPS: {:>5.1}", fps);
        draw_label_box(
            &mut pixels,
            width,
            height,
            Rect {
                x: 8,
                y: 8,
                w: 140,
                h: 20,
            },
            0x101010,
        );
        draw_text(&mut pixels, width, height, 12, 14, &label, 0x00ff00);

        window
            .update_with_buffer(&pixels, width, height)
            .context("failed to update window buffer")?;

        window.set_title(&format!(
            "underscore_quad - {} - {}",
            selected_device, label
        ));
    }

    Ok(())
}

fn print_devices(verbose: bool) -> Result<()> {
    let devices = list_video_devices()?;
    if devices.is_empty() {
        print_action("LIST", "No video devices found under /dev/video*");
        return Ok(());
    }

    print_action("LIST", "Camera inventory");
    println!("{}", render_device_table(&devices));

    if verbose {
        for device in &devices {
            println!();
            print_action("CAPS", &format!("{} ({})", device.path, device.name));
            match query_capabilities(&device.path) {
                Ok(caps) => {
                    if caps.formats.is_empty() {
                        println!("  no formats reported");
                        continue;
                    }

                    for fmt in caps.formats {
                        println!(
                            "  - {}  {}",
                            fmt.fourcc,
                            if fmt.description.is_empty() {
                                "(no description)"
                            } else {
                                &fmt.description
                            }
                        );
                        println!("      resolutions: {}", fmt.resolutions.join(", "));
                        println!("      intervals:   {}", fmt.intervals.join(", "));
                    }
                }
                Err(err) => {
                    println!("  failed to read capabilities: {err}");
                }
            }
        }
    }

    println!();
    print_action("TIP", "Run preview with: ");
    println!(
        "  cargo run -- --device {} --width 640 --height 480 --fps 30",
        devices[0].path
    );
    if !verbose {
        println!("  cargo run -- --list-devices --verbose");
    }

    Ok(())
}

fn resolve_device(requested: &str) -> Result<String> {
    if requested != "auto" {
        return Ok(requested.to_string());
    }

    let devices = list_video_devices()?;
    let first = devices
        .first()
        .map(|device| device.path.clone())
        .context("--device auto requested but no /dev/video* devices were found")?;
    Ok(first)
}

fn print_action(label: &str, message: &str) {
    println!("[{: <7}] {message}", label);
}

fn render_device_table(devices: &[VideoDevice]) -> String {
    let headers = ["IDX", "DEVICE", "NAME"];
    let mut rows = Vec::with_capacity(devices.len());
    for (idx, device) in devices.iter().enumerate() {
        rows.push(vec![
            idx.to_string(),
            device.path.clone(),
            device.name.clone(),
        ]);
    }

    let mut widths = [headers[0].len(), headers[1].len(), headers[2].len()];
    for row in &rows {
        for (col, cell) in row.iter().enumerate() {
            widths[col] = widths[col].max(cell.len());
        }
    }

    let border = format!(
        "+-{}-+-{}-+-{}-+",
        "-".repeat(widths[0]),
        "-".repeat(widths[1]),
        "-".repeat(widths[2])
    );

    let mut out = String::new();
    out.push_str(&border);
    out.push('\n');
    out.push_str(&format!(
        "| {:<w0$} | {:<w1$} | {:<w2$} |",
        headers[0],
        headers[1],
        headers[2],
        w0 = widths[0],
        w1 = widths[1],
        w2 = widths[2]
    ));
    out.push('\n');
    out.push_str(&border);

    for row in rows {
        out.push('\n');
        out.push_str(&format!(
            "| {:<w0$} | {:<w1$} | {:<w2$} |",
            row[0],
            row[1],
            row[2],
            w0 = widths[0],
            w1 = widths[1],
            w2 = widths[2]
        ));
    }

    out.push('\n');
    out.push_str(&border);
    out
}
