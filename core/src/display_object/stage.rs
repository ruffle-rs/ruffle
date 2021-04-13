//! Root stage impl

use crate::context::UpdateContext;
use crate::display_object::container::{
    ChildContainer, DisplayObjectContainer, TDisplayObjectContainer,
};
use crate::display_object::{DisplayObject, DisplayObjectBase, TDisplayObject};
use crate::prelude::*;
use crate::types::{Degrees, Percent};
use gc_arena::{Collect, GcCell};

/// The Stage is the root of the display object hierarchy. It contains all AVM1
/// levels as well as AVM2 movies.
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct Stage<'gc>(GcCell<'gc, StageData<'gc>>);

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct StageData<'gc> {
    /// Base properties for all display objects.
    ///
    /// This particular base has additional constraints currently not
    /// expressable by the type system. Notably, this should never have a
    /// parent, as the stage does not respect it.
    base: DisplayObjectBase<'gc>,

    /// The list of all children of the stage.
    ///
    /// Stage children are exposed to AVM1 as `_level*n*` on all stage objects.
    child: ChildContainer<'gc>,
}

impl<'gc> TDisplayObject<'gc> for Stage<'gc> {
    impl_display_object!(base);

    fn id(&self) -> CharacterId {
        u16::MAX
    }

    fn self_bounds(&self) -> BoundingBox {
        Default::default()
    }

    fn as_container(self) -> Option<DisplayObjectContainer<'gc>> {
        Some(self.into())
    }
}

impl<'gc> TDisplayObjectContainer<'gc> for Stage<'gc> {
    impl_display_object_container!(child);
}
