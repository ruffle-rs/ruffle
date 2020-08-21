//! flash.filter.BitmapFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let proto = activation
        .context
        .avm1
        .prototypes
        .bitmap_filter_constructor;
    let cloned = proto.construct(activation, &[])?;
    Ok(cloned.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    //TODO: attributes
    object.force_set_function("clone", clone, gc_context, EnumSet::empty(), fn_proto);

    object.into()
}
