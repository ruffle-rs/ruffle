use crate::avm2::bytearray::{ByteArrayStorage, EofError};
use crate::avm2::error::range_error;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Activation, Error, Value as Avm2Value};
use crate::bitmap::bitmap_data::{
    BitmapData, BitmapDataDrawError, BitmapDataWrapper, ChannelOptions, Color, IBitmapDrawable,
    LehmerRng, ThresholdOperation,
};
use crate::bitmap::turbulence::Turbulence;
use crate::context::{RenderContext, UpdateContext};
use crate::display_object::TDisplayObject;
use gc_arena::MutationContext;
use ruffle_render::bitmap::PixelRegion;
use ruffle_render::commands::{CommandHandler, CommandList};
use ruffle_render::filters::Filter;
use ruffle_render::matrix::Matrix;
use ruffle_render::quality::StageQuality;
use ruffle_render::transform::Transform;
use std::cell::{Ref, RefMut};
use swf::{BlendMode, ColorTransform, Fixed8, Rectangle, Twips};

/// AVM1 and AVM2 have a shared set of operations they can perform on BitmapDatas.
/// Instead of directly manipulating the BitmapData in each place, they should call
/// a shared method here which will do it.
///
/// This will allow us to be able to optimise the implementations and share the
/// same code between VMs.

pub fn fill_rect<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    color: u32,
) {
    let mut rect = PixelRegion::for_region_i32(x, y, width, height);
    rect.clamp(target.width(), target.height());

    if rect.width() == 0 || rect.height() == 0 {
        return;
    }

    let target = if rect.width() == target.width() && rect.height() == target.height() {
        // If we're filling the whole region, we can discard the gpu data
        target.overwrite_cpu_pixels_from_gpu(mc).0
    } else {
        // If we're filling a partial region, finish any gpu->cpu sync
        target.sync()
    };
    let mut write = target.write(mc);
    let color = Color::from(color).to_premultiplied_alpha(write.transparency());

    for y in rect.y_min..rect.y_max {
        for x in rect.x_min..rect.x_max {
            write.set_pixel32_raw(x, y, color);
        }
    }
    write.set_cpu_dirty(rect);
}

pub fn set_pixel32<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    x: u32,
    y: u32,
    color: u32,
) {
    if x >= target.width() || y >= target.height() {
        return;
    }
    let target = target.sync();
    let mut write = target.write(mc);
    let transparency = write.transparency();
    write.set_pixel32_raw(
        x,
        y,
        Color::from(color).to_premultiplied_alpha(transparency),
    );
    write.set_cpu_dirty(PixelRegion::for_pixel(x, y));
}

pub fn get_pixel32(target: BitmapDataWrapper, x: u32, y: u32) -> u32 {
    if x >= target.width() || y >= target.height() {
        return 0;
    }
    let read = target.read_area(PixelRegion::for_pixel(x, y));
    read.get_pixel32_raw(x, y).to_un_multiplied_alpha().into()
}

pub fn set_pixel<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    x: u32,
    y: u32,
    color: Color,
) {
    if x >= target.width() || y >= target.height() {
        return;
    }
    let target = target.sync();
    let mut write = target.write(mc);

    if write.transparency() {
        let current_alpha = write.get_pixel32_raw(x, y).alpha();
        let color = color.with_alpha(current_alpha).to_premultiplied_alpha(true);
        write.set_pixel32_raw(x, y, color);
    } else {
        write.set_pixel32_raw(x, y, color.with_alpha(0xFF));
    }
    write.set_cpu_dirty(PixelRegion::for_whole_size(x, y));
}

pub fn get_pixel(target: BitmapDataWrapper, x: u32, y: u32) -> u32 {
    if x >= target.width() || y >= target.height() {
        return 0;
    }
    let read = target.read_area(PixelRegion::for_pixel(x, y));
    read.get_pixel32_raw(x, y)
        .to_un_multiplied_alpha()
        .with_alpha(0x0)
        .into()
}

pub fn clone(original: BitmapDataWrapper) -> BitmapData {
    // Sync now to bring everything to cpu so we don't force multiple syncs to happen later
    let original = original.sync();
    let read = original.read();
    read.clone()
}

pub fn flood_fill<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    x: u32,
    y: u32,
    color: u32,
) {
    if x >= target.width() || y >= target.height() {
        return;
    }
    let target = target.sync();
    let mut write = target.write(mc);
    let expected_color = write.get_pixel32_raw(x, y);
    let replace_color = Color::from(color).to_premultiplied_alpha(write.transparency());

    if expected_color == replace_color {
        // If we try to replace X with X, we'll infinite loop
        return;
    }

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
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    seed: i32,
    low: u8,
    high: u8,
    channel_options: ChannelOptions,
    gray_scale: bool,
) {
    let (target, _) = target.overwrite_cpu_pixels_from_gpu(mc);
    let mut write = target.write(mc);

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
    mc: MutationContext<'gc, '_>,
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
    let (target, _) = target.overwrite_cpu_pixels_from_gpu(mc);
    let mut write = target.write(mc);

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

pub fn copy_channel<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    dest_point: (i32, i32),
    src_rect: (i32, i32, i32, i32),
    source_bitmap: BitmapDataWrapper<'gc>,
    source_channel: i32,
    dest_channel: i32,
) {
    let (min_x, min_y) = dest_point;
    let (src_min_x, src_min_y, src_width, src_height) = src_rect;

    let channel_shift: u32 = match source_channel {
        // red
        1 => 16,
        // green
        2 => 8,
        // blue
        4 => 0,
        // alpha
        8 => 24,
        _ => 0,
    };
    let transparency = target.transparency();

    let mut source_region =
        PixelRegion::for_whole_size(source_bitmap.width(), source_bitmap.height());
    let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
    dest_region.clamp_with_intersection(
        (min_x, min_y),
        (src_min_x, src_min_y),
        (src_width, src_height),
        &mut source_region,
    );

    if dest_region.width() == 0 || dest_region.height() == 0 {
        return;
    }

    let source = if source_bitmap.ptr_eq(target) {
        None
    } else {
        Some(source_bitmap.read_area(source_region))
    };

    let target = target.sync();
    let mut write = target.write(mc);

    for y in 0..dest_region.height() {
        for x in 0..dest_region.width() {
            let dst_x = dest_region.x_min + x;
            let dst_y = dest_region.y_min + y;

            let original_color: u32 = write
                .get_pixel32_raw(dst_x, dst_y)
                .to_un_multiplied_alpha()
                .into();

            let source_color: u32 = if let Some(source) = &source {
                source.get_pixel32_raw(x, y).to_un_multiplied_alpha().into()
            } else {
                write.get_pixel32_raw(x, y).to_un_multiplied_alpha().into()
            };

            let source_part = (source_color >> channel_shift) & 0xFF;

            let result_color: u32 = match dest_channel {
                // red
                1 => (original_color & 0xFF00FFFF) | source_part << 16,
                // green
                2 => (original_color & 0xFFFF00FF) | source_part << 8,
                // blue
                4 => (original_color & 0xFFFFFF00) | source_part,
                // alpha
                8 => (original_color & 0x00FFFFFF) | source_part << 24,
                _ => original_color,
            };

            write.set_pixel32_raw(
                dst_x,
                dst_y,
                Color::from(result_color).to_premultiplied_alpha(transparency),
            );
        }
    }

    write.set_cpu_dirty(dest_region);
}

