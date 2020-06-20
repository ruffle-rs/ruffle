use crate::avm1::error::Error;
use crate::avm1::function::Executable;
use crate::avm1::object::Object;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, ScriptObject, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::MutationContext;
use std::convert::Into;

fn allow_domain<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("System.security.allowDomain() not implemented");
    Ok(Value::Undefined.into())
}

fn allow_insecure_domain<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("System.security.allowInsecureDomain() not implemented");
    Ok(Value::Undefined.into())
}

fn load_policy_file<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("System.security.allowInsecureDomain() not implemented");
    Ok(Value::Undefined.into())
}

fn escape_domain<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("System.security.escapeDomain() not implemented");
    Ok(Value::Undefined.into())
}

fn get_sandbox_type<'gc>(
    _avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    Ok(context.system.sandbox_type.to_string().into())
}

fn get_choose_local_swf_path<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("System.security.chooseLocalSwfPath() not implemented");
    Ok(Value::Undefined.into())
}

fn policy_file_resolver<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error<'gc>> {
    log::warn!("System.security.chooseLocalSwfPath() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut security = ScriptObject::object(gc_context, proto);

    security.force_set_function(
        "PolicyFileResolver",
        policy_file_resolver,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    security.force_set_function(
        "allowDomain",
        allow_domain,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    security.force_set_function(
        "allowInsecureDomain",
        allow_insecure_domain,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    security.force_set_function(
        "loadPolicyFile",
        load_policy_file,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    security.force_set_function(
        "escapeDomain",
        escape_domain,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    security.add_property(
        gc_context,
        "sandboxType",
        Executable::Native(get_sandbox_type),
        None,
        EnumSet::empty(),
    );

    security.add_property(
        gc_context,
        "chooseLocalSwfPath",
        Executable::Native(get_choose_local_swf_path),
        None,
        EnumSet::empty(),
    );

    security.into()
}
