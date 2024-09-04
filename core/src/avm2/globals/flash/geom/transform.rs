use crate::avm2::parameters::ParametersExt;
use crate::avm2::Multiname;
use crate::avm2::{Activation, Error, Object, TObject, Value};
use crate::avm2_stub_getter;
use crate::display_object::TDisplayObject;
use crate::prelude::{DisplayObject, Matrix, Twips};
use ruffle_render::quality::StageQuality;
use swf::{ColorTransform, Fixed8, Rectangle};

fn get_display_object<'gc>(
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<DisplayObject<'gc>, Error<'gc>> {
    let namespaces = activation.avm2().namespaces;

    Ok(this
        .get_property(
            &Multiname::new(namespaces.flash_geom_internal, "_displayObject"),
            activation,
        )?
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap())
}

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let namespaces = activation.avm2().namespaces;

    this.set_property(
        &Multiname::new(namespaces.flash_geom_internal, "_displayObject"),
        args.get_value(0),
        activation,
    )?;
    Ok(Value::Undefined)
}

pub fn get_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let display_object = get_display_object(this, activation)?;
    let display_object = display_object.base();
    color_transform_to_object(display_object.color_transform(), activation)
}

pub fn set_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let ct = object_to_color_transform(args.get_object(activation, 0, "value")?, activation)?;
    let dobj = get_display_object(this, activation)?;
    dobj.set_color_transform(activation.context.gc_context, ct);
    if let Some(parent) = dobj.parent() {
        parent.invalidate_cached_bitmap(activation.context.gc_context);
    }
    Ok(Value::Undefined)
}

pub fn get_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = *get_display_object(this, activation)?.base().matrix();
    matrix_to_object(matrix, activation)
}

pub fn set_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = object_to_matrix(args.get_object(activation, 0, "value")?, activation)?;
    let dobj = get_display_object(this, activation)?;
    dobj.set_matrix(activation.context.gc_context, matrix);
    if let Some(parent) = dobj.parent() {
        // Self-transform changes are automatically handled,
        // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
        parent.invalidate_cached_bitmap(activation.context.gc_context);
    }
    Ok(Value::Undefined)
}

pub fn get_concatenated_matrix<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let dobj = get_display_object(this, activation)?;
    let mut node = Some(dobj);
    while let Some(obj) = node {
        if obj.as_stage().is_some() {
            break;
        }
        node = obj.parent();
    }

    // We're a child of the Stage, and not the stage itself
    if node.is_some() && dobj.as_stage().is_none() {
        let matrix =
            get_display_object(this, activation)?.local_to_global_matrix_without_own_scroll_rect();
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

        let mut mat = *dobj.base().matrix();
        mat.a *= scale;
        mat.d *= scale;

        matrix_to_object(mat, activation)
    }
}

pub fn get_concatenated_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.geom.Transform",
        "concatenatedColorTransform"
    );
    Ok(Value::Undefined)
}

// FIXME - handle clamping. We're throwing away precision here in converting to an integer:
// is that what we should be doing?
pub fn object_to_color_transform<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<ColorTransform, Error<'gc>> {
    let red_multiplier = object
        .get_public_property("redMultiplier", activation)?
        .coerce_to_number(activation)?;
    let green_multiplier = object
        .get_public_property("greenMultiplier", activation)?
        .coerce_to_number(activation)?;
    let blue_multiplier = object
        .get_public_property("blueMultiplier", activation)?
        .coerce_to_number(activation)?;
    let alpha_multiplier = object
        .get_public_property("alphaMultiplier", activation)?
        .coerce_to_number(activation)?;
    let red_offset = object
        .get_public_property("redOffset", activation)?
        .coerce_to_number(activation)?;
    let green_offset = object
        .get_public_property("greenOffset", activation)?
        .coerce_to_number(activation)?;
    let blue_offset = object
        .get_public_property("blueOffset", activation)?
        .coerce_to_number(activation)?;
    let alpha_offset = object
        .get_public_property("alphaOffset", activation)?
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
    Ok(object.into())
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
    Ok(object.into())
}

pub fn object_to_matrix<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Result<Matrix, Error<'gc>> {
    let a = object
        .get_public_property("a", activation)?
        .coerce_to_number(activation)? as f32;
    let b = object
        .get_public_property("b", activation)?
        .coerce_to_number(activation)? as f32;
    let c = object
        .get_public_property("c", activation)?
        .coerce_to_number(activation)? as f32;
    let d = object
        .get_public_property("d", activation)?
        .coerce_to_number(activation)? as f32;
    let tx = Twips::from_pixels(
        object
            .get_public_property("tx", activation)?
            .coerce_to_number(activation)?,
    );
    let ty = Twips::from_pixels(
        object
            .get_public_property("ty", activation)?
            .coerce_to_number(activation)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}

pub fn get_pixel_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let display_object = get_display_object(this, activation)?;
    rectangle_to_object(display_object.world_bounds(), activation)
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
    Ok(object.into())
}
