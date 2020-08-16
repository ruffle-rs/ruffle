//! Collect Wrapper Utility

use gc_arena::Collect;

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct CollectWrapper<T>(pub T);
