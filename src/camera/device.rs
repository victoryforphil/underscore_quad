use anyhow::{Context, Result};
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use std::fs;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use std::path::Path;

#[derive(Debug, Clone)]
pub struct VideoDevice {
    pub path: String,
    pub name: String,
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub fn list_video_devices() -> Result<Vec<VideoDevice>> {
    let mut devices = Vec::new();

    for entry in fs::read_dir("/dev").context("failed to read /dev")? {
        let entry = entry.context("failed to read /dev entry")?;
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if !file_name.starts_with("video") {
            continue;
        }

        let path = format!("/dev/{file_name}");
        let sys_name_path = format!("/sys/class/video4linux/{file_name}/name");
        let name = read_trimmed(&sys_name_path).unwrap_or_else(|_| "Unknown device".to_string());

        devices.push(VideoDevice { path, name });
    }

    devices.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(devices)
}

#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
pub fn list_video_devices() -> Result<Vec<VideoDevice>> {
    Ok(Vec::new())
}

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
fn read_trimmed(path: &str) -> Result<String> {
    let value = fs::read_to_string(Path::new(path))
        .with_context(|| format!("failed to read device metadata: {path}"))?;
    Ok(value.trim().to_string())
}
