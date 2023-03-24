use crate::bitmap::bitmap_data::{BitmapDataWrapper, ChannelOptions, Color, LehmerRng};
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
