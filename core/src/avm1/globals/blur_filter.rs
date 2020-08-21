//! flash.filter.BlurFilter object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let blur_x = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)
        .unwrap_or(4);

    let blur_y = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)
        .unwrap_or(4);
    //TODO: clamp to [0, 255]

    let quality = args
        .get(3)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)
        .unwrap_or(1);
    //TODO: clamp to [1,3]

    //TODO: check if virt
    this.set("blurX", blur_x.into(), activation)?;
    this.set("blurY", blur_y.into(), activation)?;
    this.set("quality", quality.into(), activation)?;

    Ok(Value::Undefined)
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let proto = activation
        .context
        .avm1
        .prototypes
        .blur_filter_constructor;

    let blur_x = this.get("blurX", activation)?;
    let blur_y = this.get("blurY", activation)?;
    let quality = this.get("quality", activation)?;

    let cloned = proto.construct(
            activation,
            &[
                blur_x,
                blur_y,
                quality
            ],
        )?;
    Ok(cloned.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    //TODO: attributes and rest of funcs
    object.force_set_function("clone", clone, gc_context, EnumSet::empty(), fn_proto);

    object.into()
}
