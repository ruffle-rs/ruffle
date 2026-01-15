//! flash.geom.Transform

use crate::avm1::globals::color_transform::ColorTransformObject;
use crate::avm1::globals::matrix::{matrix_to_value, object_to_matrix};
use crate::avm1::object::NativeObject;
use crate::avm1::object_reference::MovieClipReference;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Activation, Error, Object, Value};
use crate::display_object::{DisplayObject, TDisplayObject};
use crate::string::AvmString;
use gc_arena::Collect;
use ruffle_macros::istr;
use swf::{Rectangle, Twips};

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub struct TransformObject<'gc> {
    clip: Option<MovieClipReference<'gc>>,
}

impl<'gc> TransformObject<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Option<Self> {
        let clip = match args {
            // `Transform` constructor accepts exactly 1 argument.
            [Value::MovieClip(clip)] => Some(*clip),
            [Value::Object(clip)] => MovieClipReference::try_from_stage_object(activation, *clip),
            _ => return None,
        };
        Some(Self { clip })
    }

    pub fn clip(self, activation: &mut Activation<'_, 'gc>) -> Option<DisplayObject<'gc>> {
        let (_, _, clip) = self.clip?.resolve_reference(activation)?;
        Some(clip)
    }
}

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "matrix" => property(GET_MATRIX, SET_MATRIX; VERSION_8);
    "concatenatedMatrix" => property(GET_CONCATENATED_MATRIX; VERSION_8);
    "colorTransform" => property(GET_COLOR_TRANSFORM, SET_COLOR_TRANSFORM; VERSION_8);
    "concatenatedColorTransform" => property(GET_CONCATENATED_COLOR_TRANSFORM; VERSION_8);
    "pixelBounds" => property(GET_PIXEL_BOUNDS; VERSION_8);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(table_constructor!(method), None, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

pub mod method {
    pub const CONSTRUCTOR: u16 = 0;
    pub const GET_MATRIX: u16 = 101;
    pub const SET_MATRIX: u16 = 102;
    pub const GET_CONCATENATED_MATRIX: u16 = 103;
    pub const GET_COLOR_TRANSFORM: u16 = 105;
    pub const SET_COLOR_TRANSFORM: u16 = 106;
    pub const GET_CONCATENATED_COLOR_TRANSFORM: u16 = 107;
    pub const GET_PIXEL_BOUNDS: u16 = 109;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;

    if index == CONSTRUCTOR {
        let Some(transform) = TransformObject::new(activation, args) else {
            return Ok(Value::Undefined);
        };
        this.set_native(activation.gc(), NativeObject::Transform(transform));
        return Ok(this.into());
    }

    let NativeObject::Transform(this) = this.native() else {
        return Ok(Value::Undefined);
    };
    let Some(clip) = this.clip(activation) else {
        return Ok(Value::Undefined);
    };

    Ok(match index {
        GET_MATRIX => matrix_to_value(&clip.base().matrix(), activation)?,
        SET_MATRIX => {
            let matrix_props: &[AvmString<'_>] = &[
                istr!("a"),
                istr!("b"),
                istr!("c"),
                istr!("d"),
                istr!("tx"),
                istr!("ty"),
            ];

            if let [value] = args {
                let object = value.coerce_to_object_or_bare(activation)?;
                // Assignment only occurs for an object with Matrix properties (a, b, c, d, tx, ty).
                let is_matrix = matrix_props
                    .iter()
                    .all(|p| object.has_own_property(activation, *p));
                if is_matrix {
                    let matrix = object_to_matrix(object, activation)?;
                    clip.set_matrix(matrix);
                    clip.set_transformed_by_script(true);
                    if let Some(parent) = clip.parent() {
                        // Self-transform changes are automatically handled,
                        // we only want to inform ancestors to avoid unnecessary invalidations for tx/ty
                        parent.invalidate_cached_bitmap();
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
            ColorTransformObject::construct(activation, &clip.base().color_transform())?
        }
        SET_COLOR_TRANSFORM => {
            if let [value] = args {
                // Set only occurs for an object with actual ColorTransform data.
                if let Some(color_transform) = ColorTransformObject::cast(*value) {
                    clip.set_color_transform((*color_transform).clone().into());
                    clip.invalidate_cached_bitmap();
                    clip.set_transformed_by_script(true);
                }
            }
            Value::Undefined
        }
        GET_CONCATENATED_COLOR_TRANSFORM => {
            // Walk through parents to get combined color transform.
            let mut color_transform = clip.base().color_transform();
            let mut node = clip.avm1_parent();
            while let Some(display_object) = node {
                color_transform = display_object.base().color_transform() * color_transform;
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
                    x_min: Twips::ZERO,
                    x_max: Twips::ZERO,
                    y_min: Twips::ZERO,
                    y_max: Twips::ZERO,
                }
            } else {
                world_bounds
            };

            // Return Rectangle object.
            let constructor = activation.prototypes().rectangle_constructor;
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
