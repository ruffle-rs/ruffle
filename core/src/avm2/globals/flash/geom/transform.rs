use crate::avm2::globals::slots::flash_geom_color_transform as ct_slots;
use crate::avm2::globals::slots::flash_geom_matrix as matrix_slots;
use crate::avm2::globals::slots::flash_geom_matrix_3d as matrix3d_slots;
use crate::avm2::globals::slots::flash_geom_perspective_projection as pp_slots;
use crate::avm2::globals::slots::flash_geom_point as point_slots;
use crate::avm2::globals::slots::flash_geom_transform as transform_slots;
use crate::avm2::object::VectorObject;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::vector::VectorStorage;
use crate::avm2::{Activation, Error, Object, TObject as _, Value};
use crate::display_object::TDisplayObject;
use crate::prelude::{DisplayObject, Matrix, Twips};
use crate::{avm2_stub_getter, avm2_stub_method, avm2_stub_setter};
use ruffle_render::matrix3d::Matrix3D;
use ruffle_render::perspective_projection::PerspectiveProjection;
use ruffle_render::quality::StageQuality;
use swf::{ColorTransform, Fixed8, Rectangle};

fn get_display_object(this: Object<'_>) -> DisplayObject<'_> {
    this.get_slot(transform_slots::DISPLAY_OBJECT)
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
}

pub fn get_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let color_transform = color_transform_from_transform_object(this);
    color_transform_to_object(&color_transform, activation)
}

pub fn set_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let ct = object_to_color_transform(
        args.get_object(activation, 0, "colorTransform")?,
        activation,
    )?;
    let dobj = get_display_object(this);
    dobj.set_color_transform(ct);
    if let Some(parent) = dobj.parent() {
        parent.invalidate_cached_bitmap();
    }
    Ok(Value::Undefined)
}

pub fn get_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if get_display_object(this).base().has_matrix3d_stub() {
        Ok(Value::Null)
    } else {
        let matrix = matrix_from_transform_object(this);
        matrix_to_object(matrix, activation)
    }
}

pub fn set_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let dobj = get_display_object(this);
    let Some(obj) = args.try_get_object(0) else {
        dobj.base().set_has_matrix3d_stub(true);
        return Ok(Value::Undefined);
    };

    let matrix = object_to_matrix(obj, activation)?;
    dobj.set_matrix(matrix);
    if let Some(parent) = dobj.parent() {
        // Self-transform changes are automatically handled,
        // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
        parent.invalidate_cached_bitmap();
    }
    dobj.base().set_has_matrix3d_stub(false);
    Ok(Value::Undefined)
}

pub fn get_concatenated_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let dobj = get_display_object(this);
    let mut node = Some(dobj);
    while let Some(obj) = node {
        if obj.as_stage().is_some() {
            break;
        }
        node = obj.parent();
    }

    // We're a child of the Stage, and not the stage itself
    if node.is_some() && dobj.as_stage().is_none() {
        let matrix = get_display_object(this).local_to_global_matrix_without_own_scroll_rect();
        matrix_to_object(matrix, activation)
    } else {
        // If this object is the Stage itself, or an object
        // that's not a child of the stage, then we need to mimic
        // Flash's bizarre behavior.
        let scale = match activation.context.stage.quality() {
            StageQuality::Low => 20.0,
            StageQuality::Medium => 10.0,
            StageQuality::High | StageQuality::Best => 5.0,
            StageQuality::High8x8 | StageQuality::High8x8Linear => 2.5,
            StageQuality::High16x16 | StageQuality::High16x16Linear => 1.25,
        };

        let mut mat = dobj.base().matrix();
        mat.a *= scale;
        mat.d *= scale;

        matrix_to_object(mat, activation)
    }
}

pub fn has_matrix3d_from_transform_object(transform_object: Object<'_>) -> bool {
    get_display_object(transform_object)
        .base()
        .has_matrix3d_stub()
}

pub fn matrix_from_transform_object(transform_object: Object<'_>) -> Matrix {
    get_display_object(transform_object).base().matrix()
}

pub fn color_transform_from_transform_object(transform_object: Object<'_>) -> ColorTransform {
    get_display_object(transform_object)
        .base()
        .color_transform()
}

