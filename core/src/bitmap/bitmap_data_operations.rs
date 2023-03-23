use crate::bitmap::bitmap_data::BitmapDataWrapper;
use crate::context::UpdateContext;

/// AVM1 and AVM2 have a shared set of operations they can perform on BitmapDatas.
/// Instead of directly manipulating the BitmapData in each place, they should call
/// a shared method here which will do it.
///
/// This will allow us to be able to optimise the implementations and share the
/// same code between VMs.

pub fn fill_rect<'gc>(
    context: &mut UpdateContext<'_, 'gc>,
    target: BitmapDataWrapper<'gc>,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: i32,
) {
    if target.disposed() {
        return;
    }

    target
        .sync()
        .write(context.gc_context)
        .fill_rect(x, y, width, height, color.into());
}