pub fn color_transform<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    x_min: u32,
    y_min: u32,
    x_max: u32,
    y_max: u32,
    color_transform: &ColorTransform,
) {
    // Flash bug: applying a color transform with only an alpha multiplier > 1 has no effect.
    if color_transform.r_multiply == Fixed8::ONE
        && color_transform.g_multiply == Fixed8::ONE
        && color_transform.b_multiply == Fixed8::ONE
        && color_transform.a_multiply >= Fixed8::ONE
        && color_transform.r_add == 0
        && color_transform.g_add == 0
        && color_transform.b_add == 0
        && color_transform.a_add == 0
    {
        return;
    }

    let x_max = x_max.min(target.width());
    let y_max = y_max.min(target.height());

    if x_max == 0 || y_max == 0 {
        return;
    }

    let target = target.sync();
    let mut write = target.write(mc);
    let transparency = write.transparency();

    for y in y_min..y_max {
        for x in x_min..x_max {
            let color = write.get_pixel32_raw(x, y).to_un_multiplied_alpha();

            let color = color_transform * swf::Color::from(color);

            write.set_pixel32_raw(
                x,
                y,
                Color::from(color).to_premultiplied_alpha(transparency),
            )
        }
    }
    write.set_cpu_dirty(PixelRegion::encompassing_pixels(
        (x_min, y_min),
        (x_max - 1, y_max - 1),
    ));
}

#[allow(clippy::too_many_arguments)]
pub fn threshold<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    source_bitmap: BitmapDataWrapper<'gc>,
    src_rect: (i32, i32, i32, i32),
    dest_point: (i32, i32),
    operation: ThresholdOperation,
    threshold: u32,
    colour: u32,
    mask: u32,
    copy_source: bool,
) -> u32 {
    // Pre-compute the masked threshold
    let masked_threshold = threshold & mask;

    // Extract coords
    let (src_min_x, src_min_y, src_width, src_height) = src_rect;
    let (dest_min_x, dest_min_y) = dest_point;

    // The number of modified pixels
    // This doesn't seem to include pixels changed due to copy_source
    let mut modified_count = 0;
    let mut dirty_area: Option<PixelRegion> = None;

    let mut source_region =
        PixelRegion::for_whole_size(source_bitmap.width(), source_bitmap.height());
    let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
    dest_region.clamp_with_intersection(
        (dest_min_x, dest_min_y),
        (src_min_x, src_min_y),
        (src_width, src_height),
        &mut source_region,
    );

    if dest_region.width() == 0 || dest_region.height() == 0 {
        return 0;
    }

    let source = if source_bitmap.ptr_eq(target) {
        None
    } else {
        Some(source_bitmap.read_area(source_region))
    };

    let target = target.sync();
    let mut write = target.write(mc);

    // Check each pixel
    for y in 0..dest_region.height() {
        for x in 0..dest_region.width() {
            let dest_x = dest_region.x_min + x;
            let dest_y = dest_region.y_min + y;
            let src_x = source_region.x_min + x;
            let src_y = source_region.y_min + y;

            // Extract source colour
            let source_color = if let Some(source) = &source {
                source
                    .get_pixel32_raw(src_x, src_y)
                    .to_un_multiplied_alpha()
            } else {
                write.get_pixel32_raw(src_x, src_y).to_un_multiplied_alpha()
            };

            // If the test, as defined by the operation pass then set to input colour
            if operation.matches(u32::from(source_color) & mask, masked_threshold) {
                modified_count += 1;
                write.set_pixel32_raw(dest_x, dest_y, Color::from(colour));
            } else {
                // If the test fails, but copy_source is true then take the colour from the source
                if copy_source {
                    let new_color = if let Some(source) = &source {
                        source
                            .get_pixel32_raw(dest_x, dest_y)
                            .to_un_multiplied_alpha()
                    } else {
                        write
                            .get_pixel32_raw(dest_x, dest_y)
                            .to_un_multiplied_alpha()
                    };

                    write.set_pixel32_raw(dest_x, dest_y, new_color);
                }
            }
            if let Some(dirty_area) = &mut dirty_area {
                dirty_area.encompass(dest_x, dest_y);
            } else {
                dirty_area = Some(PixelRegion::for_pixel(dest_x, dest_y));
            }
        }
    }

    if let Some(dirty_area) = dirty_area {
        write.set_cpu_dirty(dirty_area);
    }

    modified_count
}

pub fn scroll<'gc>(mc: MutationContext<'gc, '_>, target: BitmapDataWrapper<'gc>, x: i32, y: i32) {
    let width = target.width() as i32;
    let height = target.height() as i32;

    if (x == 0 && y == 0) || x.abs() >= width || y.abs() >= height {
        return; // no-op
    }

    // since this is an "in-place copy", we have to iterate from bottom to top
    // when scrolling downwards - so if y is positive
    let reverse_y = y > 0;
    // and if only scrolling horizontally, we have to iterate from right to left
    // when scrolling right - so if x is positive
    let reverse_x = y == 0 && x > 0;

    // iteration ranges to use as source for the copy, from is inclusive, to is exclusive
    let y_from = if reverse_y { height - y - 1 } else { -y };
    let y_to = if reverse_y { -1 } else { height };
    let dy = if reverse_y { -1 } else { 1 };

    let x_from = if reverse_x {
        // we know x > 0
        width - x - 1
    } else {
        // x can be any sign
        (-x).max(0)
    };
    let x_to = if reverse_x { -1 } else { width.min(width - x) };
    let dx = if reverse_x { -1 } else { 1 };

    let target = target.sync();
    let mut write = target.write(mc);

    let mut src_y = y_from;
    while src_y != y_to {
        let mut src_x = x_from;
        while src_x != x_to {
            let color = write.get_pixel32_raw(src_x as u32, src_y as u32);
            write.set_pixel32_raw((src_x + x) as u32, (src_y + y) as u32, color);
            src_x += dx;
        }
        src_y += dy;
    }

    let region = PixelRegion::for_whole_size(write.width(), write.height());
    write.set_cpu_dirty(region);
}

