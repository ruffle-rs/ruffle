//! Object builtin and prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{FunctionObject, Object, TObject};
use crate::avm2::traits::Trait;
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `Object`'s instance initializer.
pub fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// Implements `Object`'s class initializer
pub fn class_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        let mut object_proto = this
            .get_property(this, &QName::dynamic_name("prototype"), activation)?
            .coerce_to_object(activation)?;
        let gc_context = activation.context.gc_context;

        object_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "hasOwnProperty"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(has_own_property, "hasOwnProperty", gc_context),
                None,
                None,
            )
            .into(),
        )?;
        object_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "propertyIsEnumerable"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(property_is_enumerable, "propertyIsEnumerable", gc_context),
                None,
                None,
            )
            .into(),
        )?;
        object_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "setPropertyIsEnumerable"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(
                    set_property_is_enumerable,
                    "setPropertyIsEnumerable",
                    gc_context,
                ),
                None,
                None,
            )
            .into(),
        )?;
        object_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "isPrototypeOf"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(is_prototype_of, "isPrototypeOf", gc_context),
                None,
                None,
            )
            .into(),
        )?;
        object_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "toString"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_string, "toString", gc_context),
                None,
                None,
            )
            .into(),
        )?;
        object_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "toLocaleString"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(to_locale_string, "toLocaleString", gc_context),
                None,
                None,
            )
            .into(),
        )?;
        object_proto.install_dynamic_property(
            gc_context,
            QName::new(Namespace::public(), "valueOf"),
            FunctionObject::from_method(
                activation,
                Method::from_builtin(value_of, "valueOf", gc_context),
                None,
                None,
            )
            .into(),
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `Object.prototype.toString`
fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    this.map(|t| t.to_string(activation.context.gc_context))
        .unwrap_or(Ok(Value::Undefined))
}

/// Implements `Object.prototype.toLocaleString`
fn to_locale_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    this.map(|t| t.to_locale_string(activation.context.gc_context))
        .unwrap_or(Ok(Value::Undefined))
}

/// Implements `Object.prototype.valueOf`
fn value_of<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    this.map(|t| t.value_of(activation.context.gc_context))
        .unwrap_or(Ok(Value::Undefined))
}

/// `Object.prototype.hasOwnProperty`
pub fn has_own_property<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this: Result<Object<'gc>, Error> = this.ok_or_else(|| "No valid this parameter".into());
    let this = this?;
    let name: Result<&Value<'gc>, Error> = args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.coerce_to_string(activation)?;

    if let Some(ns) = this.resolve_any(name)? {
        if !ns.is_private() {
            let qname = QName::new(ns, name);
            return Ok(this.has_own_property(&qname)?.into());
        }
    }

    Ok(false.into())
}

/// `Object.prototype.isPrototypeOf`
pub fn is_prototype_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let search_proto: Result<Object<'gc>, Error> =
        this.ok_or_else(|| "No valid this parameter".into());
    let search_proto = search_proto?;
    let mut target_proto = args.get(0).cloned().unwrap_or(Value::Undefined);

    while let Value::Object(proto) = target_proto {
        if Object::ptr_eq(search_proto, proto) {
            return Ok(true.into());
        }

        target_proto = proto.proto().map(|o| o.into()).unwrap_or(Value::Undefined);
    }

    Ok(false.into())
}

/// `Object.prototype.propertyIsEnumerable`
pub fn property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this: Result<Object<'gc>, Error> = this.ok_or_else(|| "No valid this parameter".into());
    let this = this?;
    let name: Result<&Value<'gc>, Error> = args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.coerce_to_string(activation)?;

    if let Some(ns) = this.resolve_any(name)? {
        if !ns.is_private() {
            let qname = QName::new(ns, name);
            return Ok(this.property_is_enumerable(&qname).into());
        }
    }

    Ok(false.into())
}

/// `Object.prototype.setPropertyIsEnumerable`
pub fn set_property_is_enumerable<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this: Result<Object<'gc>, Error> = this.ok_or_else(|| "No valid this parameter".into());
    let this = this?;
    let name: Result<&Value<'gc>, Error> = args.get(0).ok_or_else(|| "No name specified".into());
    let name = name?.coerce_to_string(activation)?;

    if let Some(Value::Bool(is_enum)) = args.get(1) {
        if let Some(ns) = this.resolve_any(name)? {
            if !ns.is_private() {
                let qname = QName::new(ns, name);
                this.set_local_property_is_enumerable(
                    activation.context.gc_context,
                    &qname,
                    *is_enum,
                )?;
            }
        }
    }

    Ok(Value::Undefined)
}

/// Construct `Object`'s class.
pub fn create_class<'gc>(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let object_class = Class::new(
        QName::new(Namespace::public(), "Object"),
        None,
        Method::from_builtin(instance_init, "<Object instance initializer>", gc_context),
        Method::from_builtin(class_init, "<Object class initializer>", gc_context),
        gc_context,
    );
    let mut write = object_class.write(gc_context);

    write.define_class_trait(Trait::from_const(
        QName::new(Namespace::public(), "length"),
        QName::new(Namespace::public(), "int").into(),
        None,
    ));

    // Fixed traits (in AS3 namespace)
    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("hasOwnProperty", has_own_property),
        ("isPrototypeOf", is_prototype_of),
        ("propertyIsEnumerable", property_is_enumerable),
    ];
    write.define_as3_builtin_instance_methods(gc_context, PUBLIC_INSTANCE_METHODS);

    object_class
}