// FIXME - handle clamping. We're throwing away precision here in converting to an integer:
// is that what we should be doing?
pub fn object_to_color_transform<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<ColorTransform, Error<'gc>> {
    let red_multiplier = object
        .get_slot(ct_slots::RED_MULTIPLIER)
        .coerce_to_number(activation)?;
    let green_multiplier = object
        .get_slot(ct_slots::GREEN_MULTIPLIER)
        .coerce_to_number(activation)?;
    let blue_multiplier = object
        .get_slot(ct_slots::BLUE_MULTIPLIER)
        .coerce_to_number(activation)?;
    let alpha_multiplier = object
        .get_slot(ct_slots::ALPHA_MULTIPLIER)
        .coerce_to_number(activation)?;
    let red_offset = object
        .get_slot(ct_slots::RED_OFFSET)
        .coerce_to_number(activation)?;
    let green_offset = object
        .get_slot(ct_slots::GREEN_OFFSET)
        .coerce_to_number(activation)?;
    let blue_offset = object
        .get_slot(ct_slots::BLUE_OFFSET)
        .coerce_to_number(activation)?;
    let alpha_offset = object
        .get_slot(ct_slots::ALPHA_OFFSET)
        .coerce_to_number(activation)?;

    Ok(ColorTransform {
        r_multiply: Fixed8::from_f64(red_multiplier),
        g_multiply: Fixed8::from_f64(green_multiplier),
        b_multiply: Fixed8::from_f64(blue_multiplier),
        a_multiply: Fixed8::from_f64(alpha_multiplier),
        r_add: red_offset as i16,
        g_add: green_offset as i16,
        b_add: blue_offset as i16,
        a_add: alpha_offset as i16,
    })
}

pub fn color_transform_to_object<'gc>(
    color_transform: &ColorTransform,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let args = [
        color_transform.r_multiply.to_f64().into(),
        color_transform.g_multiply.to_f64().into(),
        color_transform.b_multiply.to_f64().into(),
        color_transform.a_multiply.to_f64().into(),
        color_transform.r_add.into(),
        color_transform.g_add.into(),
        color_transform.b_add.into(),
        color_transform.a_add.into(),
    ];
    let ct_class = activation.avm2().classes().colortransform;
    let object = ct_class.construct(activation, &args)?;
    Ok(object)
}

pub fn matrix3d_to_object<'gc>(
    matrix: Matrix3D,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let number = activation.avm2().class_defs().number;
    let mut raw_data_storage = VectorStorage::new(16, true, Some(number));
    for (i, data) in matrix.raw_data.iter().enumerate() {
        raw_data_storage.set(i, Value::Number(*data), activation)?;
    }
    let vector = VectorObject::from_vector(raw_data_storage, activation)?.into();
    let object = activation
        .avm2()
        .classes()
        .matrix3d
        .construct(activation, &[vector])?;
    Ok(object)
}

fn object_to_matrix3d<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix3D, Error<'gc>> {
    let raw_data = object
        .get_slot(matrix3d_slots::_RAW_DATA)
        .as_object()
        .expect("rawData cannot be null");
    let raw_data = raw_data
        .as_vector_storage()
        .expect("rawData is not a Vector");
    let raw_data: Vec<f64> = (0..16)
        .map(|i| -> Result<f64, Error<'gc>> { Ok(raw_data.get(i, activation)?.as_f64()) })
        .collect::<Result<Vec<f64>, _>>()?;
    let raw_data = raw_data
        .as_slice()
        .try_into()
        .expect("rawData size must be 16");
    Ok(Matrix3D { raw_data })
}

pub fn object_to_perspective_projection<'gc>(
    object: Object<'gc>,
    _activation: &mut Activation<'_, 'gc>,
) -> Result<PerspectiveProjection, Error<'gc>> {
    if let Some(display_object) = object.get_slot(pp_slots::DISPLAY_OBJECT).as_object() {
        return Ok(display_object
            .as_display_object()
            .unwrap()
            .base()
            .perspective_projection()
            .unwrap_or_default());
    }

    let fov = object.get_slot(pp_slots::FOV).as_f64();

    let center = object.get_slot(pp_slots::CENTER).as_object().unwrap();
    let x = center.get_slot(point_slots::X).as_f64();
    let y = center.get_slot(point_slots::Y).as_f64();

    Ok(PerspectiveProjection {
        field_of_view: fov,
        center: (x, y),
    })
}

pub fn matrix_to_object<'gc>(
    matrix: Matrix,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let args = [
        matrix.a.into(),
        matrix.b.into(),
        matrix.c.into(),
        matrix.d.into(),
        matrix.tx.to_pixels().into(),
        matrix.ty.to_pixels().into(),
    ];
    let object = activation
        .avm2()
        .classes()
        .matrix
        .construct(activation, &args)?;
    Ok(object)
}

