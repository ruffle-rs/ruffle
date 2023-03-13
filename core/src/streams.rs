use gc_arena::Collect;

/// A stream representing download of some (audiovisual) data.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct NetStream {}
