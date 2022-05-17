pub use crate::avm2::Value as Avm2Value;
pub use crate::display_object::{
    DisplayObject, DisplayObjectContainer, HitTestOptions, TDisplayObject, TDisplayObjectContainer,
};
pub use log::{error, info, trace, warn};
pub use ruffle_render::bounding_box::BoundingBox;
pub use ruffle_render::color_transform::ColorTransform;
pub use ruffle_render::matrix::Matrix;
pub use std::ops::{Bound, RangeBounds};
pub use swf::{CharacterId, Color, Twips,Twips2d};

/// A depth for a Flash display object in AVM1.
/// This is different than defined in `swf`; during execution, clips
/// created from SWF tags have their depth biased to negative numbers,
/// and clips can be dynamically switched by AS to depths in the range of 32-bits.
pub type Depth = i32;
