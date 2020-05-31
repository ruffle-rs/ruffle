use crate::avm1::function::Executable;
use crate::avm1::object::Object;
use crate::avm1::property::Attribute::{DontDelete, DontEnum, ReadOnly};
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use gc_arena::MutationContext;
use std::convert::Into;

fn allow_domain<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("system.allowDomain() not implemented");
    Ok(Value::Undefined.into())
}

fn allow_insecure_domain<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("system.allowInsecureDomain() not implemented");
    Ok(Value::Undefined.into())
}

fn load_policy_file<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("system.allowInsecureDomain() not implemented");
    Ok(Value::Undefined.into())
}

fn get_sandbox_type<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(context.system.sandbox_type.get_sandbox_name().into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut security = ScriptObject::object(gc_context, proto);

    security.force_set_function(
        "allowDomain",
        allow_domain,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    security.force_set_function(
        "allowInsecureDomain",
        allow_insecure_domain,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    security.force_set_function(
        "loadPolicyFile",
        load_policy_file,
        gc_context,
        DontDelete | ReadOnly | DontEnum,
        fn_proto,
    );

    security.add_property(
        gc_context,
        "sandboxType",
        Executable::Native(get_sandbox_type),
        None,
        DontDelete | ReadOnly | DontEnum,
    );

    security.into()
}
