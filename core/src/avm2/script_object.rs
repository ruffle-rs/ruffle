//! Default AVM2 object impl

use crate::avm2::names::QName;
use crate::avm2::object::{Object, ObjectPtr, TObject};
use crate::avm2::property::Property;
use crate::avm2::return_value::ReturnValue;
use crate::avm2::value::Value;
use crate::avm2::{Avm2, Error};
use crate::context::UpdateContext;
use gc_arena::{Collect, GcCell, MutationContext};
use std::collections::HashMap;
use std::fmt::Debug;

/// Default implementation of `avm2::Object`.
#[derive(Clone, Collect, Debug, Copy)]
#[collect(no_drop)]
pub struct ScriptObject<'gc>(GcCell<'gc, ScriptObjectData<'gc>>);

#[derive(Clone, Collect, Debug)]
#[collect(no_drop)]
pub struct ScriptObjectData<'gc> {
    /// Properties stored on this object.
    values: HashMap<QName, Property<'gc>>,

    /// Implicit prototype (or declared base class) of this script object.
    proto: Option<Object<'gc>>,
}

impl<'gc> TObject<'gc> for ScriptObject<'gc> {
    fn get_property(
        self,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<ReturnValue<'gc>, Error> {
        self.0.read().get_property(name, avm, context, self.into())
    }

    fn set_property(
        self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<(), Error> {
        self.0
            .write(context.gc_context)
            .set_property(name, value, avm, context, self.into())
    }

    fn has_property(self, name: &QName) -> bool {
        self.0.read().has_property(name)
    }

    fn proto(&self) -> Option<Object<'gc>> {
        self.0.read().proto
    }

    fn as_ptr(&self) -> *const ObjectPtr {
        self.0.as_ptr() as *const ObjectPtr
    }
}

impl<'gc> ScriptObject<'gc> {
    /// Construct a bare object with no base class.
    ///
    /// This is *not* the same thing as an object literal, which actually does
    /// have a base class: `Object`.
    pub fn bare_object(mc: MutationContext<'gc, '_>) -> Object<'gc> {
        ScriptObject(GcCell::allocate(mc, ScriptObjectData::base_new(None))).into()
    }
}

impl<'gc> ScriptObjectData<'gc> {
    pub fn base_new(proto: Option<Object<'gc>>) -> Self {
        ScriptObjectData {
            values: HashMap::new(),
            proto,
        }
    }

    pub fn get_property(
        &self,
        name: &QName,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<ReturnValue<'gc>, Error> {
        let prop = self.values.get(name);

        if let Some(prop) = prop {
            prop.get(avm, context, this)
        } else {
            Ok(Value::Undefined.into())
        }
    }

    pub fn set_property(
        &mut self,
        name: &QName,
        value: Value<'gc>,
        avm: &mut Avm2<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: Object<'gc>,
    ) -> Result<(), Error> {
        if let Some(prop) = self.values.get_mut(name) {
            prop.set(avm, context, this, value)?;
        } else {
            //TODO: Not all classes are dynamic like this
            self.values
                .insert(name.clone(), Property::new_dynamic_property(value));
        }

        Ok(())
    }

    pub fn has_property(&self, name: &QName) -> bool {
        self.values.get(name).is_some()
    }

    pub fn proto(&self) -> Option<Object<'gc>> {
        self.proto
    }
}
