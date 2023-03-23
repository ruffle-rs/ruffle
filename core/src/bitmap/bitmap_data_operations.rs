use crate::bitmap::bitmap_data::{BitmapDataWrapper, Color};
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

    let target = target.sync();
    let mut write = target.write(context.gc_context);
    let color = Color::from(color).to_premultiplied_alpha(write.transparency());

    for x in rect.min_x..rect.max_x {
        for y in rect.min_y..rect.max_y {
            write.set_pixel32_raw(x, y, color);
        }
    }
    write.set_cpu_dirty(rect);
}
