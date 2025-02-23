//! Object builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::error;
use crate::avm2::method::{Method, NativeMethodImpl, ParamConfig};
use crate::avm2::object::{FunctionObject, Object, ScriptObject, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::{Error, Multiname, QName};
use crate::string::AvmString;

/// Implements `Object`'s instance initializer.
fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `Object`'s custom constructor, called when ActionScript code runs
/// `new Object(...)` directly.
fn object_constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(arg) = args.get(0) {
        if !matches!(arg, Value::Undefined | Value::Null) {
            return Ok(*arg);
        }
    }

    let constructed_object = ScriptObject::new_object(activation);
    Ok(constructed_object.into())
}

fn class_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Calling `Object(...)` is equivalent to constructing `new Object(...)`
    object_constructor(activation, args)
}

/// Implements `Object`'s class initializer
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// This method initializes Object's prototype. We can't do this in the class
/// initializer because the Function class is not yet loaded when Object's class
/// is created. Instead, we call this method in Function's code, right after
/// the Function class is loaded.
pub fn init_object_prototype<'gc>(activation: &mut Activation<'_, 'gc>) -> Result<(), Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.gc();
    let object_class = activation.avm2().classes().object;
    let object_proto = object_class.prototype();

    object_proto.set_string_property_local(
        "hasOwnProperty",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(has_own_property, "hasOwnProperty", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    object_proto.set_string_property_local(
        "propertyIsEnumerable",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(property_is_enumerable, "propertyIsEnumerable", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    object_proto.set_string_property_local(
        "setPropertyIsEnumerable",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(
                set_property_is_enumerable,
                "setPropertyIsEnumerable",
                gc_context,
            ),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    object_proto.set_string_property_local(
        "isPrototypeOf",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(is_prototype_of, "isPrototypeOf", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    object_proto.set_string_property_local(
        "toString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toString", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    object_proto.set_string_property_local(
        "toLocaleString",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(to_string, "toLocaleString", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;
    object_proto.set_string_property_local(
        "valueOf",
        FunctionObject::from_method(
            activation,
            Method::from_builtin(value_of, "valueOf", gc_context),
            scope,
            None,
            None,
            None,
        )
        .into(),
        activation,
    )?;

    object_proto.set_local_property_is_enumerable(gc_context, "hasOwnProperty".into(), false);
    object_proto.set_local_property_is_enumerable(gc_context, "propertyIsEnumerable".into(), false);
    object_proto.set_local_property_is_enumerable(
        gc_context,
        "setPropertyIsEnumerable".into(),
        false,
    );
    object_proto.set_local_property_is_enumerable(gc_context, "isPrototypeOf".into(), false);
    object_proto.set_local_property_is_enumerable(gc_context, "toString".into(), false);
    object_proto.set_local_property_is_enumerable(gc_context, "toLocaleString".into(), false);
    object_proto.set_local_property_is_enumerable(gc_context, "valueOf".into(), false);

    Ok(())
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_object() {
        Ok(this.to_string(activation.gc()).into())
    } else {
        let class_name = this.instance_class(activation).name().local_name();

        Ok(AvmString::new_utf8(activation.gc(), format!("[object {class_name}]")).into())
    }
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this)
}

/// `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get(0).expect("No name specified");
    let name = name.coerce_to_string(activation)?;

    if let Some(this) = this.as_object() {
        Ok(this.has_own_property_string(name, activation)?.into())
    } else {
        let name = Multiname::new(activation.avm2().find_public_namespace(), name);

        Ok(this.has_trait(activation, &name).into())
    }
}

/// `Object.prototype.isPrototypeOf`
pub fn is_prototype_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_object() {
        let mut target_proto = args.get(0).cloned().unwrap_or(Value::Undefined);

        while let Value::Object(proto) = target_proto {
            if Object::ptr_eq(this, proto) {
                return Ok(true.into());
            }

            target_proto = proto.proto().map(|o| o.into()).unwrap_or(Value::Undefined);
        }
    }

    Ok(false.into())
}

/// `Object.prototype.propertyIsEnumerable`
pub fn property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_object() {
        let name = args.get(0).expect("No name specified");
        let name = name.coerce_to_string(activation)?;

        Ok(this.property_is_enumerable(name).into())
    } else {
        Ok(false.into())
    }
}

/// `Object.prototype.setPropertyIsEnumerable`
pub fn set_property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args.get(0).expect("No name specified");
    let name = name.coerce_to_string(activation)?;

    if let Some(this) = this.as_object() {
        if let Some(Value::Bool(is_enum)) = args.get(1) {
            this.set_local_property_is_enumerable(activation.gc(), name, *is_enum);
        }
    } else {
        let instance_class = this.instance_class(activation);
        let multiname = Multiname::new(activation.avm2().find_public_namespace(), name);

        return Err(error::make_reference_error(
            activation,
            error::ReferenceErrorCode::InvalidWrite,
            &multiname,
            instance_class,
        ));
    }

    Ok(Value::Undefined)
}

/// Undocumented `Object.init`, which is a no-op
pub fn init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Construct `Object`'s i_class.
pub fn create_i_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let gc_context = activation.gc();
    let namespaces = activation.avm2().namespaces;

    let object_i_class = Class::custom_new(
        QName::new(namespaces.public_all(), "Object"),
        None,
        Method::from_builtin(instance_init, "<Object instance initializer>", gc_context),
        gc_context,
    );

    object_i_class.set_call_handler(
        gc_context,
        Method::from_builtin(class_call, "<Object call handler>", gc_context),
    );

    object_i_class.set_custom_constructor(gc_context, object_constructor);

    // Fixed traits (in AS3 namespace)
    let as3_instance_methods: Vec<(&str, NativeMethodImpl, _, _)> = vec![
        // These signatures are weird, but they match the describeTypeJSON output
        (
            "hasOwnProperty",
            has_own_property,
            vec![ParamConfig::optional(None, Value::Undefined)],
            Some(activation.avm2().multinames.boolean),
        ),
        (
            "isPrototypeOf",
            is_prototype_of,
            vec![ParamConfig::optional(None, Value::Undefined)],
            Some(activation.avm2().multinames.boolean),
        ),
        (
            "propertyIsEnumerable",
            property_is_enumerable,
            vec![ParamConfig::optional(None, Value::Undefined)],
            Some(activation.avm2().multinames.boolean),
        ),
    ];
    object_i_class.define_builtin_instance_methods_with_sig(
        gc_context,
        namespaces.as3,
        as3_instance_methods,
    );

    object_i_class.mark_traits_loaded(activation.gc());
    object_i_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    object_i_class
}

/// Construct `Object`'s c_class.
pub fn create_c_class<'gc>(
    activation: &mut Activation<'_, 'gc>,
    class_i_class: Class<'gc>,
) -> Class<'gc> {
    let gc_context = activation.gc();
    let namespaces = activation.avm2().namespaces;

    let object_c_class = Class::custom_new(
        QName::new(namespaces.public_all(), "Object$"),
        Some(class_i_class),
        Method::from_builtin(class_init, "<Object class initializer>", gc_context),
        gc_context,
    );
    object_c_class.set_attributes(gc_context, ClassAttributes::FINAL);

    object_c_class.define_instance_trait(
        gc_context,
        Trait::from_const(
            QName::new(activation.avm2().namespaces.public_all(), "length"),
            Some(activation.avm2().multinames.int),
            Some(1.into()),
        ),
    );

    const INTERNAL_INIT_METHOD: &[(&str, NativeMethodImpl)] = &[("init", init)];
    object_c_class.define_builtin_instance_methods(
        gc_context,
        namespaces.internal,
        INTERNAL_INIT_METHOD,
    );

    object_c_class.mark_traits_loaded(activation.gc());
    object_c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    object_c_class
}