pub fn palette_map<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    source_bitmap: BitmapDataWrapper<'gc>,
    src_rect: (i32, i32, i32, i32),
    dest_point: (i32, i32),
    channel_arrays: ([u32; 256], [u32; 256], [u32; 256], [u32; 256]),
) {
    let (src_min_x, src_min_y, src_width, src_height) = src_rect;
    let (dest_min_x, dest_min_y) = dest_point;

    let mut source_region =
        PixelRegion::for_whole_size(source_bitmap.width(), source_bitmap.height());
    let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
    dest_region.clamp_with_intersection(
        (dest_min_x, dest_min_y),
        (src_min_x, src_min_y),
        (src_width, src_height),
        &mut source_region,
    );

    if dest_region.width() == 0 || dest_region.height() == 0 {
        return;
    }

    let source = if source_bitmap.ptr_eq(target) {
        None
    } else {
        Some(source_bitmap.read_area(source_region))
    };

    let target = target.sync();
    let mut write = target.write(mc);

    for y in 0..dest_region.height() {
        for x in 0..dest_region.width() {
            let dest_x = dest_region.x_min + x;
            let dest_y = dest_region.y_min + y;
            let src_x = source_region.x_min + x;
            let src_y = source_region.y_min + y;

            let source_color = if let Some(source) = &source {
                source
                    .get_pixel32_raw(src_x, src_y)
                    .to_un_multiplied_alpha()
            } else {
                write.get_pixel32_raw(src_x, src_y).to_un_multiplied_alpha()
            };

            let r = channel_arrays.0[source_color.red() as usize];
            let g = channel_arrays.1[source_color.green() as usize];
            let b = channel_arrays.2[source_color.blue() as usize];
            let a = channel_arrays.3[source_color.alpha() as usize];

            let sum = u32::wrapping_add(u32::wrapping_add(r, g), u32::wrapping_add(b, a));
            let mix_color = Color::from(sum).to_premultiplied_alpha(true);

            write.set_pixel32_raw(dest_x, dest_y, mix_color);
        }
    }

    write.set_cpu_dirty(dest_region);
}

/// Compare two BitmapData objects.
/// Returns `None` if the bitmaps are equivalent.
pub fn compare<'gc>(
    left: BitmapDataWrapper<'gc>,
    right: BitmapDataWrapper<'gc>,
) -> Option<BitmapData<'gc>> {
    // This function expects that the two bitmaps have the same dimensions.
    // TODO: Relax this assumption and return a special value instead?
    debug_assert_eq!(left.width(), right.width());
    debug_assert_eq!(left.height(), right.height());

    let left = left.sync();
    let left = left.read();
    let right = right.sync();
    let right = right.read();

    let mut different = false;
    let pixels = left
        .pixels()
        .iter()
        .zip(right.pixels())
        .map(|(bitmap_pixel, other_pixel)| {
            let bitmap_pixel = bitmap_pixel.to_un_multiplied_alpha();
            let other_pixel = other_pixel.to_un_multiplied_alpha();
            if bitmap_pixel == other_pixel {
                Color::argb(0, 0, 0, 0)
            } else if bitmap_pixel.with_alpha(0) != other_pixel.with_alpha(0) {
                different = true;
                Color::argb(
                    0xff,
                    bitmap_pixel.red().wrapping_sub(other_pixel.red()),
                    bitmap_pixel.green().wrapping_sub(other_pixel.green()),
                    bitmap_pixel.blue().wrapping_sub(other_pixel.blue()),
                )
            } else {
                different = true;
                let alpha = bitmap_pixel.alpha().wrapping_sub(other_pixel.alpha());
                Color::argb(alpha, alpha, alpha, alpha)
            }
        })
        .collect();

    if different {
        Some(BitmapData::new_with_pixels(
            left.width(),
            left.height(),
            true,
            pixels,
        ))
    } else {
        None
    }
}

pub fn hit_test_point(
    target: BitmapDataWrapper,
    alpha_threshold: u8,
    test_point: (i32, i32),
) -> bool {
    if target.is_point_in_bounds(test_point.0, test_point.1) {
        let x = test_point.0 as u32;
        let y = test_point.1 as u32;
        target
            .read_area(PixelRegion::for_pixel(x, y))
            .get_pixel32_raw(x, y)
            .alpha()
            >= alpha_threshold
    } else {
        false
    }
}

pub fn hit_test_rectangle(
    target: BitmapDataWrapper,
    alpha_threshold: u8,
    top_left: (i32, i32),
    size: (i32, i32),
) -> bool {
    let mut region = PixelRegion::for_region_i32(top_left.0, top_left.1, size.0, size.1);
    region.clamp(target.width(), target.height());
    let read = target.read_area(region);

    for y in region.y_min..region.y_max {
        for x in region.x_min..region.x_max {
            if read.get_pixel32_raw(x, y).alpha() >= alpha_threshold {
                return true;
            }
        }
    }
    false
}

