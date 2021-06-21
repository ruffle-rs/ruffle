//! YUV-to-RGB decode

fn clamp(v: f32) -> u8 {
    (v + 0.5) as u8
}

fn clamped_index(width: i32, height: i32, x: i32, y: i32) -> usize {
    (x.max(0).min(width - 1) + (y.max(0).min(height - 1) * width)) as usize
}

fn unclamped_index(width: i32, x: i32, y: i32) -> usize {
    (x + y * width) as usize
}

fn sample_chroma_for_luma(
    chroma: &[u8],
    chroma_width: usize,
    chroma_height: usize,
    luma_x: usize,
    luma_y: usize,
    clamp: bool,
) -> u8 {
    let width = chroma_width as i32;
    let height = chroma_height as i32;

    let sample_00;
    let sample_01;
    let sample_10;
    let sample_11;

    if clamp {
        let chroma_x = if luma_x == 0 {
            -1
        } else {
            (luma_x as i32 - 1) / 2
        };
        let chroma_y = if luma_y == 0 {
            -1
        } else {
            (luma_y as i32 - 1) / 2
        };

        sample_00 = chroma
            .get(clamped_index(width, height, chroma_x, chroma_y))
            .copied()
            .unwrap_or(0) as u16;
        sample_10 = chroma
            .get(clamped_index(width, height, chroma_x + 1, chroma_y))
            .copied()
            .unwrap_or(0) as u16;
        sample_01 = chroma
            .get(clamped_index(width, height, chroma_x, chroma_y + 1))
            .copied()
            .unwrap_or(0) as u16;
        sample_11 = chroma
            .get(clamped_index(width, height, chroma_x + 1, chroma_y + 1))
            .copied()
            .unwrap_or(0) as u16;
    } else {
        let chroma_x = (luma_x as i32 - 1) / 2;
        let chroma_y = (luma_y as i32 - 1) / 2;

        let base = unclamped_index(width, chroma_x, chroma_y);
        sample_00 = chroma.get(base).copied().unwrap_or(0) as u16;
        sample_10 = chroma.get(base + 1).copied().unwrap_or(0) as u16;
        sample_01 = chroma.get(base + chroma_width).copied().unwrap_or(0) as u16;
        sample_11 = chroma.get(base + chroma_width + 1).copied().unwrap_or(0) as u16;
    }

    let interp_left = luma_x % 2 != 0;
    let interp_top = luma_y % 2 != 0;

    let mut sample: u16 = 0;
    sample += sample_00 * if interp_left { 3 } else { 1 };
    sample += sample_10 * if interp_left { 1 } else { 3 };

    sample += sample_01 * if interp_left { 3 } else { 1 };
    sample += sample_11 * if interp_left { 1 } else { 3 };

    sample += sample_00 * if interp_top { 3 } else { 1 };
    sample += sample_01 * if interp_top { 1 } else { 3 };

    sample += sample_10 * if interp_top { 3 } else { 1 };
    sample += sample_11 * if interp_top { 1 } else { 3 };

    ((sample + 8) / 16) as u8
}

fn yuv_to_rgb(yuv: (f32, f32, f32)) -> (f32, f32, f32) {
    let (mut y_sample, mut b_sample, mut r_sample) = yuv;

    y_sample = (y_sample - 16.0) * (255.0 / (235.0 - 16.0));
    b_sample = (b_sample - 16.0) * (255.0 / (240.0 - 16.0)) - 128.0;
    r_sample = (r_sample - 16.0) * (255.0 / (240.0 - 16.0)) - 128.0;

    let r = y_sample + r_sample * 1.370705;
    let g = y_sample + r_sample * -0.698001 + b_sample * -0.337633;
    let b = y_sample + b_sample * 1.732446;

    (r, g, b)
}

fn convert_and_write_pixel(
    yuv: (f32, f32, f32),
    rgba: &mut Vec<u8>,
    width: usize,
    x_pos: usize,
    y_pos: usize,
) {
    let (r, g, b) = yuv_to_rgb(yuv);

    let base = (x_pos + y_pos * width) * 4;
    rgba[base] = clamp(r);
    rgba[base + 1] = clamp(g);
    rgba[base + 2] = clamp(b);
    rgba[base + 3] = 255;
}

/// Convert YUV 4:2:0 data into RGB 1:1:1 data.
///
/// This function yields an RGBA picture with the same number of pixels as were
/// provided in the `y` picture. The `b` and `r` pictures will be resampled at
/// this stage, and the resulting picture will have color components mixed.
pub fn yuv420_to_rgba(
    y: &[u8],
    chroma_b: &[u8],
    chroma_r: &[u8],
    y_width: usize,
    br_width: usize,
) -> Vec<u8> {
    let y_height = y.len() / y_width;
    let br_height = chroma_b.len() / br_width;

    let mut rgba = vec![0; y.len() * 4];

    // do the bulk of the pixels faster, with no clamping, leaving out the edges
    for y_pos in 1..y_height - 1 {
        for x_pos in 1..y_width - 1 {
            let y_sample = y.get(x_pos + y_pos * y_width).copied().unwrap_or(0) as f32;
            let b_sample =
                sample_chroma_for_luma(chroma_b, br_width, br_height, x_pos, y_pos, false) as f32;
            let r_sample =
                sample_chroma_for_luma(chroma_r, br_width, br_height, x_pos, y_pos, false) as f32;

            convert_and_write_pixel(
                (y_sample, b_sample, r_sample),
                &mut rgba,
                y_width,
                x_pos,
                y_pos,
            );
        }
    }

    // doing the sides with clamping
    for y_pos in 0..y_height {
        for x_pos in [0, y_width - 1].iter() {
            let y_sample = y.get(x_pos + y_pos * y_width).copied().unwrap_or(0) as f32;
            let b_sample =
                sample_chroma_for_luma(chroma_b, br_width, br_height, *x_pos, y_pos, true) as f32;
            let r_sample =
                sample_chroma_for_luma(chroma_r, br_width, br_height, *x_pos, y_pos, true) as f32;

            convert_and_write_pixel(
                (y_sample, b_sample, r_sample),
                &mut rgba,
                y_width,
                *x_pos,
                y_pos,
            );
        }
    }

    // doing the top and bottom edges with clamping
    for x_pos in 0..y_width {
        for y_pos in [0, y_height - 1].iter() {
            let y_sample = y.get(x_pos + y_pos * y_width).copied().unwrap_or(0) as f32;
            let b_sample =
                sample_chroma_for_luma(chroma_b, br_width, br_height, x_pos, *y_pos, true) as f32;
            let r_sample =
                sample_chroma_for_luma(chroma_r, br_width, br_height, x_pos, *y_pos, true) as f32;

            convert_and_write_pixel(
                (y_sample, b_sample, r_sample),
                &mut rgba,
                y_width,
                x_pos,
                *y_pos,
            );
        }
    }

    rgba
}
