use crate::bitmap::bitmap_data::BitmapData;
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
        for y in 0..dest_region.height() {
            for x in 0..dest_region.width() {
                let mut color =
                    dest.get_pixel32_raw(source_region.x_min + x, source_region.y_min + y);
                if blend {
                    color = dest
                        .get_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y)
                        .blend_over(&color);
                }
                dest.set_pixel32_raw(dest_region.x_min + x, dest_region.y_min + y, color);
            }
        }
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