pub fn hit_test_bitmapdata<'gc>(
    target: BitmapDataWrapper<'gc>,
    self_point: (i32, i32),
    self_threshold: u8,
    test: BitmapDataWrapper<'gc>,
    test_point: (i32, i32),
    test_threshold: u8,
) -> bool {
    let xd = test_point.0 - self_point.0;
    let yd = test_point.1 - self_point.1;
    let self_width = target.width() as i32;
    let self_height = target.height() as i32;
    let test_width = test.width() as i32;
    let test_height = test.height() as i32;
    let (self_x0, test_x0, width) = if xd < 0 {
        (
            0,
            (-xd) as u32,
            self_width.min(test_width + xd).max(0) as u32,
        )
    } else {
        (xd as u32, 0, test_width.min(self_width - xd).max(0) as u32)
    };
    let (self_y0, test_y0, height) = if yd < 0 {
        (
            0,
            (-yd) as u32,
            self_height.min(test_height + yd).max(0) as u32,
        )
    } else {
        (
            yd as u32,
            0,
            test_height.min(self_height - yd).max(0) as u32,
        )
    };

    let target = target.read_area(PixelRegion::for_region(self_x0, self_y0, width, height));
    let test = test.read_area(PixelRegion::for_region(test_x0, test_y0, width, height));

    for x in 0..width {
        for y in 0..height {
            let self_is_opaque =
                target.get_pixel32_raw(self_x0 + x, self_y0 + y).alpha() >= self_threshold;
            let test_is_opaque =
                test.get_pixel32_raw(test_x0 + x, test_y0 + y).alpha() >= test_threshold;
            if self_is_opaque && test_is_opaque {
                return true;
            }
        }
    }
    false
}

