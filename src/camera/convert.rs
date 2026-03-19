use anyhow::{anyhow, Result};
use zune_jpeg::JpegDecoder;

pub(crate) fn yuyv_to_rgb_u32(src: &[u8], dst: &mut [u32]) {
    for (pair_idx, chunk) in src.chunks_exact(4).enumerate() {
        let y0 = chunk[0] as i32;
        let u = chunk[1] as i32;
        let y1 = chunk[2] as i32;
        let v = chunk[3] as i32;

        let base = pair_idx * 2;
        dst[base] = yuv_to_rgb_u32(y0, u, v);
        dst[base + 1] = yuv_to_rgb_u32(y1, u, v);
    }
}

pub(crate) fn rgb3_to_rgb_u32(src: &[u8], dst: &mut [u32]) {
    for (idx, chunk) in src.chunks_exact(3).enumerate() {
        let r = chunk[0] as u32;
        let g = chunk[1] as u32;
        let b = chunk[2] as u32;
        dst[idx] = (r << 16) | (g << 8) | b;
    }
}

pub(crate) fn bgr3_to_rgb_u32(src: &[u8], dst: &mut [u32]) {
    for (idx, chunk) in src.chunks_exact(3).enumerate() {
        let b = chunk[0] as u32;
        let g = chunk[1] as u32;
        let r = chunk[2] as u32;
        dst[idx] = (r << 16) | (g << 8) | b;
    }
}

pub(crate) fn yuv420_to_rgb_u32(
    src: &[u8],
    dst: &mut [u32],
    width: usize,
    height: usize,
    yv12: bool,
) {
    let y_len = width * height;
    let uv_len = y_len / 4;

    let y_plane = &src[..y_len];
    let (u_plane, v_plane) = if yv12 {
        (
            &src[y_len + uv_len..y_len + uv_len * 2],
            &src[y_len..y_len + uv_len],
        )
    } else {
        (
            &src[y_len..y_len + uv_len],
            &src[y_len + uv_len..y_len + uv_len * 2],
        )
    };

    for py in 0..height {
        for px in 0..width {
            let y_idx = py * width + px;
            let uv_idx = (py / 2) * (width / 2) + (px / 2);
            let y = y_plane[y_idx] as i32;
            let u = u_plane[uv_idx] as i32;
            let v = v_plane[uv_idx] as i32;
            dst[y_idx] = yuv_to_rgb_u32(y, u, v);
        }
    }
}

pub(crate) fn mjpg_to_rgb_u32(
    src: &[u8],
    dst: &mut [u32],
    width: usize,
    height: usize,
) -> Result<()> {
    let mut decoder = JpegDecoder::new(src);
    let decoded = decoder
        .decode()
        .map_err(|err| anyhow!("jpeg decode failed: {err}"))?;

    let expected = width * height;
    if decoded.len() == expected * 3 {
        rgb3_to_rgb_u32(&decoded, dst);
        return Ok(());
    }

    if decoded.len() == expected {
        for (idx, y) in decoded.iter().enumerate() {
            let y = *y as u32;
            dst[idx] = (y << 16) | (y << 8) | y;
        }
        return Ok(());
    }

    Err(anyhow!(
        "unexpected decoded jpeg size: got {}, expected {} (gray) or {} (rgb)",
        decoded.len(),
        expected,
        expected * 3
    ))
}

fn yuv_to_rgb_u32(y: i32, u: i32, v: i32) -> u32 {
    let c = y - 16;
    let d = u - 128;
    let e = v - 128;

    let r = clamp((298 * c + 409 * e + 128) >> 8);
    let g = clamp((298 * c - 100 * d - 208 * e + 128) >> 8);
    let b = clamp((298 * c + 516 * d + 128) >> 8);

    ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}

fn clamp(v: i32) -> u8 {
    if v < 0 {
        0
    } else if v > 255 {
        255
    } else {
        v as u8
    }
}
