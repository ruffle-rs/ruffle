//! AVM2 object impl for the display hierarchy.

use crate::avm2::activation::Activation;
use crate::avm2::object::script_object::ScriptObjectData;
use crate::avm2::object::{ClassObject, Object, ObjectPtr, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::display_object::DisplayObject;
use gc_arena::{Collect, GcCell, MutationContext};
use std::cell::{Ref, RefMut};
use std::fmt::Debug;

/// A class instance allocator that allocates Stage objects.
pub fn stage_allocator<'gc>(
    class: ClassObject<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Object<'gc>, Error<'gc>> {
    let base = ScriptObjectData::new(class);

    Ok(StageObject(GcCell::allocate(
        activation.context.gc_context,
        StageObjectData {
            base,
            display_object: None,
        },
    ))
    .into())
}

#[derive(Clone, Collect, Copy)]
#[collect(no_drop)]
pub struct StageObject<'gc>(GcCell<'gc, StageObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct StageObjectData<'gc> {
    /// The base data common to all AVM2 objects.
    base: ScriptObjectData<'gc>,

    /// The associated display object, if one exists.
    display_object: Option<DisplayObject<'gc>>,
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
        activation: &mut Activation<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        class: ClassObject<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let mut instance = Self(GcCell::allocate(
            activation.context.gc_context,
            StageObjectData {
                base: ScriptObjectData::new(class),
                display_object: Some(display_object),
            },
        ));
        instance.install_instance_slots(activation);

        Ok(instance)
    }

    /// Allocate and construct the AVM2 side of a display object intended to be
    /// of a given class's type.
    ///
    /// This function is intended for display objects that do not have children
    /// and thus do not need to be allocated and initialized in separate phases.
    pub fn for_display_object_childless(
        activation: &mut Activation<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
        class: ClassObject<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let this = Self::for_display_object(activation, display_object, class)?;

        class.call_native_init(Some(this.into()), &[], activation)?;

        Ok(this)
    }

    /// Create a `graphics` object for a given display object.
    pub fn graphics(
        activation: &mut Activation<'_, 'gc, '_>,
        display_object: DisplayObject<'gc>,
    ) -> Result<Self, Error<'gc>> {
        let class = activation.avm2().classes().graphics;
        let mut this = Self(GcCell::allocate(
            activation.context.gc_context,
            StageObjectData {
                base: ScriptObjectData::new(class),
                display_object: Some(display_object),
            },
        ));
        this.install_instance_slots(activation);

        class.call_native_init(Some(this.into()), &[], activation)?;

        Ok(this)
    }
}

impl<'gc> TObject<'gc> for StageObject<'gc> {
    fn base(&self) -> Ref<ScriptObjectData<'gc>> {
        Ref::map(self.0.read(), |read| &read.base)
    }

    fn base_mut(&self, mc: MutationContext<'gc, '_>) -> RefMut<ScriptObjectData<'gc>> {
        RefMut::map(self.0.write(mc), |write| &mut write.base)
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }

    fn as_display_object(&self) -> Option<DisplayObject<'gc>> {
        self.0.read().display_object
    }

    fn init_display_object(&self, mc: MutationContext<'gc, '_>, obj: DisplayObject<'gc>) {
        self.0.write(mc).display_object = Some(obj);
    }

    fn value_of(&self, _mc: MutationContext<'gc, '_>) -> Result<Value<'gc>, Error<'gc>> {
        Ok(Value::Object(Object::from(*self)))
    }
}

impl<'gc> Debug for StageObject<'gc> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.0.try_read() {
            Ok(obj) => f
                .debug_struct("StageObject")
                .field("name", &obj.base.debug_class_name())
                .field("display_object", &obj.display_object)
                .field("ptr", &self.0.as_ptr())
                .finish(),
            Err(err) => f
                .debug_struct("StageObject")
                .field("name", &err)
                .field("display_object", &err)
                .field("ptr", &self.0.as_ptr())
                .finish(),
        }
    }
}
