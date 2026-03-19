#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod capabilities;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod capture;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod convert;
mod device;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
mod pixel_format;
#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
mod unsupported;

#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use capabilities::query_capabilities;
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub use capture::CameraCapture;
pub use device::{list_video_devices, VideoDevice};
#[cfg(not(any(target_os = "linux", target_os = "freebsd")))]
pub use unsupported::{query_capabilities, CameraCapture};
