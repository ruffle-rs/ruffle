pub use crate::display_object::{DisplayObject, TDisplayObject};
pub use swf::{Point, Rectangle};

/// A depth for a Flash display object in AVM1.
/// This is different than defined in `swf`; during execution, clips
/// created from SWF tags have their depth biased to negative numbers,
/// and clips can be dynamically switched by AS to depths in the range of 32-bits.
pub type Depth = i32;
