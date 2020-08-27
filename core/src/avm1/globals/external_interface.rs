use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use gc_arena::MutationContext;

pub fn get_available<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(activation.context.external_interface.available().into())
}

pub fn create_external_interface_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let object = ScriptObject::object(gc_context, Some(proto));

    object.add_property(
        gc_context,
        "available",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_available),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        Attribute::DontDelete | Attribute::DontEnum,
    );

    object.into()
}

pub fn create_proto<'gc>(gc_context: MutationContext<'gc, '_>, proto: Object<'gc>) -> Object<'gc> {
    // It's a custom prototype but it's empty.
    ScriptObject::object(gc_context, Some(proto)).into()
}
