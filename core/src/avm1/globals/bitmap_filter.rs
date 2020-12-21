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
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this.as_blur_filter_object() {
        let proto = activation.context.avm1.prototypes.blur_filter_constructor;

        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let quality = this.get("quality", activation)?;

        let cloned = proto.construct(activation, &[blur_x, blur_y, quality])?;
        return Ok(cloned.into());
    }

    if let Some(this) = this.as_bevel_filter_object() {
        let proto = activation.context.avm1.prototypes.bevel_filter_constructor;

        let distance = this.get("distance", activation)?;
        let angle = this.get("angle", activation)?;
        let highlight_color = this.get("highlightColor", activation)?;
        let highlight_alpha = this.get("highlightAlpha", activation)?;
        let shadow_color = this.get("shadowColor", activation)?;
        let shadow_alpha = this.get("shadowAlpha", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;
        let type_ = this.get("type", activation)?;
        let knockout = this.get("knockout", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                distance,
                angle,
                highlight_color,
                highlight_alpha,
                shadow_color,
                shadow_alpha,
                blur_x,
                blur_y,
                strength,
                quality,
                type_,
                knockout,
            ],
        )?;
        return Ok(cloned.into());
    }

    if let Some(this) = this.as_glow_filter_object() {
        let proto = activation.context.avm1.prototypes.glow_filter_constructor;

        let color = this.get("color", activation)?;
        let alpha = this.get("alpha", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;

        let cloned = proto.construct(
            activation,
            &[color, alpha, blur_x, blur_y, strength, quality],
        )?;
        return Ok(cloned.into());
    }

    if let Some(this) = this.as_drop_shadow_filter_object() {
        let proto = activation
            .context
            .avm1
            .prototypes
            .drop_shadow_filter_constructor;

        let distance = this.get("distance", activation)?;
        let angle = this.get("angle", activation)?;
        let color = this.get("color", activation)?;
        let alpha = this.get("alpha", activation)?;
        let blur_x = this.get("blurX", activation)?;
        let blur_y = this.get("blurY", activation)?;
        let strength = this.get("strength", activation)?;
        let quality = this.get("quality", activation)?;
        let inner = this.get("inner", activation)?;
        let knockout = this.get("knockout", activation)?;
        let hide_object = this.get("hide_object", activation)?;

        let cloned = proto.construct(
            activation,
            &[
                distance,
                angle,
                color,
                alpha,
                blur_x,
                blur_y,
                strength,
                quality,
                inner,
                knockout,
                hide_object,
            ],
        )?;
        return Ok(cloned.into());
    }

    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut object = ScriptObject::object(gc_context, Some(proto));

    object.force_set_function("clone", clone, gc_context, EnumSet::empty(), fn_proto);

    object.into()
}
