use crate::bitmap::bitmap_data::{BitmapData, BitmapRawData};
use gc_arena::Mutation;
use ruffle_render::backend::RenderBackend;
use ruffle_render::bitmap::PixelRegion;

pub fn copy_on_cpu<'gc>(
    context: &Mutation<'gc>,
    renderer: &mut dyn RenderBackend,
    source: BitmapData<'gc>,
    dest: BitmapData<'gc>,
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

    let dest_is_source = source.ptr_eq(dest);
    let mut dest = dest.sync(renderer).borrow_mut(context);

    if dest_is_source {
        copy_on_cpu_self(&mut dest, source_region, dest_region, blend);
    } else {
        let source = source.read_area(source_region, renderer);

        if !blend && (dest.transparency() || !source.transparency()) {
            // Copying (not blending) anything to a transparent texture,
            // or copying an opaque texture to an opaque texture,
            // means we can skip alpha premultiplication

            if dest_region == source_region
                && dest_region.width() == dest.width()
                && dest_region.height() == dest.height()
                && dest_region.width() == source.width()
                && dest_region.height() == source.height()
            {
                // Copying an entire texture that's the same size and type? Just replace the whole thing
                dest.raw_pixels_mut().copy_from_slice(source.raw_pixels());
            } else {
                let width = dest_region.width() as usize;
                for y in 0..dest_region.height() {
                    let src_start =
                        (source_region.x_min + (source_region.y_min + y) * source.width()) as usize;
                    let dest_start =
                        (dest_region.x_min + (dest_region.y_min + y) * dest.width()) as usize;
                    dest.raw_pixels_mut()[dest_start..dest_start + width]
                        .copy_from_slice(&source.raw_pixels()[src_start..src_start + width]);
                }
            }
        } else {
            // Copying (not blending) a transparent texture to an opaque texture,
            // or blending anything to anything

            let opaque = !dest.transparency();

            let width = dest_region.width() as usize;
            for y in 0..dest_region.height() {
                let src_start =
                    (source_region.x_min + (source_region.y_min + y) * source.width()) as usize;
                let dest_start =
                    (dest_region.x_min + (dest_region.y_min + y) * dest.width()) as usize;
                let dest_row = &mut dest.raw_pixels_mut()[dest_start..dest_start + width];
                let src_row = &source.raw_pixels()[src_start..src_start + width];
                if blend {
                    // Keep the `opaque` check inside the loop: LLVM if-converts it and
                    // vectorizes the whole body, measurably better than two split loops.
                    for (dest, src) in dest_row.iter_mut().zip(src_row) {
                        let mut color = dest.blend_over(src);
                        if opaque {
                            color = color.with_alpha(255);
                        }
                        *dest = color;
                    }
                } else {
                    // `!blend` with a transparent dest always takes the copy branch
                    // above, so the dest is opaque here.
                    debug_assert!(opaque);
                    for (dest, src) in dest_row.iter_mut().zip(src_row) {
                        *dest = src.to_un_multiplied_alpha().with_alpha(255);
                    }
                }
            }
        }
    }

    dest.set_cpu_dirty(context, dest_region);
}

/// Copies `source_region` to `dest_region` within the same bitmap.
fn copy_on_cpu_self(
    data: &mut BitmapRawData<'_>,
    source_region: PixelRegion,
    dest_region: PixelRegion,
    blend: bool,
) {
    // Source and dest regions may overlap so blindly copying the region can
    // overwrite data that hasn't been read yet. That's why the order of
    // iteration actually depends on which direction we're copying the
    // pixels (we have to do the reverse). This applies to both columns and
    // rows separately. However, of course Flash Player has to have a bug
    // here and in certain situations can overwrite the data being copied...
    // When directions match between axes, the order is properly reversed,
    // but when they differ, Flash Player will read overwritten data.
    let dx = dest_region.x_min as i64 - source_region.x_min as i64;
    let dy = dest_region.y_min as i64 - source_region.y_min as i64;
    let reverse = dy >= 0 && dx >= 0;

    if blend {
        copy_on_cpu_self_blend(data, source_region, dest_region, reverse);
    } else {
        copy_on_cpu_self_no_blend(data, source_region, dest_region, reverse);
    }
}

/// Blends `source_region` over `dest_region` within the same bitmap.
///
/// Every pixel is a read-compute-write step, so we can't memmove rows.
fn copy_on_cpu_self_blend(
    data: &mut BitmapRawData<'_>,
    source_region: PixelRegion,
    dest_region: PixelRegion,
    reverse: bool,
) {
    let mut blend_pixel = |x: u32, y: u32| {
        let src_x = source_region.x_min + x;
        let src_y = source_region.y_min + y;
        let dest_x = dest_region.x_min + x;
        let dest_y = dest_region.y_min + y;
        let color = data
            .get_pixel32_raw(dest_x, dest_y)
            .blend_over(&data.get_pixel32_raw(src_x, src_y));
        data.set_pixel32_raw(dest_x, dest_y, color);
    };

    let height = dest_region.height();
    let width = dest_region.width();

    if reverse {
        for y in (0..height).rev() {
            for x in (0..width).rev() {
                blend_pixel(x, y);
            }
        }
    } else {
        for y in 0..height {
            for x in 0..width {
                blend_pixel(x, y);
            }
        }
    }
}

/// Copies `source_region` to `dest_region` within the same bitmap without
/// blending.
///
/// Since we don't need to blend we can memmove rows.
fn copy_on_cpu_self_no_blend(
    data: &mut BitmapRawData<'_>,
    source_region: PixelRegion,
    dest_region: PixelRegion,
    reverse: bool,
) {
    let width = dest_region.width() as usize;
    let bitmap_width = data.width();

    let mut copy_row = |y: u32| {
        let src_start = (source_region.x_min + (source_region.y_min + y) * bitmap_width) as usize;
        let dest_start = (dest_region.x_min + (dest_region.y_min + y) * bitmap_width) as usize;
        data.raw_pixels_mut()
            .copy_within(src_start..src_start + width, dest_start);
    };

    let height = dest_region.height();
    if reverse {
        for y in (0..height).rev() {
            // We don't need to pick the x direction here, copy_within does that
            // for us. Additionally we don't have to care about the FP
            // overwriting behavior here, because it's not observable; row
            // overwriting is observable only when dy=0, but then the direction
            // is always correct.
            copy_row(y);
        }
    } else {
        for y in 0..height {
            copy_row(y);
        }
    }
}
