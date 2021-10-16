//! flash.geom.Transform

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::globals::{color_transform, matrix};
use crate::avm1::object::transform_object::TransformObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::display_object::{DisplayObject, MovieClip, TDisplayObject};
use gc_arena::MutationContext;

macro_rules! tx_getter {
    ( $get:ident ) => {
        |activation, this, _args| {
            if let Some(transform) = this.as_transform_object() {
                if let Some(clip) = transform.clip() {
                    return $get(activation, clip);
                }
            }
            Ok(Value::Undefined)
        }
    };
}

macro_rules! tx_setter {
    ( $set:ident ) => {
        |activation, this, args| {
            if let Some(transform) = this.as_transform_object() {
                if let Some(clip) = transform.clip() {
                    let value = args.get(0).unwrap_or(&Value::Undefined).clone();
                    $set(activation, clip, value)?;
                }
            }
            Ok(Value::Undefined)
        }
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "concatenatedColorTransform" => property(tx_getter!(concatenated_color_transform));
    "concatenatedMatrix" => property(tx_getter!(concatenated_matrix));
    "colorTransform" => property(tx_getter!(color_transform), tx_setter!(set_color_transform));
    "matrix" => property(tx_getter!(matrix), tx_setter!(set_matrix));
    "pixelBounds" => property(tx_getter!(pixel_bounds));
};

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let clip = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)
        .as_display_object()
        .and_then(|o| o.as_movie_clip());

    if let (Some(transform), Some(clip)) = (this.as_transform_object(), clip) {
        transform.set_clip(activation.context.gc_context, clip);
    }

    Ok(this.into())
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let transform_object = TransformObject::empty(gc_context, Some(proto));
    let object = transform_object.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    transform_object.into()
}

fn concatenated_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    clip: MovieClip<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // Walk through parents to get combined color transform.
    let mut color_transform = *clip.base().color_transform();
    let mut node = clip.avm1_parent();
    while let Some(display_object) = node {
        color_transform = *display_object.base().color_transform() * color_transform;
        node = display_object.parent();
    }
    let color_transform = color_transform::color_transform_to_object(color_transform, activation)?;
    Ok(color_transform)
}

fn concatenated_matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    clip: MovieClip<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = matrix::matrix_to_object(clip.local_to_global_matrix(), activation)?;
    Ok(matrix)
}

fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    clip: MovieClip<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let color_transform =
        color_transform::color_transform_to_object(*clip.base().color_transform(), activation)?;
    Ok(color_transform)
}

fn set_color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    clip: MovieClip<'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let as_color_transform = value.coerce_to_object(activation);
    // Set only occurs for an object with actual ColorTransform data.
    if as_color_transform.as_color_transform_object().is_some() {
        let swf_color_transform =
            color_transform::object_to_color_transform(as_color_transform, activation)?;
        clip.set_color_transform(activation.context.gc_context, &swf_color_transform);
        clip.set_transformed_by_script(activation.context.gc_context, true);
    }

    Ok(())
}

fn matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    clip: MovieClip<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = matrix::matrix_to_object(*clip.base().matrix(), activation)?;
    Ok(matrix)
}

fn set_matrix<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    clip: MovieClip<'gc>,
    value: Value<'gc>,
) -> Result<(), Error<'gc>> {
    let as_matrix = value.coerce_to_object(activation);
    // Assignment only occurs for an object with Matrix properties (a, b, c, d, tx, ty).
    let is_matrix = ["a", "b", "c", "d", "tx", "ty"]
        .iter()
        .all(|p| as_matrix.has_own_property(activation, (*p).into()));
    if is_matrix {
        let swf_matrix = matrix::object_to_matrix(as_matrix, activation)?;
        clip.set_matrix(activation.context.gc_context, &swf_matrix);
        clip.set_transformed_by_script(activation.context.gc_context, true);
    }

    Ok(())
}

fn pixel_bounds<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    clip: MovieClip<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    // This is equivalent to `clip.getBounds()`.
    let bounds = clip.world_bounds();

    // Return Rectangle object.
    let constructor = activation.context.avm1.prototypes.rectangle_constructor;
    let result = constructor.construct(
        activation,
        &[
            bounds.x_min.to_pixels().into(),
            bounds.y_min.to_pixels().into(),
            bounds.width().to_pixels().into(),
            bounds.height().to_pixels().into(),
        ],
    )?;
    Ok(result)
}

pub fn apply_to_display_object<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    transform: Object<'gc>,
    display_object: DisplayObject<'gc>,
) -> Result<(), Error<'gc>> {
    if let Some(transform) = transform.as_transform_object() {
        if let Some(clip) = transform.clip() {
            let matrix = *clip.base().matrix();
            display_object.set_matrix(activation.context.gc_context, &matrix);
            let color_transform = *clip.base().color_transform();
            display_object.set_color_transform(activation.context.gc_context, &color_transform);
            display_object.set_transformed_by_script(activation.context.gc_context, true);
        }
    }

    Ok(())
}