pub fn color_bounds_rect(
    target: BitmapDataWrapper,
    find_color: bool,
    mask: u32,
    color: u32,
) -> (u32, u32, u32, u32) {
    let mut min_x = target.width();
    let mut max_x = 0;
    let mut min_y = target.height();
    let mut max_y = 0;

    let target = target.sync();
    let read = target.read();

    for x in 0..read.width() {
        for y in 0..read.height() {
            let pixel_raw: u32 = read.get_pixel32_raw(x, y).into();
            let color_matches = if find_color {
                (pixel_raw & mask) == color
            } else {
                (pixel_raw & mask) != color
            };

            if color_matches {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    // Flash treats a match of (0, 0) alone as none.
    if max_x > 0 || max_y > 0 {
        let x = min_x;
        let y = min_y;
        let w = max_x - min_x + 1;
        let h = max_y - min_y + 1;
        (x, y, w, h)
    } else {
        (0, 0, 0, 0)
    }
}

pub fn merge<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    source_bitmap: BitmapDataWrapper<'gc>,
    src_rect: (i32, i32, i32, i32),
    dest_point: (i32, i32),
    rgba_mult: (i32, i32, i32, i32),
) {
    let (src_min_x, src_min_y, src_width, src_height) = src_rect;
    let (dest_min_x, dest_min_y) = dest_point;
    let transparency = target.transparency();

    let mut source_region =
        PixelRegion::for_whole_size(source_bitmap.width(), source_bitmap.height());
    let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
    dest_region.clamp_with_intersection(
        (dest_min_x, dest_min_y),
        (src_min_x, src_min_y),
        (src_width, src_height),
        &mut source_region,
    );

    if dest_region.width() == 0 || dest_region.height() == 0 {
        return;
    }

    let source = if source_bitmap.ptr_eq(target) {
        None
    } else {
        Some(source_bitmap.read_area(source_region))
    };

    let target = target.sync();
    let mut write = target.write(mc);

    for y in 0..dest_region.height() {
        for x in 0..dest_region.width() {
            let dest_x = dest_region.x_min + x;
            let dest_y = dest_region.y_min + y;
            let src_x = source_region.x_min + x;
            let src_y = source_region.y_min + y;

            let source_color = if let Some(source) = &source {
                source
                    .get_pixel32_raw(src_x, src_y)
                    .to_un_multiplied_alpha()
            } else {
                write.get_pixel32_raw(src_x, src_y).to_un_multiplied_alpha()
            };

            let dest_color = write
                .get_pixel32_raw(dest_x, dest_y)
                .to_un_multiplied_alpha();

            let red_mult = rgba_mult.0.clamp(0, 256) as u16;
            let green_mult = rgba_mult.1.clamp(0, 256) as u16;
            let blue_mult = rgba_mult.2.clamp(0, 256) as u16;
            let alpha_mult = rgba_mult.3.clamp(0, 256) as u16;

            let red = (source_color.red() as u16 * red_mult
                + dest_color.red() as u16 * (256 - red_mult))
                / 256;
            let green = (source_color.green() as u16 * green_mult
                + dest_color.green() as u16 * (256 - green_mult))
                / 256;
            let blue = (source_color.blue() as u16 * blue_mult
                + dest_color.blue() as u16 * (256 - blue_mult))
                / 256;
            let alpha = (source_color.alpha() as u16 * alpha_mult
                + dest_color.alpha() as u16 * (256 - alpha_mult))
                / 256;

            let mix_color = Color::argb(alpha as u8, red as u8, green as u8, blue as u8);

            write.set_pixel32_raw(
                dest_x,
                dest_y,
                mix_color.to_premultiplied_alpha(transparency),
            );
        }
    }

    write.set_cpu_dirty(dest_region);
}

pub fn copy_pixels<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    source_bitmap: BitmapDataWrapper<'gc>,
    src_rect: (i32, i32, i32, i32),
    dest_point: (i32, i32),
    merge_alpha: bool,
) {
    let (src_min_x, src_min_y, src_width, src_height) = src_rect;
    let (dest_min_x, dest_min_y) = dest_point;
    let transparency = target.transparency();
    let source_transparency = source_bitmap.transparency();

    let mut source_region =
        PixelRegion::for_whole_size(source_bitmap.width(), source_bitmap.height());
    let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
    dest_region.clamp_with_intersection(
        (dest_min_x, dest_min_y),
        (src_min_x, src_min_y),
        (src_width, src_height),
        &mut source_region,
    );

    if dest_region.width() == 0 || dest_region.height() == 0 {
        return;
    }

    copy_on_cpu(
        context.gc_context,
        source_bitmap,
        target,
        source_region,
        dest_region,
        (source_transparency && !transparency) || merge_alpha,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn copy_pixels_with_alpha_source<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    source_bitmap: BitmapDataWrapper<'gc>,
    src_rect: (i32, i32, i32, i32),
    dest_point: (i32, i32),
    alpha_bitmap: BitmapDataWrapper<'gc>,
    alpha_point: (i32, i32),
    merge_alpha: bool,
) {
    if source_bitmap.ptr_eq(alpha_bitmap)
        && alpha_point.0 == src_rect.0
        && alpha_point.1 == src_rect.1
    {
        copy_pixels(
            context,
            target,
            source_bitmap,
            src_rect,
            dest_point,
            merge_alpha,
        );
        return;
    }

    let (src_min_x, src_min_y, src_width, src_height) = src_rect;
    let (dest_min_x, dest_min_y) = dest_point;
    let transparency = target.transparency();
    let source_transparency = source_bitmap.transparency();
    let alpha_transparency = alpha_bitmap.transparency();

    let mut source_region =
        PixelRegion::for_region_i32(src_min_x, src_min_y, src_width, src_height);
    source_region.clamp(source_bitmap.width(), source_bitmap.height());
    let source_bitmap = if source_bitmap.ptr_eq(target) {
        None
    } else {
        Some(source_bitmap.read_area(source_region))
    };

    let mut alpha_region =
        PixelRegion::for_region_i32(alpha_point.0, alpha_point.1, src_width, src_height);
    alpha_region.clamp(alpha_bitmap.width(), alpha_bitmap.height());
    let alpha_bitmap = if alpha_bitmap.ptr_eq(target) {
        None
    } else {
        Some(alpha_bitmap.read_area(alpha_region))
    };

    let target = target.sync();
    let mut write = target.write(context.gc_context);

    for src_y in src_min_y..(src_min_y + src_height) {
        for src_x in src_min_x..(src_min_x + src_width) {
            let dest_x = src_x - src_min_x + dest_min_x;
            let dest_y = src_y - src_min_y + dest_min_y;

            if !write.is_point_in_bounds(dest_x, dest_y) {
                continue;
            }

            let source_color = if let Some(source_bitmap) = &source_bitmap {
                if !source_bitmap.is_point_in_bounds(src_x, src_y) {
                    continue;
                }
                source_bitmap.get_pixel32_raw(src_x as u32, src_y as u32)
            } else {
                if !write.is_point_in_bounds(src_x, src_y) {
                    continue;
                }
                write.get_pixel32_raw(src_x as u32, src_y as u32)
            };

            let mut dest_color = write.get_pixel32_raw(dest_x as u32, dest_y as u32);

            let alpha_x = src_x - src_min_x + alpha_point.0;
            let alpha_y = src_y - src_min_y + alpha_point.1;

            let final_alpha = if alpha_transparency {
                let a = if let Some(alpha_bitmap) = &alpha_bitmap {
                    if !alpha_bitmap.is_point_in_bounds(alpha_x, alpha_y) {
                        continue;
                    }
                    alpha_bitmap
                        .get_pixel32_raw(alpha_x as u32, alpha_y as u32)
                        .alpha()
                } else {
                    if !write.is_point_in_bounds(alpha_x, alpha_y) {
                        continue;
                    }
                    write
                        .get_pixel32_raw(alpha_x as u32, alpha_y as u32)
                        .alpha()
                };

                if source_transparency {
                    ((a as u16 * source_color.alpha() as u16) >> 8) as u8
                } else {
                    a
                }
            } else if source_transparency {
                source_color.alpha()
            } else {
                255
            };

            // there could be a faster or more accurate way to do this,
            // (without converting to floats and back, twice),
            // but for now this should suffice
            let a = source_color.alpha() as f64 / 255.0;
            let r = (source_color.red() as f64 / a).round() as u8;
            let g = (source_color.green() as f64 / a).round() as u8;
            let b = (source_color.blue() as f64 / a).round() as u8;
            let intermediate_color = Color::argb(source_color.alpha(), r, g, b)
                .with_alpha(final_alpha)
                .to_premultiplied_alpha(true);

            // there are some interesting conditions in the following
            // lines, these are a result of comparing the output in
            // many parameter combinations with that of Adobe's player,
            // and finding patterns in the differences.
            dest_color = if merge_alpha || !transparency {
                dest_color.blend_over(&intermediate_color)
            } else {
                intermediate_color
            };

            write.set_pixel32_raw(dest_x as u32, dest_y as u32, dest_color);
        }
    }
    let mut dirty_region = PixelRegion::encompassing_pixels_i32(
        ((dest_min_x), (dest_min_y)),
        ((dest_min_x + src_width), (dest_min_y + src_height)),
    );
    dirty_region.clamp(write.width(), write.height());
    write.set_cpu_dirty(dirty_region);
}

pub fn apply_filter<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    source: BitmapDataWrapper<'gc>,
    source_point: (u32, u32),
    source_size: (u32, u32),
    dest_point: (u32, u32),
    filter: Filter,
) {
    if !context.renderer.is_filter_supported(&filter) {
        let mut source_region = PixelRegion::for_whole_size(source.width(), source.height());
        let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
        dest_region.clamp_with_intersection(
            (dest_point.0 as i32, dest_point.1 as i32),
            (source_point.0 as i32, source_point.1 as i32),
            (source_size.0 as i32, source_size.1 as i32),
            &mut source_region,
        );

        if dest_region.width() == 0 || dest_region.height() == 0 {
            return;
        }

        // Until we support these filters, treat this like a copy
        copy_on_cpu(
            context.gc_context,
            source,
            target,
            source_region,
            dest_region,
            false,
        );
        return;
    }

    let source_handle = source.bitmap_handle(context.gc_context, context.renderer);
    let (target, _) = target.overwrite_cpu_pixels_from_gpu(context.gc_context);
    let mut write = target.write(context.gc_context);
    let dest = write.bitmap_handle(context.renderer).unwrap();

    let sync_handle = context.renderer.apply_filter(
        source_handle,
        source_point,
        source_size,
        dest,
        dest_point,
        filter,
    );
    let region = PixelRegion::for_whole_size(write.width(), write.height());
    match sync_handle {
        Some(sync_handle) => write.set_gpu_dirty(sync_handle, region),
        None => {
            tracing::warn!("BitmapData.apply_filter: Renderer not yet implemented")
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn copy_on_cpu<'gc>(
    context: MutationContext<'gc, '_>,
    source: BitmapDataWrapper<'gc>,
    dest: BitmapDataWrapper<'gc>,
    source_region: PixelRegion,
    dest_region: PixelRegion,
    mut blend: bool,
) {
    if !source.transparency() {
        // We don't need to blend if we're copying an opaque texture, the alpha would be 255 anyway
        blend = false;
    }
    if !blend && source.ptr_eq(dest) && source_region == dest_region {
        // Copying the same area of self to self, noop
        return;
    }

    if source.ptr_eq(dest) {
        let dest = dest.sync();
        let mut write = dest.write(context);

        for y in 0..dest_region.height() {
            for x in 0..dest_region.width() {
                let mut color =
                    write.get_pixel32_raw(source_region.x_min + x, source_region.y_min + y);
                if blend {
                    color = write
                        .get_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y)
                        .blend_over(&color);
                }
                write.set_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y, color);
            }
        }

        write.set_cpu_dirty(dest_region);
    } else {
        let dest = dest.sync();
        let mut dest_write = dest.write(context);
        let source_read = source.read_area(source_region);

        if !blend && (dest_write.transparency() || !source_read.transparency()) {
            // Copying (not blending) anything to a transparent texture,
            // or copying an opaque texture to an opaque texture,
            // means we can skip alpha premultiplication

            if dest_region == source_region
                && dest_region.width() == dest_write.width()
                && dest_region.height() == dest_write.height()
                && dest_region.width() == source_read.width()
                && dest_region.height() == source_read.height()
            {
                // Copying an entire texture that's the same size and type? Just replace the whole thing
                dest_write
                    .raw_pixels_mut()
                    .copy_from_slice(source_read.raw_pixels());
            } else {
                for y in 0..dest_region.height() {
                    for x in 0..dest_region.width() {
                        let color = source_read
                            .get_pixel32_raw(source_region.x_min + x, source_region.y_min + y);
                        dest_write.set_pixel32_raw(
                            dest_region.x_min + x,
                            dest_region.y_min + y,
                            color,
                        );
                    }
                }
            }
        } else {
            // Copying (not blending) a transparent texture to an opaque texture,
            // or blending anything to anything

            let opaque = !dest_write.transparency();

            for y in 0..dest_region.height() {
                for x in 0..dest_region.width() {
                    let mut color = source_read
                        .get_pixel32_raw(source_region.x_min + x, source_region.y_min + y);
                    if blend {
                        color = dest_write
                            .get_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y)
                            .blend_over(&color);
                    }
                    if opaque {
                        color = color.with_alpha(255);
                    }
                    dest_write.set_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y, color);
                }
            }
        }

        dest_write.set_cpu_dirty(dest_region);
    }
}

