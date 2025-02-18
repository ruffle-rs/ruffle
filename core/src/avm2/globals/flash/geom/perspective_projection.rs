use std::f64::consts::PI;

use crate::avm2::error::argument_error;
use crate::avm2::globals::slots::flash_geom_perspective_projection as pp_slots;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::display_object::TDisplayObject;
use crate::{avm2_stub_getter, avm2_stub_setter};

const DEG2RAD: f64 = PI / 180.0;

fn get_width<'gc>(activation: &mut Activation<'_, 'gc>, this: Object<'gc>) -> f64 {
    let dobj = this
        .get_slot(pp_slots::DISPLAY_OBJECT)
        .as_object()
        .and_then(|e| e.as_display_object());

    match dobj {
        // Not associated with any DO
        None => 500.0,
        // Stage's PerspectiveProjection
        Some(dobj) if dobj.as_stage().is_some() => 500.0,
        // Associated with other DO.
        Some(_dobj) => activation.context.stage.stage_size().0 as f64,
    }
}

pub fn get_focal_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "focalLength"
    );

    let this = this.as_object().unwrap();

    let fov = this.get_slot(pp_slots::FOV).coerce_to_number(activation)?;

    let width = get_width(activation, this);
    let focal_length = (width / 2.0) / f64::tan(fov / 2.0 * DEG2RAD);

    Ok(focal_length.into())
}

pub fn set_focal_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // TODO: This setter should update the associated displayObject when there is.
    avm2_stub_setter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "focalLength"
    );
    let this = this.as_object().unwrap();

    let focal_length = args.get(0).unwrap().coerce_to_number(activation)?;

    if focal_length <= 0.0 {
        return Err(Error::AvmError(argument_error(
            activation,
            &format!("Error #2186: Invalid focalLength {focal_length}."),
            2186,
        )?));
    }

    let width = get_width(activation, this);
    let fov = f64::atan((width / 2.0) / focal_length) / DEG2RAD * 2.0;

    this.set_slot(pp_slots::FOV, fov.into(), activation)?;

    Ok(Value::Undefined)
}