pub fn object_to_matrix<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix, Error<'gc>> {
    let a = object
        .get_slot(matrix_slots::A)
        .coerce_to_number(activation)? as f32;
    let b = object
        .get_slot(matrix_slots::B)
        .coerce_to_number(activation)? as f32;
    let c = object
        .get_slot(matrix_slots::C)
        .coerce_to_number(activation)? as f32;
    let d = object
        .get_slot(matrix_slots::D)
        .coerce_to_number(activation)? as f32;
    let tx = Twips::from_pixels(
        object
            .get_slot(matrix_slots::TX)
            .coerce_to_number(activation)?,
    );
    let ty = Twips::from_pixels(
        object
            .get_slot(matrix_slots::TY)
            .coerce_to_number(activation)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}

pub fn get_pixel_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let display_object = get_display_object(this);
    rectangle_to_object(display_object.pixel_bounds(), activation)
}

fn rectangle_to_object<'gc>(
    rectangle: Rectangle<Twips>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let object = activation.avm2().classes().rectangle.construct(
        activation,
        &[
            rectangle.x_min.to_pixels().into(),
            rectangle.y_min.to_pixels().into(),
            rectangle.width().to_pixels().into(),
            rectangle.height().to_pixels().into(),
        ],
    )?;
    Ok(object)
}

pub fn get_matrix_3d<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    // FIXME: This Matrix3D is generated from the 2D Matrix.
    // It does not work when the matrix contains any transformation in 3D.
    // Support native Matrix3D.
    avm2_stub_getter!(activation, "flash.geom.Transform", "matrix3D");

    let display_object = get_display_object(this);
    if display_object.base().has_matrix3d_stub() {
        let matrix = get_display_object(this).base().matrix();
        let mut matrix3d = Matrix3D::from_matrix(matrix);
        matrix3d.set_tz(display_object.z());
        matrix3d_to_object(matrix3d, activation)
    } else {
        Ok(Value::Null)
    }
}

pub fn set_matrix_3d<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    // FIXME: This sets 2D Matrix generated from the given Matrix3D, ignoring 3D parameters.
    // Support native Matrix3D.
    avm2_stub_setter!(activation, "flash.geom.Transform", "matrix3D");

    let display_object = get_display_object(this);

    let (matrix, has_matrix3d, tz) = {
        match args.try_get_object(0) {
            Some(obj) => {
                let matrix3d = object_to_matrix3d(obj, activation)?;
                let matrix = matrix3d.to_matrix();
                let tz = matrix3d.tz();
                (matrix, true, tz)
            }
            None => (Matrix::IDENTITY, false, 0.0),
        }
    };

    display_object.set_matrix(matrix);
    display_object.set_z(tz);
    if let Some(parent) = display_object.parent() {
        // Self-transform changes are automatically handled,
        // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
        parent.invalidate_cached_bitmap();
    }
    display_object.base().set_has_matrix3d_stub(has_matrix3d);

    Ok(Value::Undefined)
}

pub fn get_perspective_projection<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    if get_display_object(this)
        .base()
        .perspective_projection()
        .is_some()
    {
        let object = activation
            .avm2()
            .classes()
            .perspectiveprojection
            .construct(activation, &[])?
            .as_object()
            .unwrap();

        object.set_slot(
            pp_slots::DISPLAY_OBJECT,
            this.get_slot(transform_slots::DISPLAY_OBJECT),
            activation,
        )?;
        Ok(object.into())
    } else {
        Ok(Value::Null)
    }
}

pub fn set_perspective_projection<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    // FIXME: Render with the given PerspectiveProjection.
    avm2_stub_setter!(activation, "flash.geom.Transform", "perspectiveProjection");

    let perspective_projection = args
        .try_get_object(0)
        .map(|object| object_to_perspective_projection(object, activation))
        .transpose()?;

    get_display_object(this).set_perspective_projection(perspective_projection);

    Ok(Value::Undefined)
}

pub fn get_relative_matrix_3d<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();

    let _relative_to = args.get_object(activation, 0, "relativeTo")?;

    avm2_stub_method!(activation, "flash.geom.Transform", "getRelativeMatrix3D");

    let display_object = get_display_object(this);
    if !display_object.base().has_matrix3d_stub() {
        return Ok(Value::Null);
    }

    matrix3d_to_object(Matrix3D::from_matrix(Matrix::IDENTITY), activation)
}