#[allow(clippy::too_many_arguments)]
fn blend_and_transform<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    source: BitmapDataWrapper<'gc>,
    dest: BitmapDataWrapper<'gc>,
    source_region: PixelRegion,
    dest_region: PixelRegion,
    transform: &ColorTransform,
) {
    if source.ptr_eq(dest) {
        let dest = dest.sync();
        let mut write = dest.write(context.gc_context);

        for y in 0..dest_region.height() {
            for x in 0..dest_region.width() {
                let mut color =
                    write.get_pixel32_raw(source_region.x_min + x, source_region.y_min + y);
                color = Color::from(transform * swf::Color::from(color.to_un_multiplied_alpha()))
                    .to_premultiplied_alpha(true);
                color = write
                    .get_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y)
                    .blend_over(&color);
                if !write.transparency() {
                    color = color.with_alpha(255);
                }
                write.set_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y, color);
            }
        }

        write.set_cpu_dirty(dest_region);
    } else {
        let dest = dest.sync();
        let mut dest_write = dest.write(context.gc_context);
        let source_read = source.read_area(source_region);
        let opaque = !dest_write.transparency();

        for y in 0..dest_region.height() {
            for x in 0..dest_region.width() {
                let mut color =
                    source_read.get_pixel32_raw(source_region.x_min + x, source_region.y_min + y);
                color = Color::from(transform * swf::Color::from(color.to_un_multiplied_alpha()))
                    .to_premultiplied_alpha(true);
                color = dest_write
                    .get_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y)
                    .blend_over(&color);
                if opaque {
                    color = color.with_alpha(255);
                }
                dest_write.set_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y, color);
            }
        }

        dest_write.set_cpu_dirty(dest_region);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    mut source: IBitmapDrawable<'gc>,
    transform: Transform,
    smoothing: bool,
    blend_mode: BlendMode,
    clip_rect: Option<Rectangle<Twips>>,
    quality: StageQuality,
) -> Result<(), BitmapDataDrawError> {
    // Calculate the maximum potential area that this draw call will affect
    let bounds = transform.matrix * source.bounds();
    let mut dirty_region = PixelRegion::from(bounds);
    dirty_region.clamp(target.width(), target.height());
    if dirty_region.width() == 0 || dirty_region.height() == 0 {
        return Ok(());
    }

    // This can be a simple and efficient blit as long as:
    // - We are using a bitmapdata as a source
    // - We aren't using impactful blend modes
    // - We don't have a scale component of the transform matrix
    if let IBitmapDrawable::BitmapData(source) = &source {
        // This seems to do nothing in Flash, regardless of the pixel contents
        // of the source BitmapData. Note - this is different from drawing a 'Bitmap'
        // with the same underlying 'BitmapData'
        if blend_mode == BlendMode::Alpha || blend_mode == BlendMode::Erase {
            return Ok(());
        }

        if (blend_mode == BlendMode::Normal || blend_mode == BlendMode::Layer)
            && transform.matrix.a == 1.0
            && transform.matrix.b == 0.0
            && transform.matrix.c == 0.0
            && transform.matrix.d == 1.0
        {
            let mut source_region = PixelRegion::for_whole_size(source.width(), source.height());
            let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
            let tx = transform.matrix.tx.to_pixels().floor() as i32;
            let ty = transform.matrix.ty.to_pixels().floor() as i32;

            let (cx, cy, cw, ch) = if let Some(clip_rect) = clip_rect {
                (
                    clip_rect.x_min.to_pixels().floor() as i32,
                    clip_rect.y_min.to_pixels().floor() as i32,
                    clip_rect.width().to_pixels().ceil() as i32,
                    clip_rect.height().to_pixels().ceil() as i32,
                )
            } else {
                (0, 0, target.width() as i32, target.height() as i32)
            };

            dest_region.clamp_with_intersection(
                (cx, cy),
                (cx - tx, cy - ty),
                (cw, ch),
                &mut source_region,
            );

            if transform.color_transform != ColorTransform::default() {
                blend_and_transform(
                    context,
                    *source,
                    target,
                    source_region,
                    dest_region,
                    &transform.color_transform,
                );
            } else {
                copy_on_cpu(
                    context.gc_context,
                    *source,
                    target,
                    source_region,
                    dest_region,
                    source.transparency(), // If transparent source, blend the pixels. Otherwise they'll be 0xFF alpha and nothing to blend.
                );
            }
            return Ok(());
        }
    }

    let mut transform_stack = ruffle_render::transform::TransformStack::new();
    transform_stack.push(&transform);

    let mut render_context = RenderContext {
        renderer: context.renderer,
        commands: CommandList::new(),
        gc_context: context.gc_context,
        library: context.library,
        transform_stack: &mut transform_stack,
        is_offscreen: true,
        stage: context.stage,
    };

    // Make the screen opacity match the opacity of this bitmap

    let clip_mat = clip_rect.map(|clip_rect| {
        // Note - we do *not* apply the matrix to the clip rect,
        // to match Flash's behavior.
        let clip_mat = Matrix {
            a: (clip_rect.x_max - clip_rect.x_min).to_pixels() as f32,
            b: 0.0,
            c: 0.0,
            d: (clip_rect.y_max - clip_rect.y_min).to_pixels() as f32,
            tx: clip_rect.x_min,
            ty: clip_rect.y_min,
        };

        render_context.commands.push_mask();
        // The color doesn't matter, as this is a mask.
        render_context
            .commands
            .draw_rect(swf::Color::BLACK, clip_mat);
        render_context.commands.activate_mask();

        clip_mat
    });

    match &mut source {
        IBitmapDrawable::BitmapData(data) => {
            data.render(smoothing, &mut render_context);
        }
        IBitmapDrawable::DisplayObject(object) => {
            // Note that we do *not* use `render_base`,
            // as we want to ignore the object's mask and normal transform
            object.render_self(&mut render_context);
        }
    }

    if let Some(clip_mat) = clip_mat {
        // Draw the rectangle again after deactivating the mask,
        // to reset the stencil buffer.
        render_context.commands.deactivate_mask();
        render_context
            .commands
            .draw_rect(swf::Color::BLACK, clip_mat);
        render_context.commands.pop_mask();
    }

    let handle = target.bitmap_handle(render_context.gc_context, render_context.renderer);

    let commands = if blend_mode == BlendMode::Normal {
        render_context.commands
    } else {
        let mut commands = CommandList::new();
        commands.blend(render_context.commands, blend_mode);
        commands
    };

    let (target, include_dirty_area) = target.overwrite_cpu_pixels_from_gpu(context.gc_context);
    let mut write = target.write(context.gc_context);
    // If we have another dirty area to preserve, expand this to include it
    if let Some(old) = include_dirty_area {
        dirty_region.union(old);
    }

    let image = context
        .renderer
        .render_offscreen(handle, commands, quality, dirty_region);

    match image {
        Some(sync_handle) => {
            write.set_gpu_dirty(sync_handle, dirty_region);
            Ok(())
        }
        None => Err(BitmapDataDrawError::Unimplemented),
    }
}

