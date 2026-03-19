use std::time::{Duration, Instant};

use anyhow::{Context, Result};

use crate::camera::{list_video_devices, query_capabilities};
use crate::gamepad::{GamepadCollector, GamepadControlMapping, GamepadEventKind};
use crate::terminal_ui::{
    default_stdout_context, render_table, ActionLine, Panel, SummaryFooter, Tone,
};
use crate::tui_arg_select::{select_required_arg, ArgSelectOption, RequiredArgSelectConfig};

pub fn print_devices(verbose: bool) -> Result<()> {
    let context = default_stdout_context();
    let devices = list_video_devices()?;
    if devices.is_empty() {
        println!(
            "{}",
            ActionLine::new(
                "LIST",
                "No video devices found under /dev/video*",
                Tone::Warning
            )
            .render(&context)
        );
        return Ok(());
    }

    let rows = devices
        .iter()
        .enumerate()
        .map(|(idx, d)| vec![idx.to_string(), d.path.clone(), d.name.clone()])
        .collect::<Vec<_>>();

    print_table_or_card(
        "LIST",
        "Camera inventory",
        Tone::Info,
        "Available Cameras",
        &["IDX", "DEVICE", "NAME"],
        &rows,
        Some("Use --device /dev/videoN or --device auto to launch"),
    );

    if verbose {
        for device in &devices {
            println!();
            println!(
                "{}",
                ActionLine::new(
                    "CAPS",
                    format!("{} ({})", device.path, device.name),
                    Tone::Info
                )
                .render(&context)
            );

            match query_capabilities(&device.path) {
                Ok(caps) => {
                    if caps.formats.is_empty() {
                        println!(
                            "{}",
                            Panel::new("no formats reported")
                                .with_tone(Tone::Warning)
                                .render(&context)
                        );
                        continue;
                    }

                    let cap_rows = caps
                        .formats
                        .into_iter()
                        .map(|fmt| {
                            vec![
                                fmt.fourcc,
                                if fmt.description.is_empty() {
                                    "(no description)".to_string()
                                } else {
                                    fmt.description
                                },
                                fmt.resolutions.join(", "),
                                fmt.intervals.join(", "),
                            ]
                        })
                        .collect::<Vec<_>>();

                    println!(
                        "{}",
                        render_table(
                            &["FOURCC", "DESC", "RESOLUTIONS", "INTERVALS"],
                            &cap_rows,
                            &context
                        )
                    );
                }
                Err(err) => {
                    println!(
                        "{}",
                        Panel::new(format!("failed to read capabilities: {err}"))
                            .with_tone(Tone::Error)
                            .render(&context)
                    );
                }
            }
        }
    }

    println!();
    println!(
        "{}",
        SummaryFooter::new(format!(
            "Try: cargo run -- --device {} --width 640 --height 480 --fps 30",
            devices[0].path
        ))
        .render(&context)
    );

    Ok(())
}

pub fn print_gamepads(verbose: bool) -> Result<()> {
    let context = default_stdout_context();
    let collector = GamepadCollector::new()?;
    let mut snapshots = collector.snapshots().cloned().collect::<Vec<_>>();
    snapshots.sort_by(|a, b| a.name.cmp(&b.name).then(a.os_name.cmp(&b.os_name)));

    if snapshots.is_empty() {
        println!(
            "{}",
            ActionLine::new("LIST", "No gamepads detected", Tone::Warning).render(&context)
        );
        println!();
        println!(
            "{}",
            SummaryFooter::new(
                "On Linux/Steam Deck, ensure Steam Input or evdev access is available"
            )
            .render(&context)
        );
        return Ok(());
    }

    let rows = snapshots
        .iter()
        .enumerate()
        .map(|(idx, snapshot)| {
            vec![
                idx.to_string(),
                snapshot.name.clone(),
                snapshot.os_name.clone(),
                format!("{}", snapshot.is_connected),
                format_vid_pid(snapshot.vendor_id, snapshot.product_id),
            ]
        })
        .collect::<Vec<_>>();

    print_table_or_card(
        "LIST",
        "Gamepad inventory",
        Tone::Info,
        "Available Gamepads",
        &["IDX", "NAME", "OS NAME", "CONNECTED", "VID:PID"],
        &rows,
        Some("Use --gamepad-debug to inspect live inputs and normalized control channels"),
    );

    if verbose {
        for snapshot in &snapshots {
            println!();
            println!(
                "{}",
                ActionLine::new(
                    "PAD",
                    format!("{} ({})", snapshot.name, snapshot.os_name),
                    Tone::Info
                )
                .render(&context)
            );
            println!(
                "{}",
                Panel::new(format!(
                    "axes: {}\nbuttons: {}",
                    format_axes(snapshot),
                    format_buttons(snapshot)
                ))
                .with_title("Current state")
                .with_tone(Tone::Info)
                .render(&context)
            );
        }
    }

    Ok(())
}

