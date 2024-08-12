//! AVM2 object impl for the display hierarchy.

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::DisplayObject;
use gc_arena::{Collect, Gc, GcWeak, Mutation};
use std::fmt::Debug;

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct StageObject<'gc>(pub Gc<'gc, StageObjectData<'gc>>);

#[derive(Clone, Collect, Copy, Debug)]
#[collect(no_drop)]
pub struct StageObjectWeak<'gc>(pub GcWeak<'gc, StageObjectData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
#[repr(C, align(8))]
pub struct StageObjectData<'gc> {
    /// The base data common to all AVM2 objects.
    base: ScriptObjectData<'gc>,

    /// The associated display object.
    display_object: DisplayObject<'gc>,
}

const _: () = assert!(std::mem::offset_of!(StageObjectData, base) == 0);
const _: () =
    assert!(std::mem::align_of::<StageObjectData>() == std::mem::align_of::<ScriptObjectData>());

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
        activation: &mut Activation<'_, 'gc>,
        display_object: DisplayObject<'gc>,
        class: ClassObject<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let instance = Self(Gc::new(
            activation.context.gc_context,
            StageObjectData {
                base: ScriptObjectData::new(class),
                display_object,
            },
        ));
        instance.install_instance_slots(activation.context.gc_context);

        Ok(instance)
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
        let this = Self::for_display_object(activation, display_object, class)?;

        class.call_native_init(this.into(), &[], activation)?;

        Ok(this)
    }

    /// Same as for_display_object_childless, but allows passing
    /// constructor arguments.
    pub fn for_display_object_childless_with_args(
        activation: &mut Activation<'_, 'gc>,
        display_object: DisplayObject<'gc>,
        class: ClassObject<'gc>,
        args: &[Value<'gc>],
    ) -> Result<Self, Error<'gc>> {
        let this = Self::for_display_object(activation, display_object, class)?;

        class.call_native_init(this.into(), args, activation)?;

        Ok(this)
    }

    /// Create a `graphics` object for a given display object.
    pub fn graphics(
        activation: &mut Activation<'_, 'gc>,
        display_object: DisplayObject<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let class = activation.avm2().classes().graphics;
        let this = Self(Gc::new(
            activation.context.gc_context,
            StageObjectData {
                base: ScriptObjectData::new(class),
                display_object,
            },
        ));
        this.install_instance_slots(activation.context.gc_context);

        // note: for Graphics, there's no need to call init.

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn gc_base(&self) -> Gc<'gc, ScriptObjectData<'gc>> {
        // SAFETY: Object data is repr(C), and a compile-time assert ensures
        // that the ScriptObjectData stays at offset 0 of the struct- so the
        // layouts are compatible

        unsafe { Gc::cast(self.0) }
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        Gc::as_ptr(self.0) as *const ObjectPtr
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        Some(self.0.display_object)
    }

    fn value_of(&self, _mc: &Mutation<'gc>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }
}

impl<'gc> Debug for StageObject<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("StageObject")
            .field("name", &self.base().debug_class_name())
            // .field("display_object", &self.0.display_object) TODO(moulins)
            .field("ptr", &Gc::as_ptr(self.0))
            .finish()
    }
}