pub fn get_vector(
    target: BitmapDataWrapper,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Vec<Avm2Value> {
    let mut region = PixelRegion::for_region_i32(x, y, width, height);
    region.clamp(target.width(), target.height());

    let mut result = Vec::with_capacity((region.width() * region.height()) as usize);

    let read = target.read_area(region);

    for y in region.y_min..region.y_max {
        for x in region.x_min..region.x_max {
            let color = read.get_pixel32_raw(x, y);
            let color = u32::from(color.to_un_multiplied_alpha());
            result.push(color.into());
        }
    }

    result
}

pub fn set_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    x_min: u32,
    y_min: u32,
    x_max: u32,
    y_max: u32,
    vector: &VectorStorage<'gc>,
) -> Result<(), Error<'gc>> {
    // If the vector doesn't contain enough data, no change happens and error immediately.
    let width = (x_max - x_min) as usize;
    let height = (y_max - y_min) as usize;
    if vector.length() < width * height {
        return Err(Error::AvmError(range_error(
            activation,
            "Error #2006: The supplied index is out of bounds.",
            2006,
        )?));
    }

    let region = PixelRegion::for_region(x_min, y_min, width as u32, height as u32);

    let bitmap_data = target.sync();
    let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
    let transparency = bitmap_data.transparency();
    let mut iter = vector.iter();
    bitmap_data.set_cpu_dirty(region);
    for y in region.y_min..region.y_max {
        for x in region.x_min..region.x_max {
            let color = iter
                .next()
                .expect("BitmapData.setVector: Expected element")
                .as_u32(activation.context.gc_context)
                .expect("BitmapData.setVector: Expected uint vector");
            bitmap_data.set_pixel32_raw(
                x,
                y,
                Color::from(color).to_premultiplied_alpha(transparency),
            );
        }
    }
    Ok(())
}

pub fn get_pixels_as_byte_array<'gc>(
    target: BitmapDataWrapper,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<ByteArrayStorage, Error<'gc>> {
    let mut result = ByteArrayStorage::new();
    let mut region = PixelRegion::for_region_i32(x, y, width, height);
    region.clamp(target.width(), target.height());

    let read = target.read_area(region);
    for y in region.y_min..region.y_max {
        for x in region.x_min..region.x_max {
            let color = read.get_pixel32_raw(x, y);
            result.write_unsigned_int(color.to_un_multiplied_alpha().into())?;
        }
    }

    Ok(result)
}

