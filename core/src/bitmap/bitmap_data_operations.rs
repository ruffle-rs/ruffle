use crate::bitmap::bitmap_data::{BitmapDataWrapper, ChannelOptions, Color, LehmerRng};
use crate::bitmap::turbulence::Turbulence;
use crate::context::UpdateContext;
use ruffle_render::bitmap::PixelRegion;

/// AVM1 and AVM2 have a shared set of operations they can perform on BitmapDatas.
/// Instead of directly manipulating the BitmapData in each place, they should call
/// a shared method here which will do it.
///
/// This will allow us to be able to optimise the implementations and share the
/// same code between VMs.

pub fn fill_rect<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: i32,
) {
    if target.disposed() {
        return;
    }

    let mut rect = PixelRegion::for_region_i32(x, y, width, height);
    rect.clamp(target.width(), target.height());

    if rect.width() == 0 || rect.height() == 0 {
        return;
    }

    let target = if rect.width() == target.width() && rect.height() == target.height() {
        // If we're filling the whole region, we can discard the gpu data
        target.overwrite_cpu_pixels_from_gpu(context).0
    } else {
        // If we're filling a partial region, finish any gpu->cpu sync
        target.sync()
    };
    let mut write = target.write(context.gc_context);
    let color = Color::from(color).to_premultiplied_alpha(write.transparency());

    for x in rect.min_x..rect.max_x {
        for y in rect.min_y..rect.max_y {
            write.set_pixel32_raw(x, y, color);
        }
    }
    write.set_cpu_dirty(rect);
}

pub fn set_pixel32<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    x: u32,
    y: u32,
    color: i32,
) {
    if target.disposed() || x >= target.width() || y >= target.height() {
        return;
    }
    let target = target.sync();
    let mut write = target.write(context.gc_context);
    let transparency = write.transparency();
    write.set_pixel32_raw(
        x,
        y,
        Color::from(color).to_premultiplied_alpha(transparency),
    );
    write.set_cpu_dirty(PixelRegion::for_pixel(x, y));
}

pub fn get_pixel32(target: BitmapDataWrapper, x: u32, y: u32) -> i32 {
    if target.disposed() || x >= target.width() || y >= target.height() {
        return 0;
    }
    let target = target.sync();
    let read = target.read();
    read.get_pixel32_raw(x, y).to_un_multiplied_alpha().into()
}

pub fn get_pixel(target: BitmapDataWrapper, x: u32, y: u32) -> i32 {
    if target.disposed() || x >= target.width() || y >= target.height() {
        return 0;
    }
    let target = target.sync();
    let read = target.read();
    read.get_pixel32_raw(x, y)
        .to_un_multiplied_alpha()
        .with_alpha(0x0)
        .into()
}

pub fn flood_fill<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    x: u32,
    y: u32,
    color: i32,
) {
    if target.disposed() || x >= target.width() || y >= target.height() {
        return;
    }
    let target = target.sync();
    let mut write = target.write(context.gc_context);
    let expected_color = write.get_pixel32_raw(x, y);
    let replace_color = Color::from(color).to_premultiplied_alpha(write.transparency());

    let mut pending = vec![(x, y)];
    let mut dirty_region = PixelRegion::for_pixel(x, y);

    while !pending.is_empty() {
        if let Some((x, y)) = pending.pop() {
            let old_color = write.get_pixel32_raw(x, y);
            if old_color == expected_color {
                if x > 0 {
                    pending.push((x - 1, y));
                }
                if y > 0 {
                    pending.push((x, y - 1));
                }
                if x < write.width() - 1 {
                    pending.push((x + 1, y))
                }
                if y < write.height() - 1 {
                    pending.push((x, y + 1));
                }
                write.set_pixel32_raw(x, y, replace_color);
                dirty_region.encompass(x, y);
            }
        }
    }
    write.set_cpu_dirty(dirty_region);
}

