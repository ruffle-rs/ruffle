use gc_arena::MutationContext;
use crate::avm1::{Value, ScriptObject, Avm1};
use crate::avm1::function::{FunctionObject, Executable};
use crate::avm1::Object;
use crate::context::UpdateContext;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::error::Error;
use enumset::EnumSet;

pub fn constructor<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    Ok(Value::Undefined.into())
}

pub fn create_context_menu_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    context_menu_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    FunctionObject::function(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        context_menu_proto,
    )
}

pub fn copy<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("ContextMenu.copy() not implemented");
    Ok(Value::Undefined.into())
}

pub fn hide_builtin_items<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("ContextMenu.hideBuiltInItems() not implemented");
    Ok(Value::Undefined.into())
}

pub fn on_select<'gc>(
    _avm: &mut Avm1<'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    log::warn!("ContextMenu.onSelect() not implemented");
    Ok(Value::Undefined.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function(
        "copy",
        copy,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto)
    );

    object.force_set_function(
        "hideBuiltInItems",
        hide_builtin_items,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto)
    );

    object.force_set_function(
        "onSelect",
        on_select,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto)
    );

    object.into()
}