pub fn set_pixels_from_byte_array<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    bytearray: &mut ByteArrayStorage,
) -> Result<(), EofError> {
    let mut region = PixelRegion::for_region_i32(x, y, width, height);
    region.clamp(target.width(), target.height());
    let transparency = target.transparency();

    let target = if region.width() == target.width() && region.height() == target.height() {
        // If we're filling the whole region, we can discard the gpu data
        target.overwrite_cpu_pixels_from_gpu(mc).0
    } else {
        // If we're filling a partial region, finish any gpu->cpu sync
        target.sync()
    };
    let mut write = target.write(mc);

    if region.width() > 0 && region.height() > 0 {
        for y in region.y_min..region.y_max {
            for x in region.x_min..region.x_max {
                // Copy data from bytearray until EOFError or finished
                let color = bytearray.read_unsigned_int()?;
                write.set_pixel32_raw(
                    x,
                    y,
                    Color::from(color).to_premultiplied_alpha(transparency),
                );
            }
        }

        write.set_cpu_dirty(region);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn pixel_dissolve<'gc>(
    mc: MutationContext<'gc, '_>,
    target: BitmapDataWrapper<'gc>,
    source_bitmap: BitmapDataWrapper<'gc>,
    src_rect: (i32, i32, i32, i32),
    dest_point: (i32, i32),
    random_seed: i32,
    num_pixels: i32,
    fill_color: u32,
) -> i32 {
    /// Returns at least 2. Always returns an even number.
    fn get_feistel_block_size(sequence_length: u32) -> u32 {
        let sequence_length = sequence_length.max(2);

        // For the given sequence length, figure out the number of bits required to
        // represent all indices for the sequence. After that, round up to
        // the nearest even number of bits.
        // For instance, for a sequence of length 9, 4 bits are required,
        // and 4 is even, so the result is 4.
        // For a sequence of length 8, 3 bits are required, but 3 is not
        // even, so round up to 4.

        let mut bit_number: u32 = 0;
        let mut num = sequence_length - 1;

        while num > 0 {
            num /= 2;
            bit_number += 1;
        }

        bit_number + (bit_number % 2)
    }

    /// Meant to be a bijective function that takes a raw index and gives the corresponding
    /// Feistel index.
    ///
    /// # Arguments
    ///
    /// * `raw_permutation_index` - Must obey '0 <= permutation_raw_index < permutation_length`.
    /// * `feistel_block_size` - See `get_feistel_block_size()`.
    fn pixel_dissolve_raw_to_feistel_index(
        raw_permutation_index: u32,
        feistel_block_size: u32,
    ) -> u32 {
        // Discussion on Feistel networks:
        // https://github.com/ruffle-rs/ruffle/issues/10962

        // For the simple balanced variant of a Feistel network, an even number of
        // bits for the block size is required (unbalanced Feistel networks
        // also exists, but are presumably more complex).

        // Applying a single round of Feistel.

        let feistel_halfpiece_size = feistel_block_size / 2;

        let halfpiece1 = raw_permutation_index >> feistel_halfpiece_size;
        let halfpiece2 = raw_permutation_index & ((1 << feistel_halfpiece_size) - 1);

        // Apply some function to make the output appear more random.
        // TODO: It would be good to test and improve this heuristic function for creating
        //   random-looking output. One possibility is some decent but fast PRNG. The function can
        //   also take `feistel_block_size` as an argument.
        // This specific function was gotten by trial-and-error on what might make the output look
        // random.
        fn f(num: u32) -> u32 {
            num * num + 1
        }
        let result_before_xor = (f(halfpiece2)) % (1 << feistel_halfpiece_size);

        let new_halfpiece1 = halfpiece2;
        let new_halfpiece2 = halfpiece1 ^ result_before_xor;

        (new_halfpiece2 << feistel_halfpiece_size) | new_halfpiece1
    }

    fn write_pixel(
        write: &mut RefMut<BitmapData>,
        different_source_than_target: &Option<Ref<BitmapData>>,
        fill_color: u32,
        transparency: bool,
        base_point: (u32, u32),
        read_offset: (u32, u32),
        write_offset: (u32, u32),
    ) {
        let read_point = (read_offset.0 + base_point.0, read_offset.1 + base_point.1);
        let write_point = (write_offset.0 + base_point.0, write_offset.1 + base_point.1);

        match different_source_than_target {
            None => {
                write.set_pixel32_raw(
                    write_point.0,
                    write_point.1,
                    Color::from(fill_color).to_premultiplied_alpha(transparency),
                );
            }
            Some(different_source) => {
                write.set_pixel32_raw(
                    write_point.0,
                    write_point.1,
                    different_source.get_pixel32_raw(read_point.0, read_point.1),
                );
            }
        }
    }

    // Apparently,
    // "numPixels:int (default = 0) — The default is 1/30 of the source area (width x height). "
    // is wrong.

    // Extract points and areas.

    let (src_min_x, src_min_y, src_width, src_height) = src_rect;

    let src_width = src_width.max(0);
    let src_height = src_height.max(0);

    if src_width == 0 || src_height == 0 {
        return 0;
    }

    let (dest_min_x, dest_min_y) = dest_point;

    let transparency = target.transparency();

    let mut source_region =
        PixelRegion::for_whole_size(source_bitmap.width(), source_bitmap.height());
    let mut dest_region = PixelRegion::for_whole_size(target.width(), target.height());
    dest_region.clamp_with_intersection(
        (dest_min_x, dest_min_y),
        (src_min_x, src_min_y),
        (src_width, src_height),
        &mut source_region,
    );

    if dest_region.width() == 0 || dest_region.height() == 0 {
        return 0;
    }

    let write_offset = (dest_region.x_min, dest_region.y_min);
    let read_offset = (source_region.x_min, source_region.y_min);

    let different_source_than_target = if source_bitmap.ptr_eq(target) {
        None
    } else {
        Some(source_bitmap.read_area(source_region))
    };

    let final_pixel_sequence_length = dest_region.width() * dest_region.height();

    let num_pixels = num_pixels.min(final_pixel_sequence_length as i32);

    let target = target.sync();
    let mut write = target.write(mc);

    // For compliance with the official Flash Player, we always write the pixel at (0, 0).
    write_pixel(
        &mut write,
        &different_source_than_target,
        fill_color,
        transparency,
        (0, 0),
        read_offset,
        write_offset,
    );

    let feistel_block_size: u32 = get_feistel_block_size(final_pixel_sequence_length);
    let permutation_length = 1 << feistel_block_size;
    // Raw permutation index.
    let mut raw_perm_index = (random_seed % (permutation_length as i32)) as u32;

    for _ in 0..num_pixels {
        // Feistel permutation index.
        let mut feistel_perm_index = 0;

        // Safety mechanism, in case there is a bug in the implementation.
        let mut loop_counter = 0;

        // Find a valid index to write to.
        // Since the pixel at (0, 0) is always written, we always skip `feistel_perm_index == 0`.
        while (feistel_perm_index == 0 || feistel_perm_index >= final_pixel_sequence_length)
            && final_pixel_sequence_length != 1
        {
            raw_perm_index = (raw_perm_index + 1) % permutation_length;

            feistel_perm_index =
                pixel_dissolve_raw_to_feistel_index(raw_perm_index, feistel_block_size);

            loop_counter += 1;

            if loop_counter > permutation_length + 2 {
                // Should never occur, but inserted as a safety mechanism, and the worst possible
                // issue from this is lack of writing all pixels as well as potentially
                // poorer performance.
                break;
            }
        }

        let base_point = (
            feistel_perm_index % dest_region.width(),
            feistel_perm_index / dest_region.width(),
        );

        write_pixel(
            &mut write,
            &different_source_than_target,
            fill_color,
            transparency,
            base_point,
            read_offset,
            write_offset,
        );
    }

    write.set_cpu_dirty(dest_region);

    raw_perm_index as i32
}
