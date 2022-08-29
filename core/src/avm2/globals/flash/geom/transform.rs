#![allow(non_snake_case)]

use crate::avm2::{Activation, Error, Namespace, Object, QName, TObject, Value};
use crate::display_object::{StageQuality, TDisplayObject};
use crate::prelude::{ColorTransform, DisplayObject, Matrix, Twips};
use swf::Fixed8;

fn get_display_object<'gc>(
    this: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<DisplayObject<'gc>, Error> {
    Ok(this
        .get_property(
            &QName::new(Namespace::Private("".into()), "_displayObject").into(),
            activation,
        )?
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap())
}

pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    this.unwrap().set_property(
        &QName::new(Namespace::Private("".into()), "_displayObject").into(),
        args[0],
        activation,
    )?;
    Ok(Value::Undefined)
}

pub fn get_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = this.unwrap();
    let ct_obj = *get_display_object(this, activation)?
        .base()
        .color_transform();
    color_transform_to_object(&ct_obj, activation)
}

pub fn set_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = this.unwrap();
    let ct = object_to_color_transform(args[0].coerce_to_object(activation)?, activation)?;
    get_display_object(this, activation)?
        .base_mut(activation.context.gc_context)
        .set_color_transform(&ct);
    Ok(Value::Undefined)
}

pub fn get_matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = this.unwrap();
    let matrix = *get_display_object(this, activation)?.base().matrix();
    matrix_to_object(matrix, activation)
}

pub fn set_matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = this.unwrap();
    let matrix = object_to_matrix(args[0].coerce_to_object(activation)?, activation)?;
    get_display_object(this, activation)?
        .base_mut(activation.context.gc_context)
        .set_matrix(&matrix);
    Ok(Value::Undefined)
}

pub fn get_concatenated_matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    let this = this.unwrap();

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
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error> {
    log::warn!("Transform.concatenatedColorTransform: not yet implemented");
    Ok(Value::Undefined)
}

// FIXME - handle clamping. We're throwing away precision here in converting to an integer:
// is that what we should be doing?
pub fn object_to_color_transform<'gc>(
    object: Object<'gc>,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<ColorTransform, Error> {
    let red_multiplier = object
        .get_property(&QName::dynamic_name("redMultiplier").into(), activation)?
        .coerce_to_number(activation)?;
    let green_multiplier = object
        .get_property(&QName::dynamic_name("greenMultiplier").into(), activation)?
        .coerce_to_number(activation)?;
    let blue_multiplier = object
        .get_property(&QName::dynamic_name("blueMultiplier").into(), activation)?
        .coerce_to_number(activation)?;
    let alpha_multiplier = object
        .get_property(&QName::dynamic_name("alphaMultiplier").into(), activation)?
        .coerce_to_number(activation)?;
    let red_offset = object
        .get_property(&QName::dynamic_name("redOffset").into(), activation)?
        .coerce_to_number(activation)?;
    let green_offset = object
        .get_property(&QName::dynamic_name("greenOffset").into(), activation)?
        .coerce_to_number(activation)?;
    let blue_offset = object
        .get_property(&QName::dynamic_name("blueOffset").into(), activation)?
        .coerce_to_number(activation)?;
    let alpha_offset = object
        .get_property(&QName::dynamic_name("alphaOffset").into(), activation)?
        .coerce_to_number(activation)?;
    Ok(ColorTransform {
        r_mult: Fixed8::from_f64(red_multiplier),
        g_mult: Fixed8::from_f64(green_multiplier),
        b_mult: Fixed8::from_f64(blue_multiplier),
        a_mult: Fixed8::from_f64(alpha_multiplier),
        r_add: red_offset as i16,
        g_add: green_offset as i16,
        b_add: blue_offset as i16,
        a_add: alpha_offset as i16,
    })
}

pub fn color_transform_to_object<'gc>(
    color_transform: &ColorTransform,
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error> {
    let args = [
        color_transform.r_mult.to_f64().into(),
        color_transform.g_mult.to_f64().into(),
        color_transform.b_mult.to_f64().into(),
        color_transform.a_mult.to_f64().into(),
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
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Value<'gc>, Error> {
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
    activation: &mut Activation<'_, 'gc, '_>,
) -> Result<Matrix, Error> {
    let a = object
        .get_property(&QName::dynamic_name("a").into(), activation)?
        .coerce_to_number(activation)? as f32;
    let b = object
        .get_property(&QName::dynamic_name("b").into(), activation)?
        .coerce_to_number(activation)? as f32;
    let c = object
        .get_property(&QName::dynamic_name("c").into(), activation)?
        .coerce_to_number(activation)? as f32;
    let d = object
        .get_property(&QName::dynamic_name("d").into(), activation)?
        .coerce_to_number(activation)? as f32;
    let tx = Twips::from_pixels(
        object
            .get_property(&QName::dynamic_name("tx").into(), activation)?
            .coerce_to_number(activation)?,
    );
    let ty = Twips::from_pixels(
        object
            .get_property(&QName::dynamic_name("ty").into(), activation)?
            .coerce_to_number(activation)?,
    );

    Ok(Matrix { a, b, c, d, tx, ty })
}
