use std::mem::ManuallyDrop;
use wgpu::RenderPass;

// This is a hack to work around a bug in the borrow checker (https://github.com/rust-lang/rust/issues/70919)
// Under certain circumstances involving loops, rustc will incorrectly think that a local variable
// containing a `Drop` type is still alive (and can therefore access the borrowed data it stores),
// even after an explicit `my_local = NewValue;` or `drop(my_local);`
//
// The wgpu API requires `RenderPass` to have long-lived borrows (e.g. `set_vertex_buffer`),
// so the only way to work around this compiler bug is to use `ManuallyDrop`.
// This modules attempts to hide the details of this approach (as much as possible)
// from the rest of the Context3D implementation.
pub struct RenderPassWrapper<'a>(ManuallyDrop<Option<RenderPass<'a>>>);

impl<'a> RenderPassWrapper<'a> {
    /// Construct a new `RenderPassWrapper`. You should use this instead of `RenderPass::new`.
    /// However, unlike `RenderPass`, you cannot just drop a `RenderPassWrapper` to finish
    /// command recording - you must call `finish_render_pass!` instead.
    pub fn new(render_pass: Option<RenderPass<'a>>) -> Self {
        Self(ManuallyDrop::new(render_pass))
    }
}

impl<'a> std::ops::Deref for RenderPassWrapper<'a> {
    type Target = Option<RenderPass<'a>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<'a> std::ops::DerefMut for RenderPassWrapper<'a> {
    // It's fine to hand out mutable references to the `RenderPass`,
    // as we never call `ManuallyDrop::drop`.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Call this macro to finish recording a render pass.
/// This *must* be used instead of dropping the `RenderPassWrapper` directly,
/// or none of your render commands will actually be recorded.
macro_rules! finish_render_pass {
    ($render_pass:ident) => {
        #[allow(unused_assignments)]
        {
            // Hack to work around the borrow checker bug.
            // This first assignment writes to the inner `&mut Option<RenderPass>`.
            // Unlike the outer `RenderPassWrapper`, this *does* have a `Drop` impl,
            // and actually runs the drop code for `RenderPass`, which finishes command
            // recording.
            *$render_pass = None;
            // The borrow checker doesn't understand that the original
            // value in `render_pass` has been dropped (and can no longer access
            // any borrowed data it was storing).
            // Assign to the local itself to make the borrow checker understand this.
            // Note that because of the `ManuallyDrop`, this *does not* run the `Drop` impl
            // for `RenderPass`. The above `*$render_pass = None` is what actually ends
            // command recording.
            $render_pass = RenderPassWrapper::new(None);
        }
    };
}
pub(crate) use finish_render_pass;
