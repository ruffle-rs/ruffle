pub use crate::avm2::Value as Avm2Value;
pub use crate::bounding_box::BoundingBox;
pub use crate::color_transform::ColorTransform;
pub use crate::display_object::{
    DisplayObject, DisplayObjectContainer, Lists, TDisplayObject, TDisplayObjectContainer,
};
pub use crate::levels::LevelId;
pub use crate::{
    impl_display_object, impl_display_object_container, impl_display_object_sansbounds,
};
pub use log::{error, info, trace, warn};
pub use std::ops::{Bound, RangeBounds};
pub use swf::Matrix;
pub use swf::{CharacterId, Color, Twips};

/// A depth for a Flash display object in AVM1.
/// This is different than defined in `swf`; during execution, clips
/// created from SWF tags have their depth biased to negative numbers,
/// and clips can be dynamically switched by AS to depths in the range of 32-bits.
pub type Depth = i32;
