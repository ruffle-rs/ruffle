//! AVM2 object impl for the display hierarchy.

use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, TObject};
use crate::avm2::Error;
use crate::display_object::DisplayObject;
use crate::utils::HasPrefixField;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::fmt::Debug;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct StageObject<'gc>(pub Gc<'gc, StageObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct StageObjectWeak<'gc>(pub GcWeak<'gc, StageObjectData<'gc>>);

#[derive(Clone, Collect, HasPrefixField)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct StageObjectData<'gc> {
    /// The base data common to all AVM2 objects.
    base: ScriptObjectData<'gc>,

    /// The associated display object.
    display_object: DisplayObject<'gc>,
}

impl<'gc> StageObject<'gc> {
    /// Allocate the AVM2 side of a display object intended to be of a given
    /// class's type.
    ///
    /// This function makes no attempt to construct the returned object. You
    /// are responsible for calling the native initializer of the given
    /// class at a later time. Typically, a display object that can contain
    /// movie-constructed children must first allocate itself (using this
    /// function), construct it's children, and then finally initialize itself.
    /// Display objects that do not need to use this flow should use
    /// `for_display_object_childless`.
    pub fn for_display_object(
        mc: &Mutation<'gc>,
        display_object: DisplayObject<'gc>,
        class: ClassObject<'gc>,
    ) -> Self {
        Self(Gc::new(
            mc,
            StageObjectData {
                base: ScriptObjectData::new(class),
                display_object,
            },
        ))
    }

    /// Allocate and construct the AVM2 side of a display object intended to be
    /// of a given class's type.
    ///
    /// This function is intended for display objects that do not have children
    /// and thus do not need to be allocated and initialized in separate phases.
    pub fn for_display_object_childless(
        activation: &mut Activation<'_, 'gc>,
        display_object: DisplayObject<'gc>,
        class: ClassObject<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let this = Self::for_display_object(activation.gc(), display_object, class);

        class.call_init(this.into(), FunctionArgs::empty(), activation)?;

        Ok(this)
    }

    /// Create a `graphics` object for a given display object.
    pub fn graphics(
        activation: &mut Activation<'_, 'gc>,
        display_object: DisplayObject<'gc>,
    ) -> Self {
        // note: for Graphics, there's no need to call init.

        let class = activation.avm2().classes().graphics;
        Self(Gc::new(
            activation.gc(),
            StageObjectData {
                base: ScriptObjectData::new(class),
                display_object,
            },
        ))
    }

    pub fn display_object(self) -> DisplayObject<'gc> {
        self.0.display_object
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        HasPrefixField::as_prefix_gc(self.0)
    }
}

impl Debug for StageObject<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("StageObject")
            .field("name", &self.base().class_name())
            // .field("display_object", &self.0.display_object) TODO(moulins)
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}
