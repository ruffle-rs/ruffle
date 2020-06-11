use crate::avm1::{ScriptObject, Avm1, Value, Error, TObject};
use gc_arena::MutationContext;
use crate::avm1::function::{Executable, FunctionObject};
use enumset::EnumSet;
use crate::context::UpdateContext;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::object::Object;


pub fn delete_all<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.deleteAll() not implemented");
    Ok(Value::Undefined.into())
}

pub fn get_disk_usage<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getDiskUsage() not implemented");
    Ok(Value::Undefined.into())
}

pub fn get_local<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {

    let so = _avm.prototypes().shared_object;
    let obj = so.new(_avm, _action_context, so, &[])?;
    let _ = constructor(_avm, _action_context, obj, &[])?;

    // log::warn!("SharedObject.getLocal() not implemented");
    Ok(obj.into())
}

pub fn get_remote<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getRemote() not implemented");
    Ok(Value::Undefined.into())
}

pub fn get_max_size<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getMaxSize() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_shared_object_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    array_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let shared_obj = FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        array_proto,
    );
    let mut object = shared_obj.as_script_object().unwrap();

    object.force_set_function(
        "deleteAll",
        delete_all,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getDiskUsage",
        get_disk_usage,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getLocal",
        get_local,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getRemote",
        get_remote,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );

    object.force_set_function(
        "getMaxSize",
        get_max_size,
        gc_context,
        EnumSet::empty(),
        fn_proto
    );
    // TODO: listeners, might not have _listeners prop

    shared_obj
}

pub fn clear<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.clear() not implemented");
    Ok(Value::Undefined.into())
}

pub fn close<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.close() not implemented");
    Ok(Value::Undefined.into())
}

pub fn connect<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.connect() not implemented");
    Ok(Value::Undefined.into())
}

pub fn flush<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.flush() not implemented");

    let data = _this.get("data", _avm, _action_context).unwrap().as_object().unwrap();

    for key in &data.get_keys(_avm) {
        log::warn!("[SharedObject] {} = {:?}", key, &data.get(key, _avm, _action_context));
    }

    Ok(Value::Undefined.into())
}

pub fn get_size<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.getSize() not implemented");
    Ok(Value::Undefined.into())
}

pub fn send<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.send() not implemented");
    Ok(Value::Undefined.into())
}

pub fn set_fps<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.setFps() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_status<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.onStatus() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_sync<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("SharedObject.onSync() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let shared_obj = ScriptObject::object(gc_context, Some(proto));
    let mut object = shared_obj.as_script_object().unwrap();

    object.force_set_function(
        "clear",
        clear,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "close",
        close,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "connect",
        connect,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "flush",
        flush,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "getSize",
        get_size,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "send",
        send,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "setFps",
        set_fps,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "onStatus",
        on_status,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    object.force_set_function(
        "onSync",
        on_sync,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    shared_obj.into()
}

pub fn constructor<'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {


    let so = avm.prototypes().object;
    let obj = so.new(avm, context, so, &[])?;
    let _ = crate::avm1::globals::object::constructor(avm, context, obj, &[])?;

    //TODO: verify args, esp. this
    this.define_value(
        context.gc_context,
        "data",
        obj.into(),
        EnumSet::empty(),
    );

    Ok(Value::Undefined.into())
}
