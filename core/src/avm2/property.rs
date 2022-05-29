//! Property data structures

use crate::avm2::names::Multiname;
use crate::avm2::object::TObject;
use crate::avm2::Activation;
use crate::avm2::ClassObject;
use crate::avm2::Error;
use crate::avm2::TranslationUnit;
use crate::avm2::Value;
use gc_arena::{Collect, Gc};

#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum Property<'gc> {
    Virtual {
        get: Option<u32>,
        set: Option<u32>,
    },
    Method {
        disp_id: u32,
    },
    Slot {
        slot_id: u32,
        class: PropertyClass<'gc>,
    },
    ConstSlot {
        slot_id: u32,
        class: PropertyClass<'gc>,
    },
}

/// The type of a `Slot`/`ConstSlot` property, represented
/// as a lazily-resolved class. This also implements the
/// property-specific coercion logic applied when setting
/// or initializing a property.
///
/// The class resolution needs to be lazy, since we can have
/// a cycle of property type references between classes
/// (e.g. Class1 has `var prop1:Class2`, and Class2
/// has `var prop2:Class1`).
///
/// Additionally, property class resolution uses special
/// logic, different from normal "runtime" class resolution,
/// that allows private types to be referenced.
#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum PropertyClass<'gc> {
    Class(ClassObject<'gc>),
    Name(Gc<'gc, (Multiname<'gc>, Option<TranslationUnit<'gc>>)>),
}

impl<'gc> PropertyClass<'gc> {
    pub fn name(
        activation: &mut Activation<'_, 'gc, '_>,
        name: Multiname<'gc>,
        unit: Option<TranslationUnit<'gc>>,
    ) -> Self {
        PropertyClass::Name(Gc::allocate(activation.context.gc_context, (name, unit)))
    }
    pub fn coerce(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Value<'gc>,
    ) -> Result<Value<'gc>, Error> {
        let class = match self {
            PropertyClass::Class(class) => *class,
            PropertyClass::Name(gc) => {
                let (name, unit) = &**gc;
                let class = resolve_class_private(&name, *unit, activation)?;
                *self = PropertyClass::Class(class);
                class
            }
        };

        // `coerce_to_type` maps `Undefined` to `Null`, but
        // we want it to stay as `Undefined` when setting a property
        match value {
            Value::Undefined => Ok(value),
            _ => value.coerce_to_type(activation, class),
        }
    }
}

/// Resolves a class definition referenced by the type of a property.
/// This supports private (`Namespace::Private`) classes,
/// and does not usethe `ScopeStack`/`ScopeChain`.
///
/// This is an internal operation used to resolve property type names.
/// It does not correspond to any opcode or native method.
fn resolve_class_private<'gc>(
    name: &Multiname<'gc>,
    unit: Option<TranslationUnit<'gc>>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<ClassObject<'gc>, Error> {
    // A Property may have a type of '*' (which corresponds to 'Multiname::any()')
    // In certain cases, we need to be able to distinguish between this
    // and a type of 'Object' (e.g. when determining the default value).
    // Here, it makes no difference.
    if name.is_any() {
        Ok(activation.avm2().classes().object)
    } else {
        // First, check the domain for an exported (non-private) class.
        // If the property we're resolving for lacks a `TranslationUnit`,
        // then it must have come from `load_player_globals`, so we use
        // the top-level `Domain`
        let domain = unit.map_or(activation.avm2().globals, |u| u.domain());
        let globals = if let Some((_, mut script)) = domain.get_defining_script(name)? {
            script.globals(&mut activation.context)?
        } else if let Some(txunit) = unit {
            // If we couldn't find an exported symbol, then check for a
            // private trait in the translation unit. This kind of trait
            // is inaccessible to `resolve_class`.
            //
            // Subtle: `get_loaded_private_trait_script` requires us to have already
            // performed the lazy-loading of `script.globals` for the correct script.
            // Since we are setting/initializing a property with an instance of
            // the class `name`, user bytecode must have already initialized
            // the `ClassObject` in order to have created the value we're setting.
            // Therefore, we don't need to run `script.globals()` for every script
            // in the `TranslationUnit` - we're guaranteed to have already loaded
            // the proper script.
            txunit
                .get_loaded_private_trait_script(name)
                .ok_or_else(|| {
                    Error::from(format!("Could not find script for class trait {:?}", name))
                })?
                .globals(&mut activation.context)?
        } else {
            return Err(format!("Missing script and translation unit for class {:?}", name).into());
        };

        globals
            .get_property(name, activation)?
            .as_object()
            .and_then(|o| o.as_class_object())
            .ok_or_else(|| {
                format!(
                    "Attempted to perform (private) resolution of nonexistent type {:?}",
                    name
                )
                .into()
            })
    }
}

impl<'gc> Property<'gc> {
    pub fn new_method(disp_id: u32) -> Self {
        Property::Method { disp_id }
    }

    pub fn new_getter(disp_id: u32) -> Self {
        Property::Virtual {
            get: Some(disp_id),
            set: None,
        }
    }

    pub fn new_setter(disp_id: u32) -> Self {
        Property::Virtual {
            get: None,
            set: Some(disp_id),
        }
    }

    pub fn new_slot(slot_id: u32, class: PropertyClass<'gc>) -> Self {
        Property::Slot { slot_id, class }
    }

    pub fn new_const_slot(slot_id: u32, class: PropertyClass<'gc>) -> Self {
        Property::ConstSlot { slot_id, class }
    }
}
