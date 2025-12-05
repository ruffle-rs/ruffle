use crate::avm2::error::{make_error_2182, make_error_2186};
use crate::avm2::globals::flash::geom::transform::{
    matrix3d_to_object, object_to_perspective_projection,
};
use crate::avm2::globals::slots::flash_geom_perspective_projection as pp_slots;
use crate::avm2::globals::slots::flash_geom_point as point_slots;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::{Activation, Error, Object, TObject as _, Value};
use crate::avm2_stub_setter;
use crate::display_object::TDisplayObject;
use ruffle_render::perspective_projection::PerspectiveProjection;

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
    let this = this.as_object().unwrap();

    let width = get_width(activation, this);
    let focal_length =
        object_to_perspective_projection(this, activation)?.focal_length(width as f32);

    Ok(focal_length.into())
}

pub fn set_focal_length<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME: Render with the given PerspectiveProjection.
    avm2_stub_setter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "focalLength"
    );
    let this = this.as_object().unwrap();

    let focal_length = args.get_f64(0);
    if focal_length <= 0.0 {
        return Err(make_error_2186(activation, focal_length));
    }

    sync_from_display_object(activation, this)?;

    let width = get_width(activation, this);
    let fov = PerspectiveProjection::from_focal_length(focal_length, width).field_of_view;
    this.set_slot(pp_slots::FOV, fov.into(), activation)?;

    sync_to_display_object(this)?;

    Ok(Value::Undefined)
}

pub fn get_field_of_view<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let perspective_projection = object_to_perspective_projection(this, activation)?;

    Ok(perspective_projection.field_of_view.into())
}

pub fn set_field_of_view<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME: Render with the given PerspectiveProjection.
    avm2_stub_setter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "fieldOfView"
    );

    let this = this.as_object().unwrap();

    let fov = args.get_f64(0);
    if fov <= 0.0 || 180.0 <= fov {
        return Err(make_error_2182(activation));
    }

    sync_from_display_object(activation, this)?;

    this.set_slot(pp_slots::FOV, fov.into(), activation)?;

    sync_to_display_object(this)?;

    Ok(Value::Undefined)
}

pub fn get_projection_center<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let perspective_projection = object_to_perspective_projection(this, activation)?;
    let (x, y) = perspective_projection.center;

    activation
        .avm2()
        .classes()
        .point
        .construct(activation, &[x.into(), y.into()])
}

pub fn set_projection_center<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // FIXME: Render with the given PerspectiveProjection.
    avm2_stub_setter!(
        activation,
        "flash.geom.PerspectiveProjection",
        "projectionCenter"
    );

    let this = this.as_object().unwrap();

    sync_from_display_object(activation, this)?;

    let point = args.get_value(0);
    this.set_slot(pp_slots::CENTER, point, activation)?;

    sync_to_display_object(this)?;

    Ok(Value::Undefined)
}

pub fn to_matrix_3d<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let width = get_width(activation, this);
    let matrix3d = object_to_perspective_projection(this, activation)?.to_matrix3d(width as f32);

    matrix3d_to_object(matrix3d, activation)
}

fn sync_from_display_object<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
) -> Result<(), Error<'gc>> {
    let Some(dobj) = this.get_slot(pp_slots::DISPLAY_OBJECT).as_object() else {
        // Not associated with DO. Unnecessary to sync.
        return Ok(());
    };
    let dobj = dobj.as_display_object().unwrap();
    let dobj = dobj.base();

    let Some(perspective_projection) = dobj.perspective_projection() else {
        return Ok(());
    };

    this.set_slot(
        pp_slots::FOV,
        perspective_projection.field_of_view.into(),
        activation,
    )?;

    this.set_slot(
        pp_slots::CENTER,
        activation.avm2().classes().point.construct(
            activation,
            &[
                perspective_projection.center.0.into(),
                perspective_projection.center.1.into(),
            ],
        )?,
        activation,
    )?;

    Ok(())
}

fn sync_to_display_object<'gc>(this: Object<'gc>) -> Result<(), Error<'gc>> {
    let Some(dobj) = this.get_slot(pp_slots::DISPLAY_OBJECT).as_object() else {
        // Not associated with DO. Unnecessary to sync.
        return Ok(());
    };
    let base = dobj.as_display_object().unwrap().base();

    let Some(mut proj) = base.perspective_projection() else {
        return Ok(());
    };

    let fov = this.get_slot(pp_slots::FOV).as_f64();
    let (x, y) = {
        let center = this.get_slot(pp_slots::CENTER).as_object().unwrap();
        let x = center.get_slot(point_slots::X).as_f64();
        let y = center.get_slot(point_slots::Y).as_f64();
        (x, y)
    };

    proj.field_of_view = fov;
    proj.center = (x, y);
    base.set_perspective_projection(Some(proj));

    Ok(())
}
