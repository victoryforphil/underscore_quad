use font8x8::{UnicodeFonts, BASIC_FONTS};

use crate::overlay::Rect;

pub fn draw_text(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    text: &str,
    color: u32,
) {
    let mut cursor_x = x;
    for ch in text.chars() {
        draw_char(pixels, width, height, cursor_x, y, ch, color);
        cursor_x += 8;
    }
}

pub fn draw_label_box(pixels: &mut [u32], width: usize, height: usize, rect: Rect, color: u32) {
    let max_x = rect.x.saturating_add(rect.w).min(width);
    let max_y = rect.y.saturating_add(rect.h).min(height);

    for py in rect.y..max_y {
        for px in rect.x..max_x {
            let idx = py * width + px;
            let src = pixels[idx];
            pixels[idx] = alpha_blend_50(src, color);
        }
    }
}

fn draw_char(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    ch: char,
    color: u32,
) {
    let Some(glyph) = BASIC_FONTS.get(ch) else {
        return;
    };

    for (row, bits) in glyph.iter().enumerate() {
        let py = y + row;
        if py >= height {
            break;
        }

        for col in 0..8 {
            let px = x + col;
            if px >= width {
                break;
            }

            if bits & (1 << col) != 0 {
                let idx = py * width + px;
                pixels[idx] = color;
            }
        }
    }
}

fn alpha_blend_50(src: u32, over: u32) -> u32 {
    let sr = ((src >> 16) & 0xff) as u16;
    let sg = ((src >> 8) & 0xff) as u16;
    let sb = (src & 0xff) as u16;

    let or = ((over >> 16) & 0xff) as u16;
    let og = ((over >> 8) & 0xff) as u16;
    let ob = (over & 0xff) as u16;

    let r = ((sr + or) / 2) as u32;
    let g = ((sg + og) / 2) as u32;
    let b = ((sb + ob) / 2) as u32;

    (r << 16) | (g << 8) | b
}
