//! Property data structures

use crate::avm2::object::TObject;
use crate::avm2::Activation;
use crate::avm2::ClassObject;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::TranslationUnit;
use crate::avm2::Value;
use gc_arena::{Collect, Gc};

#[derive(Debug, Collect, Clone, Copy)]
#[collect(no_drop)]
pub enum Property {
    Virtual { get: Option<u32>, set: Option<u32> },
    Method { disp_id: u32 },
    Slot { slot_id: u32 },
    ConstSlot { slot_id: u32 },
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
#[derive(Debug, Collect, Clone)]
#[collect(no_drop)]
pub enum PropertyClass<'gc> {
    /// The type `*` (Multiname::is_any()). This allows
    /// `Value::Undefined`, so it needs to be distinguished
    /// from the `Object` class
    Any,
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

    /// Returns `value` coerced to the type of this `PropertyClass`.
    /// The bool is `true` if this `PropertyClass` was just modified
    /// to cache a class resolution, and `false` if it was not modified.
    pub fn coerce(
        &mut self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Value<'gc>,
    ) -> Result<(Value<'gc>, bool), Error<'gc>> {
        let (class, changed) = match self {
            PropertyClass::Class(class) => (Some(*class), false),
            PropertyClass::Name(gc) => {
                let (name, unit) = &**gc;
                let outcome = resolve_class_private(name, *unit, activation)?;
                let class = match outcome {
                    ResolveOutcome::Class(class) => {
                        *self = PropertyClass::Class(class);
                        (Some(class), true)
                    }
                    ResolveOutcome::Any => {
                        *self = PropertyClass::Any;
                        (None, true)
                    }
                    ResolveOutcome::NotFound => {
                        // FP allows a class to reference its own type in a static initializer:
                        // `class Foo { static var INSTANCE: Foo = new Foo(); }`
                        // When this happens, the `ClassObject` for `Foo` will not yet
                        // be available when we perform the coercion, since we're still
                        // in the process of constructing it.
                        //
                        // Fortunately, a coercion to a non-primitive class either
                        // succeeds with the value unchanged, or fails (if the object
                        // is not an instance of the class). Therefore, we can just check
                        // if the class name and domain match our property name, without
                        // actually needing to perform resolution. This does not handle subclasses,
                        // but that's fine - a superclass cannot reference a subclass from a class
                        // initializer.
                        //
                        // We should eventually be able to remove this when we refactor
                        // `Class`/`ClassObject` to be closer to what avmplus does.
                        if let Some(object) = value.as_object() {
                            if let Some(class) = object.instance_of() {
                                if name.contains_name(&class.inner_class_definition().read().name())
                                    && unit.map(|u| u.domain())
                                        == Some(class.class_scope().domain())
                                {
                                    // Even though resolution succeeded, we haven't modified
                                    // this `PropertyClass`, so return `false`
                                    return Ok((value, false));
                                }
                            }
                        } else if matches!(value, Value::Null) || matches!(value, Value::Undefined)
                        {
                            //AVM2 properties are nullable, so null is always an instance of the class
                            return Ok((Value::Null, false));
                        }

                        return Err(Error::from(format!(
                            "Attempted to perform (private) resolution of nonexistent type {name:?}",
                        )));
                    }
                };
                class
            }
            PropertyClass::Any => (None, false),
        };

        if let Some(class) = class {
            Ok((value.coerce_to_type(activation, class)?, changed))
        } else {
            // We have a type of '*' ("any"), so don't
            // perform any coercions
            Ok((value, changed))
        }
    }
}

enum ResolveOutcome<'gc> {
    Class(ClassObject<'gc>),
    Any,
    NotFound,
}

/// Resolves a class definition referenced by the type of a property.
/// This supports private (`Namespace::Private`) classes,
/// and does not use the `ScopeStack`/`ScopeChain`.
///
/// This is an internal operation used to resolve property type names.
/// It does not correspond to any opcode or native method.
fn resolve_class_private<'gc>(
    name: &Multiname<'gc>,
    unit: Option<TranslationUnit<'gc>>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<ResolveOutcome<'gc>, Error<'gc>> {
    // A Property may have a type of '*' (which corresponds to 'Multiname::any()')
    // We don't want to perform any coercions in this case - in particular,
    // this means that the property can have a value of `Undefined`.
    // If the type is `Object`, then a value of `Undefind` gets coerced
    // to `Null`
    if name.is_any() {
        Ok(ResolveOutcome::Any)
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
                    Error::from(format!("Could not find script for class trait {name:?}"))
                })?
                .globals(&mut activation.context)?
        } else {
            return Err(format!("Missing script and translation unit for class {name:?}").into());
        };

        Ok(globals
            .get_property(name, activation)?
            .as_object()
            .and_then(|o| o.as_class_object())
            .map_or(ResolveOutcome::NotFound, ResolveOutcome::Class))
    }
}

impl Property {
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

    pub fn new_slot(slot_id: u32) -> Self {
        Property::Slot { slot_id }
    }

    pub fn new_const_slot(slot_id: u32) -> Self {
        Property::ConstSlot { slot_id }
    }
}