pub fn noise<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    seed: i32,
    low: u8,
    high: u8,
    channel_options: ChannelOptions,
    gray_scale: bool,
) {
    if target.disposed() {
        return;
    }
    let target = target.sync();
    let mut write = target.write(context.gc_context);

    let true_seed = if seed <= 0 {
        (-seed + 1) as u32
    } else {
        seed as u32
    };

    let mut rng = LehmerRng::with_seed(true_seed);

    for y in 0..write.height() {
        for x in 0..write.width() {
            let pixel_color = if gray_scale {
                let gray = rng.gen_range(low..high);
                let alpha = if channel_options.contains(ChannelOptions::ALPHA) {
                    rng.gen_range(low..high)
                } else {
                    255
                };

                Color::argb(alpha, gray, gray, gray)
            } else {
                let r = if channel_options.contains(ChannelOptions::RED) {
                    rng.gen_range(low..high)
                } else {
                    0
                };

                let g = if channel_options.contains(ChannelOptions::GREEN) {
                    rng.gen_range(low..high)
                } else {
                    0
                };

                let b = if channel_options.contains(ChannelOptions::BLUE) {
                    rng.gen_range(low..high)
                } else {
                    0
                };

                let a = if channel_options.contains(ChannelOptions::ALPHA) {
                    rng.gen_range(low..high)
                } else {
                    255
                };

                Color::argb(a, r, g, b)
            };

            write.set_pixel32_raw(x, y, pixel_color);
        }
    }
    let region = PixelRegion::for_whole_size(write.width(), write.height());
    write.set_cpu_dirty(region);
}

#[allow(clippy::too_many_arguments)]
pub fn perlin_noise<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    base: (f64, f64),
    num_octaves: usize,
    random_seed: i64,
    stitch: bool,
    fractal_noise: bool,
    channel_options: ChannelOptions,
    grayscale: bool,
    offsets: Vec<(f64, f64)>, // must contain `num_octaves` values
) {
    if target.disposed() {
        return;
    }
    let target = target.sync();
    let mut write = target.write(context.gc_context);

    let turb = Turbulence::from_seed(random_seed);

    for y in 0..write.height() {
        for x in 0..write.width() {
            let px = x as f64;
            let py = y as f64;

            let mut noise = [0.0; 4];

            // grayscale mode is different enough to warrant its own branch
            if grayscale {
                noise[0] = turb.turbulence(
                    0,
                    (px, py),
                    (1.0 / base.0, 1.0 / base.1),
                    num_octaves,
                    fractal_noise,
                    stitch,
                    (0.0, 0.0),
                    (write.width() as f64, write.height() as f64),
                    &offsets,
                );

                noise[1] = noise[0];
                noise[2] = noise[0];

                noise[3] = if channel_options.contains(ChannelOptions::ALPHA) {
                    turb.turbulence(
                        1,
                        (px, py),
                        (1.0 / base.0, 1.0 / base.1),
                        num_octaves,
                        fractal_noise,
                        stitch,
                        (0.0, 0.0),
                        (write.width() as f64, write.height() as f64),
                        &offsets,
                    )
                } else {
                    1.0
                };
            } else {
                // Flash seems to pass the `color_channel` parameter to `turbulence`
                // somewhat strangely. It's not always r=0, g=1, b=2, a=3; instead,
                // it skips incrementing the parameter after channels that are
                // not included in `channel_options`.
                let mut channel = 0;

                for (c, noise_c) in noise.iter_mut().enumerate() {
                    // this will work both in fractal_sum and turbulence "modes",
                    // because of the saturating conversion to u8
                    *noise_c = if c == 3 { 1.0 } else { -1.0 };

                    // `c` is always in 0..4, so `1 << c` is never actually truncated here
                    let c = ChannelOptions::from_bits_truncate(1 << c);
                    if channel_options.contains(c) {
                        *noise_c = turb.turbulence(
                            channel,
                            (px, py),
                            (1.0 / base.0, 1.0 / base.1),
                            num_octaves,
                            fractal_noise,
                            stitch,
                            (0.0, 0.0),
                            (write.width() as f64, write.height() as f64),
                            &offsets,
                        );
                        channel += 1;
                    }
                }
            }

            let mut color = [0_u8; 4];
            for chan in 0..4 {
                // This is precisely how Adobe Flash converts the -1..1 or 0..1 floats to u8.
                // Please don't touch, it was difficult to figure out the exact method. :)
                color[chan] = (if fractal_noise {
                    // Yes, the + 0.5 for correct (nearest) rounding is done before the division by 2.0,
                    // making it technically less correct (I think), but this is how it is!
                    ((noise[chan] * 255.0 + 255.0) + 0.5) / 2.0
                } else {
                    (noise[chan] * 255.0) + 0.5
                }) as u8;
            }

            if !write.transparency() {
                color[3] = 255;
            }

            write.set_pixel32_raw(x, y, Color::argb(color[3], color[0], color[1], color[2]));
        }
    }
    let region = PixelRegion::for_whole_size(write.width(), write.height());
    write.set_cpu_dirty(region);
}