pub fn debug_gamepads(duration_secs: u64) -> Result<()> {
    let mut collector = GamepadCollector::new()?;
    let mapping = GamepadControlMapping::trigger_throttle_default();
    let duration = Duration::from_secs(duration_secs.max(1));
    let deadline = Instant::now() + duration;

    print_gamepads(true)?;
    println!();
    println!(
        "{}",
        ActionLine::new(
            "LIVE",
            format!("Inspecting gamepad events for {}s", duration.as_secs()),
            Tone::Info
        )
        .render(&default_stdout_context())
    );

    while Instant::now() < deadline {
        let events = collector.poll_blocking(Some(Duration::from_millis(250)));
        if events.is_empty() {
            continue;
        }

        for event in events {
            let detail = match event.kind {
                GamepadEventKind::Connected => "connected".to_string(),
                GamepadEventKind::Disconnected => "disconnected".to_string(),
                GamepadEventKind::AxisChanged { axis, value } => {
                    format!("axis {:?} -> {:.3}", axis, value)
                }
                GamepadEventKind::ButtonPressed { button } => {
                    format!("button {:?} pressed", button)
                }
                GamepadEventKind::ButtonReleased { button } => {
                    format!("button {:?} released", button)
                }
                GamepadEventKind::ButtonChanged { button, value } => {
                    format!("button {:?} -> {:.3}", button, value)
                }
            };

            println!("[PAD {:?}] {detail}", event.gamepad_id);
        }

        if let Some(snapshot) = collector.active_snapshot() {
            let control = mapping.map_snapshot(snapshot);
            println!(
                "  control roll={:.3} pitch={:.3} yaw={:.3} throttle={:.3}",
                control.roll, control.pitch, control.yaw, control.throttle
            );
        }
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

    if devices.len() < 2 {
        return Ok(first);
    }

    let options = devices
        .iter()
        .map(|d| ArgSelectOption::new(format!("{}  {}", d.path, d.name), d.path.clone()))
        .collect::<Vec<_>>();

    let selected = select_required_arg(
        RequiredArgSelectConfig::new(
            "device",
            "Choose camera device",
            "cargo run -- --device /dev/video0",
        )
        .with_cancelled_message("device selection cancelled"),
        options,
    );

    match selected {
        Ok(value) => Ok(value),
        Err(_) => Ok(first),
    }
}

fn print_table_or_card(
    action: &str,
    headline: &str,
    tone: Tone,
    panel_title: &str,
    headers: &[&str],
    rows: &[Vec<String>],
    summary: Option<&str>,
) {
    let context = default_stdout_context();
    println!(
        "{}",
        ActionLine::new(action, headline, tone).render(&context)
    );
    println!();

    if rows.is_empty() {
        println!(
            "{}",
            Panel::new("No entries")
                .with_title(panel_title)
                .with_tone(Tone::Warning)
                .render(&context)
        );
    } else {
        println!("{}", render_table(headers, rows, &context));
    }

    if let Some(summary_text) = summary {
        println!();
        println!("{}", SummaryFooter::new(summary_text).render(&context));
    }
}

fn format_vid_pid(vendor_id: Option<u16>, product_id: Option<u16>) -> String {
    match (vendor_id, product_id) {
        (Some(vendor_id), Some(product_id)) => format!("{vendor_id:04x}:{product_id:04x}"),
        _ => "unknown".to_string(),
    }
}

fn format_axes(snapshot: &crate::gamepad::GamepadSnapshot) -> String {
    snapshot
        .axes
        .iter()
        .map(|(axis, value)| format!("{:?}={value:.2}", axis))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_buttons(snapshot: &crate::gamepad::GamepadSnapshot) -> String {
    snapshot
        .buttons
        .iter()
        .filter(|(_, value)| **value > 0.0)
        .map(|(button, value)| format!("{:?}={value:.2}", button))
        .collect::<Vec<_>>()
        .join(", ")
}
