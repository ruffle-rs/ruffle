//! `flash.system.ApplicationDomain` class

use crate::avm2::activation::Activation;
use crate::avm2::class::Class;
use crate::avm2::method::{Method, NativeMethod};
use crate::avm2::names::{Namespace, QName};
use crate::avm2::object::{appdomain_deriver, DomainObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use gc_arena::{GcCell, MutationContext};

/// Implements `flash.system.ApplicationDomain`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;
    }

    Ok(Value::Undefined)
}

/// Implements `flash.system.ApplicationDomain`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    Ok(Value::Undefined)
}

/// `currentDomain` static property.
pub fn current_domain<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let globals = activation.scope().map(|s| s.read().globals());
    let appdomain = globals.and_then(|g| g.as_application_domain());
    let appdomain_proto = activation.avm2().prototypes().application_domain;
    let appdomain_constr = activation.avm2().constructors().application_domain;

    if let Some(appdomain) = appdomain {
        return Ok(DomainObject::from_domain(
            activation.context.gc_context,
            appdomain_constr,
            Some(appdomain_proto),
            appdomain,
        )
        .into());
    }

    Ok(Value::Undefined)
}

/// `parentDomain` property
pub fn parent_domain<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        if let Some(parent_domain) = appdomain.parent_domain() {
            let appdomain_proto = activation.avm2().prototypes().application_domain;
            let appdomain_constr = activation.avm2().constructors().application_domain;

            return Ok(DomainObject::from_domain(
                activation.context.gc_context,
                appdomain_constr,
                Some(appdomain_proto),
                parent_domain,
            )
            .into());
        }
    }

    Ok(Value::Null)
}

/// `getDefinition` method
pub fn get_definition<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        let local_name = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| "".into())
            .coerce_to_string(activation)?;
        let qname = QName::new(Namespace::public(), local_name);

        let (qname, mut defined_script) = appdomain
            .get_defining_script(&qname.into())?
            .ok_or_else(|| format!("No definition called {} exists", local_name))?;
        let mut globals = defined_script.globals(&mut activation.context)?;
        let definition = globals.get_property(globals, &qname, activation)?;

        return Ok(definition);
    }

    Ok(Value::Undefined)
}

/// `hasDefinition` method
pub fn has_definition<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        let local_name = args
            .get(0)
            .cloned()
            .unwrap_or_else(|| "".into())
            .coerce_to_string(activation)?;
        let qname = QName::new(Namespace::public(), local_name);

        return Ok(appdomain.has_definition(qname).into());
    }

    Ok(Value::Undefined)
}

/// `domainMemory` property setter
pub fn set_domain_memory<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(Value::Object(arg)) = args.get(0) {
        if let Some(bytearray_obj) = arg.as_bytearray_object() {
            if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
                appdomain.set_domain_memory(activation.context.gc_context, bytearray_obj);
            }
        }
    }

    Ok(Value::Undefined)
}

/// `domainMemory` property getter
pub fn domain_memory<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(appdomain) = this.and_then(|this| this.as_application_domain()) {
        let bytearray_object: Object<'gc> = appdomain.domain_memory().into();
        return Ok(bytearray_object.into());
    }

    Ok(Value::Undefined)
}

/// Construct `ApplicationDomain`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.system"), "ApplicationDomain"),
        Some(QName::new(Namespace::public(), "Object").into()),
        Method::from_builtin(instance_init),
        Method::from_builtin(class_init),
        mc,
    );

    let mut write = class.write(mc);
    write.set_instance_deriver(appdomain_deriver);

    const PUBLIC_CLASS_METHODS: &[(&str, NativeMethod)] = &[
        ("currentDomain", current_domain),
        ("parentDomain", parent_domain),
        ("getDefinition", get_definition),
        ("hasDefinition", has_definition),
    ];
    write.define_public_builtin_class_methods(PUBLIC_CLASS_METHODS);

    const PUBLIC_INSTANCE_PROPERTIES: &[(&str, Option<NativeMethod>, Option<NativeMethod>)] =
        &[("domainMemory", Some(domain_memory), Some(set_domain_memory))];
    write.define_public_builtin_instance_properties(PUBLIC_INSTANCE_PROPERTIES);

    class
}
