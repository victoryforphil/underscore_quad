use std::time::{Duration, Instant};

use anyhow::{bail, Result};

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

pub struct CameraCapture;

#[derive(Debug, Clone, Copy)]
pub struct CaptureTiming {
    pub uvc_wait: Duration,
    pub decode_and_convert: Duration,
    pub frame_ready_at: Instant,
}

impl CameraCapture {
    pub fn open(_device: &str, _width: u32, _height: u32, _fps: u32) -> Result<Self> {
        bail!("camera capture is currently supported only on Linux and FreeBSD")
    }

    pub fn capture_to_u32_timed(&mut self, _out: &mut [u32]) -> Result<CaptureTiming> {
        bail!("camera capture is currently supported only on Linux and FreeBSD")
    }
}

pub fn query_capabilities(_device_path: &str) -> Result<DeviceCapabilities> {
    bail!("camera capability probing is currently supported only on Linux and FreeBSD")
}
