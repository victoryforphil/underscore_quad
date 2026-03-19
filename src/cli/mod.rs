use anyhow::{Context, Result};

use crate::camera::{list_video_devices, query_capabilities, VideoDevice};

pub fn print_devices(verbose: bool) -> Result<()> {
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
    print_action("TIP", "Run preview with:");
    println!(
        "  cargo run -- --device {} --width 640 --height 480 --fps 30",
        devices[0].path
    );
    if !verbose {
        println!("  cargo run -- --list-devices --verbose");
    }

    Ok(())
}

pub fn resolve_device(requested: &str) -> Result<String> {
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
