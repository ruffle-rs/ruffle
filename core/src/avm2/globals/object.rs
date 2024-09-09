//! Object builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl, ParamConfig};
use crate::avm2::object::{FunctionObject, Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::QName;

/// Implements `Object`'s instance initializer.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

fn class_call<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let object_class = activation.avm2().classes().object;

    if args.is_empty() {
        return object_class.construct(activation, args).map(|o| o.into());
    }
    let arg = args.get(0).cloned().unwrap();
    if matches!(arg, Value::Undefined) || matches!(arg, Value::Null) {
        return object_class.construct(activation, args).map(|o| o.into());
    }
    Ok(arg)
}

/// Implements `Object`'s class initializer
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let scope = activation.create_scopechain();
    let gc_context = activation.context.gc_context;
    let this_class = this.as_class_object().unwrap();
    let object_proto = this_class.prototype();

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
            Method::from_builtin(to_locale_string, "toLocaleString", gc_context),
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

    Ok(Value::Undefined)
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.to_string(activation)
}

/// Implements `Object.prototype.toLocaleString`
fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.to_locale_string(activation)
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    this.value_of(activation.context.gc_context)
}

/// `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name: Result<&Value<'gc>, Error<'gc>> =
        args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.coerce_to_string(activation)?;

    Ok(this.has_own_property_string(name, activation)?.into())
}

/// `Object.prototype.isPrototypeOf`
pub fn is_prototype_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let mut target_proto = args.get(0).cloned().unwrap_or(Value::Undefined);

    while let Value::Object(proto) = target_proto {
        if Object::ptr_eq(this, proto) {
            return Ok(true.into());
        }

        target_proto = proto.proto().map(|o| o.into()).unwrap_or(Value::Undefined);
    }

    Ok(false.into())
}

/// `Object.prototype.propertyIsEnumerable`
pub fn property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name: Result<&Value<'gc>, Error<'gc>> =
        args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.coerce_to_string(activation)?;

    Ok(this.property_is_enumerable(name).into())
}

/// `Object.prototype.setPropertyIsEnumerable`
pub fn set_property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name: Result<&Value<'gc>, Error<'gc>> =
        args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.coerce_to_string(activation)?;

    if let Some(Value::Bool(is_enum)) = args.get(1) {
        this.set_local_property_is_enumerable(activation.context.gc_context, name, *is_enum);
    }

    Ok(Value::Undefined)
}

/// Undocumented `Object.init`, which is a no-op
pub fn init<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Construct `Object`'s i_class.
pub fn create_i_class<'gc>(activation: &mut Activation<'_, 'gc>) -> Class<'gc> {
    let gc_context = activation.context.gc_context;
    let object_i_class = Class::custom_new(
        QName::new(activation.avm2().public_namespace_base_version, "Object"),
        None,
        Method::from_builtin(instance_init, "<Object instance initializer>", gc_context),
        gc_context,
    );

    object_i_class.set_call_handler(
        gc_context,
        Method::from_builtin(class_call, "<Object call handler>", gc_context),
    );

    // Fixed traits (in AS3 namespace)
    let as3_instance_methods: Vec<(&str, NativeMethodImpl, _, _)> = vec![
        // These signatures are weird, but they match the describeTypeJSON output
        (
            "hasOwnProperty",
            has_own_property,
            vec![ParamConfig::optional(
                "name",
                Multiname::any(),
                Value::Undefined,
            )],
            Multiname::new(activation.avm2().public_namespace_base_version, "Boolean"),
        ),
        (
            "isPrototypeOf",
            is_prototype_of,
            vec![ParamConfig::optional(
                "theClass",
                Multiname::any(),
                Value::Undefined,
            )],
            Multiname::new(activation.avm2().public_namespace_base_version, "Boolean"),
        ),
        (
            "propertyIsEnumerable",
            property_is_enumerable,
            vec![ParamConfig::optional(
                "name",
                Multiname::any(),
                Value::Undefined,
            )],
            Multiname::new(activation.avm2().public_namespace_base_version, "Boolean"),
        ),
    ];
    object_i_class.define_builtin_instance_methods_with_sig(
        gc_context,
        activation.avm2().as3_namespace,
        as3_instance_methods,
    );

    object_i_class.mark_traits_loaded(activation.context.gc_context);
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
    let gc_context = activation.context.gc_context;
    let object_c_class = Class::custom_new(
        QName::new(activation.avm2().public_namespace_base_version, "Object$"),
        Some(class_i_class),
        Method::from_builtin(class_init, "<Object class initializer>", gc_context),
        gc_context,
    );
    object_c_class.set_attributes(gc_context, ClassAttributes::FINAL);

    object_c_class.define_instance_trait(
        gc_context,
        Trait::from_const(
            QName::new(activation.avm2().public_namespace_base_version, "length"),
            Multiname::new(activation.avm2().public_namespace_base_version, "int"),
            Some(1.into()),
        ),
    );

    const INTERNAL_INIT_METHOD: &[(&str, NativeMethodImpl)] = &[("init", init)];
    object_c_class.define_builtin_instance_methods(
        gc_context,
        activation.avm2().internal_namespace,
        INTERNAL_INIT_METHOD,
    );

    object_c_class.mark_traits_loaded(activation.context.gc_context);
    object_c_class
        .init_vtable(activation.context)
        .expect("Native class's vtable should initialize");

    object_c_class
}
