#[derive(Debug, Clone, Copy)]
pub(crate) enum PixelFormat {
    Rgb3,
    Bgr3,
    Yuyv,
    Mjpg,
    Yu12,
    Yv12,
}

impl PixelFormat {
    pub(crate) fn preferred_order() -> &'static [Self] {
        &[
            Self::Rgb3,
            Self::Bgr3,
            Self::Yuyv,
            Self::Mjpg,
            Self::Yu12,
            Self::Yv12,
        ]
    }

    pub(crate) fn fourcc(self) -> &'static [u8; 4] {
        match self {
            Self::Rgb3 => b"RGB3",
            Self::Bgr3 => b"BGR3",
            Self::Yuyv => b"YUYV",
            Self::Mjpg => b"MJPG",
            Self::Yu12 => b"YU12",
            Self::Yv12 => b"YV12",
        }
    }

    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Rgb3 => "RGB3",
            Self::Bgr3 => "BGR3",
            Self::Yuyv => "YUYV",
            Self::Mjpg => "MJPG",
            Self::Yu12 => "YU12",
            Self::Yv12 => "YV12",
        }
    }
}
