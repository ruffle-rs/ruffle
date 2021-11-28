//! `flash.system.Security` class

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::Object;
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::string::AvmString;
use gc_arena::{GcCell, MutationContext};

fn instance_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Err("The Security class cannot be constructed.".into())
}

fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

fn sandbox_type<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let sandbox_type = activation.context.system.sandbox_type.to_string();
    return Ok(AvmString::new_utf8(activation.context.gc_context, sandbox_type).into());
}

fn allow_domain<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Security.allowDomain not implemented");
    Ok(Value::Undefined)
}

fn allow_insecure_domain<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Security.allowInsecureDomain not implemented");
    Ok(Value::Undefined)
}

fn load_policy_file<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Security.loadPolicyFile not implemented");
    Ok(Value::Undefined)
}

fn show_settings<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Security.showSettings not implemented");
    Ok(Value::Undefined)
}

pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.system"), "Security"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init, "<Security instance initializer>", mc),
        Method::from_builtin(class_init, "<Security class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);

    const PUBLIC_CLASS_TRAITS: &[(&str, Option<NativeMethodImpl>, Option<NativeMethodImpl>)] =
        &[("sandboxType", Some(sandbox_type), None)];
    write.define_public_builtin_class_properties(mc, PUBLIC_CLASS_TRAITS);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("allowDomain", allow_domain),
        ("allowInsecureDomain", allow_insecure_domain),
        ("loadPolicyFile", load_policy_file),
        ("showSettings", show_settings),
    ];
    write.define_public_builtin_class_methods(mc, PUBLIC_CLASS_METHODS);

    const CONSTANTS: &[(&str, &str)] = &[
        ("APPLICATION", "application"),
        ("LOCAL_TRUSTED", "localTrusted"),
        ("LOCAL_WITH_FILE", "localWithFile"),
        ("LOCAL_WITH_NETWORK", "localWithNetwork"),
        ("REMOTE", "remote"),
    ];
    write.define_public_constant_string_class_traits(CONSTANTS);

    class
}
