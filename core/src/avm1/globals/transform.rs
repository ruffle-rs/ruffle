//! flash.geom.Transform

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::color_transform::ColorTransformObject;
use crate::avm1::globals::matrix::{matrix_to_value, object_to_matrix};
use crate::avm1::object::NativeObject;
use crate::avm1::object_reference::MovieClipReference;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::string::StringContext;
use gc_arena::Collect;
use swf::{Rectangle, Twips};

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct TransformObject<'gc> {
    clip: Option<MovieClipReference<'gc>>,
}

impl<'gc> TransformObject<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Option<Self> {
        let clip = match args {
            // `Transform` constructor accepts exactly 1 argument.
            [Value::MovieClip(clip)] => Some(*clip),
            [Value::Object(clip)] => {
                let stage_object = clip.as_stage_object()?;
                MovieClipReference::try_from_stage_object(activation, stage_object)
            }
            _ => return None,
        };
        Some(Self { clip })
    }

    pub fn clip(&self, activation: &mut Activation<'_, 'gc>) -> Option<DisplayObject<'gc>> {
        let (_, _, clip) = self.clip?.resolve_reference(activation)?;
        Some(clip)
    }
}

macro_rules! transform_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "matrix" => property(transform_method!(101), transform_method!(102); VERSION_8);
    "concatenatedMatrix" => property(transform_method!(103), transform_method!(104); VERSION_8);
    "colorTransform" => property(transform_method!(105), transform_method!(106); VERSION_8);
    "concatenatedColorTransform" => property(transform_method!(107), transform_method!(108); VERSION_8);
    "pixelBounds" => property(transform_method!(109), transform_method!(110); VERSION_8);
};

fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u8,
) -> Result<Value<'gc>, Error<'gc>> {
    const CONSTRUCTOR: u8 = 0;
    const GET_MATRIX: u8 = 101;
    const SET_MATRIX: u8 = 102;
    const GET_CONCATENATED_MATRIX: u8 = 103;
    const GET_COLOR_TRANSFORM: u8 = 105;
    const SET_COLOR_TRANSFORM: u8 = 106;
    const GET_CONCATENATED_COLOR_TRANSFORM: u8 = 107;
    const GET_PIXEL_BOUNDS: u8 = 109;

    if index == CONSTRUCTOR {
        let Some(transform) = TransformObject::new(activation, args) else {
            return Ok(Value::Undefined);
        };
        this.set_native(
            activation.context.gc_context,
            NativeObject::Transform(transform),
        );
        return Ok(this.into());
    }

    let NativeObject::Transform(this) = this.native() else {
        return Ok(Value::Undefined);
    };
    let Some(clip) = this.clip(activation) else {
        return Ok(Value::Undefined);
    };

    Ok(match index {
        GET_MATRIX => matrix_to_value(clip.base().matrix(), activation)?,
        SET_MATRIX => {
            if let [value] = args {
                let object = value.coerce_to_object(activation);
                // Assignment only occurs for an object with Matrix properties (a, b, c, d, tx, ty).
                let is_matrix = ["a", "b", "c", "d", "tx", "ty"]
                    .iter()
                    .all(|p| object.has_own_property(activation, (*p).into()));
                if is_matrix {
                    let matrix = object_to_matrix(object, activation)?;
                    clip.set_matrix(activation.context.gc_context, matrix);
                    clip.set_transformed_by_script(activation.context.gc_context, true);
                    if let Some(parent) = clip.parent() {
                        // Self-transform changes are automatically handled,
                        // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                        parent.invalidate_cached_bitmap(activation.context.gc_context);
                    }
                }
            }
            Value::Undefined
        }
        GET_CONCATENATED_MATRIX => {
            // Testing shows that 'concatenatedMatrix' does *not* include the 'scrollRect' translation
            // for the object itself, but *does* include the 'scrollRect' translation for ancestors.
            matrix_to_value(
                &clip.local_to_global_matrix_without_own_scroll_rect(),
                activation,
            )?
        }
        GET_COLOR_TRANSFORM => {
            ColorTransformObject::construct(activation, clip.base().color_transform())?
        }
        SET_COLOR_TRANSFORM => {
            if let [value] = args {
                // Set only occurs for an object with actual ColorTransform data.
                if let Some(color_transform) = ColorTransformObject::cast(*value) {
                    clip.set_color_transform(
                        activation.context.gc_context,
                        color_transform.read().clone().into(),
                    );
                    clip.invalidate_cached_bitmap(activation.context.gc_context);
                    clip.set_transformed_by_script(activation.context.gc_context, true);
                }
            }
            Value::Undefined
        }
        GET_CONCATENATED_COLOR_TRANSFORM => {
            // Walk through parents to get combined color transform.
            let mut color_transform = *clip.base().color_transform();
            let mut node = clip.avm1_parent();
            while let Some(display_object) = node {
                color_transform = *display_object.base().color_transform() * color_transform;
                node = display_object.parent();
            }
            ColorTransformObject::construct(activation, &color_transform)?
        }
        GET_PIXEL_BOUNDS => {
            // This is equivalent to `clip.getBounds()`.
            let world_bounds = clip.world_bounds();

            // If the bounds are invalid, the pixelBounds rectangle consists only of zeroes.
            let bounds = if world_bounds == Rectangle::default() {
                Rectangle {
                    x_min: Twips::new(0),
                    x_max: Twips::new(0),
                    y_min: Twips::new(0),
                    y_max: Twips::new(0),
                }
            } else {
                world_bounds
            };

            // Return Rectangle object.
            let constructor = activation.context.avm1.prototypes().rectangle_constructor;
            constructor.construct(
                activation,
                &[
                    bounds.x_min.to_pixels().into(),
                    bounds.y_min.to_pixels().into(),
                    bounds.width().to_pixels().into(),
                    bounds.height().to_pixels().into(),
                ],
            )?
        }
        _ => Value::Undefined,
    })
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let transform_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, transform_proto, fn_proto);
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(transform_method!(0)),
        constructor_to_fn!(transform_method!(0)),
        fn_proto,
        transform_proto.into(),
    )
}
