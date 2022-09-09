use crate::avm2::activation::Activation;
use crate::avm2::object::{Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::display_object::TDisplayObject;
use swf::Twips;

/// Implements `stageX`'s getter.
pub fn get_stage_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            let local_x = this
                .get_property(&Multiname::public("localX"), activation)?
                .coerce_to_number(activation)?;

            if local_x.is_nan() {
                return Ok(Value::Number(local_x));
            } else if let Some(target) = evt.target().and_then(|t| t.as_display_object()) {
                let as_twips = Twips::from_pixels(local_x);
                let xformed = target.local_to_global((as_twips, Twips::ZERO)).0;

                return Ok(Value::Number(xformed.to_pixels()));
            } else {
                return Ok(Value::Number(local_x * 0.0));
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `stageY`'s getter.
pub fn get_stage_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    if let Some(this) = this {
        if let Some(evt) = this.as_event() {
            let local_y = this
                .get_property(&Multiname::public("localY"), activation)?
                .coerce_to_number(activation)?;

            if local_y.is_nan() {
                return Ok(Value::Number(local_y));
            } else if let Some(target) = evt.target().and_then(|t| t.as_display_object()) {
                let as_twips = Twips::from_pixels(local_y);
                let xformed = target.local_to_global((Twips::ZERO, as_twips)).1;

                return Ok(Value::Number(xformed.to_pixels()));
            } else {
                return Ok(Value::Number(local_y * 0.0));
            }
        }
    }

    Ok(Value::Undefined)
}
