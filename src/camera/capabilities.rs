use anyhow::{Context, Result};
use rscam::{Camera, IntervalInfo, ResolutionInfo};

#[derive(Debug, Clone)]
pub struct FormatSupport {
    pub fourcc: String,
    pub description: String,
    pub resolutions: Vec<String>,
    pub intervals: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceCapabilities {
    pub formats: Vec<FormatSupport>,
}

pub fn query_capabilities(device_path: &str) -> Result<DeviceCapabilities> {
    let camera = Camera::new(device_path)
        .with_context(|| format!("failed to open camera device for capabilities: {device_path}"))?;

    let mut formats = Vec::new();
    for fmt_result in camera.formats() {
        let fmt = fmt_result
            .with_context(|| format!("failed to enumerate format for device: {device_path}"))?;

        let fourcc = String::from_utf8_lossy(&fmt.format).to_string();
        let resolutions_info = camera.resolutions(&fmt.format);
        let resolutions = match &resolutions_info {
            Ok(info) => summarize_resolutions(info),
            Err(_) => vec!["unknown".to_string()],
        };

        let intervals = match resolutions_info {
            Ok(info) => first_resolution(&info)
                .and_then(|res| camera.intervals(&fmt.format, res).ok())
                .map_or_else(
                    || vec!["unknown".to_string()],
                    |info| summarize_intervals(&info),
                ),
            Err(_) => vec!["unknown".to_string()],
        };

        formats.push(FormatSupport {
            fourcc,
            description: fmt.description,
            resolutions,
            intervals,
        });
    }

    Ok(DeviceCapabilities { formats })
}

fn summarize_resolutions(info: &ResolutionInfo) -> Vec<String> {
    match info {
        ResolutionInfo::Discretes(values) => values
            .iter()
            .take(6)
            .map(|(w, h)| format!("{w}x{h}"))
            .collect(),
        ResolutionInfo::Stepwise { min, max, step } => vec![format!(
            "{}x{}..{}x{} step {}x{}",
            min.0, min.1, max.0, max.1, step.0, step.1
        )],
    }
}

fn summarize_intervals(info: &IntervalInfo) -> Vec<String> {
    match info {
        IntervalInfo::Discretes(values) => values
            .iter()
            .take(6)
            .map(|(n, d)| format!("{}fps", d / (*n).max(1)))
            .collect(),
        IntervalInfo::Stepwise { min, max, step } => vec![format!(
            "{}fps..{}fps step {}fps",
            max.1 / max.0.max(1),
            min.1 / min.0.max(1),
            step.1 / step.0.max(1)
        )],
    }
}

fn first_resolution(info: &ResolutionInfo) -> Option<(u32, u32)> {
    match info {
        ResolutionInfo::Discretes(values) => values.first().copied(),
        ResolutionInfo::Stepwise { max, .. } => Some(*max),
    }
}
