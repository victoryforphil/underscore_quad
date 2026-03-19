use anyhow::{anyhow, Context, Result};
use rscam::{Camera, Config};

use crate::camera::convert::{
    bgr3_to_rgb_u32, mjpg_to_rgb_u32, rgb3_to_rgb_u32, yuv420_to_rgb_u32, yuyv_to_rgb_u32,
};
use crate::camera::pixel_format::PixelFormat;

pub struct CameraCapture {
    camera: Camera,
    width: usize,
    height: usize,
    pixel_format: PixelFormat,
}

impl CameraCapture {
    pub fn open(device: &str, width: u32, height: u32, fps: u32) -> Result<Self> {
        let mut camera = Camera::new(device)
            .with_context(|| format!("failed to open camera device: {device}"))?;

        let available_formats = collect_available_formats(&camera)?;
        let mut tried = Vec::new();

        for pixel_format in PixelFormat::preferred_order() {
            if !available_formats
                .iter()
                .any(|fmt| fmt == pixel_format.fourcc())
            {
                continue;
            }

            let config = Config {
                interval: (1, fps),
                resolution: (width, height),
                format: pixel_format.fourcc(),
                ..Default::default()
            };

            match camera.start(&config) {
                Ok(()) => {
                    return Ok(Self {
                        camera,
                        width: width as usize,
                        height: height as usize,
                        pixel_format: *pixel_format,
                    });
                }
                Err(err) => {
                    tried.push(format!("{} ({err})", pixel_format.name()));
                }
            }
        }

        let available = if available_formats.is_empty() {
            "none".to_string()
        } else {
            available_formats
                .iter()
                .map(|fmt| String::from_utf8_lossy(fmt).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let attempted = if tried.is_empty() {
            "no compatible formats found".to_string()
        } else {
            tried.join(", ")
        };

        Err(anyhow!(
            "failed to start camera stream {device} at {width}x{height}@{fps}. available formats: {available}. attempts: {attempted}"
        ))
    }

    pub fn capture_to_u32(&mut self, out: &mut [u32]) -> Result<()> {
        let expected = self.width * self.height;
        if out.len() != expected {
            return Err(anyhow!(
                "output buffer length {} does not match expected {expected}",
                out.len()
            ));
        }

        let frame = self.camera.capture().context("failed to capture frame")?;
        let frame_data = frame.as_ref();

        match self.pixel_format {
            PixelFormat::Yuyv => {
                let expected_bytes = expected * 2;
                if frame_data.len() < expected_bytes {
                    return Err(anyhow!(
                        "captured YUYV frame too short: got {}, expected at least {expected_bytes}",
                        frame_data.len()
                    ));
                }
                yuyv_to_rgb_u32(&frame_data[..expected_bytes], out);
            }
            PixelFormat::Rgb3 => {
                let expected_bytes = expected * 3;
                if frame_data.len() < expected_bytes {
                    return Err(anyhow!(
                        "captured RGB3 frame too short: got {}, expected at least {expected_bytes}",
                        frame_data.len()
                    ));
                }
                rgb3_to_rgb_u32(&frame_data[..expected_bytes], out);
            }
            PixelFormat::Bgr3 => {
                let expected_bytes = expected * 3;
                if frame_data.len() < expected_bytes {
                    return Err(anyhow!(
                        "captured BGR3 frame too short: got {}, expected at least {expected_bytes}",
                        frame_data.len()
                    ));
                }
                bgr3_to_rgb_u32(&frame_data[..expected_bytes], out);
            }
            PixelFormat::Yu12 => {
                let expected_bytes = expected * 3 / 2;
                if frame_data.len() < expected_bytes {
                    return Err(anyhow!(
                        "captured YU12 frame too short: got {}, expected at least {expected_bytes}",
                        frame_data.len()
                    ));
                }
                yuv420_to_rgb_u32(
                    &frame_data[..expected_bytes],
                    out,
                    self.width,
                    self.height,
                    false,
                );
            }
            PixelFormat::Yv12 => {
                let expected_bytes = expected * 3 / 2;
                if frame_data.len() < expected_bytes {
                    return Err(anyhow!(
                        "captured YV12 frame too short: got {}, expected at least {expected_bytes}",
                        frame_data.len()
                    ));
                }
                yuv420_to_rgb_u32(
                    &frame_data[..expected_bytes],
                    out,
                    self.width,
                    self.height,
                    true,
                );
            }
            PixelFormat::Mjpg => {
                mjpg_to_rgb_u32(frame_data, out, self.width, self.height)
                    .context("failed to decode MJPG frame")?;
            }
        }

        Ok(())
    }
}

fn collect_available_formats(camera: &Camera) -> Result<Vec<[u8; 4]>> {
    let mut formats = Vec::new();
    for fmt in camera.formats() {
        let fmt = fmt.context("failed to enumerate camera formats")?;
        formats.push(fmt.format);
    }
    Ok(formats)
}
