use crate::avm2::error::make_error_2008;
use crate::avm2::globals::flash::display::shader_job::get_shader_args;
use crate::avm2::globals::slots::flash_filters_bevel_filter as bevel_filter_slots;
use crate::avm2::globals::slots::flash_filters_blur_filter as blur_filter_slots;
use crate::avm2::globals::slots::flash_filters_color_matrix_filter as color_matrix_filter_slots;
use crate::avm2::globals::slots::flash_filters_convolution_filter as convolution_filter_slots;
use crate::avm2::globals::slots::flash_filters_displacement_map_filter as displacement_map_filter_slots;
use crate::avm2::globals::slots::flash_filters_drop_shadow_filter as drop_shadow_filter_slots;
use crate::avm2::globals::slots::flash_filters_glow_filter as glow_filter_slots;
use crate::avm2::globals::slots::flash_filters_gradient_bevel_filter as gradient_bevel_filter_slots;
use crate::avm2::globals::slots::flash_filters_gradient_glow_filter as gradient_glow_filter_slots;
use crate::avm2::globals::slots::flash_filters_shader_filter as shader_filter_slots;
use crate::avm2::globals::slots::flash_geom_point as point_slots;
use crate::avm2::object::{ArrayObject, ClassObject, Object, TObject};
use crate::avm2::{Activation, Error, Value};

use gc_arena::{Collect, DynamicRoot, Gc, Rootable};
use ruffle_macros::istr;
use ruffle_render::filters::{
    DisplacementMapFilter, DisplacementMapFilterMode, Filter, ShaderFilter, ShaderObject,
};
use std::any::Any;
use std::fmt::Debug;
use swf::{
    BevelFilter, BevelFilterFlags, BlurFilter, BlurFilterFlags, Color, ColorMatrixFilter,
    ConvolutionFilter, ConvolutionFilterFlags, DropShadowFilter, DropShadowFilterFlags, Fixed16,
    Fixed8, GlowFilter, GlowFilterFlags, GradientFilter, GradientFilterFlags, GradientRecord,
};

pub trait FilterAvm2Ext {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>>;

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>>;
}

#[derive(Clone, Collect)]
#[collect(require_static)]
pub struct ObjectWrapper {
    root: DynamicRoot<Rootable![Object<'_>]>,
}

impl ShaderObject for ObjectWrapper {
    fn clone_box(&self) -> Box<dyn ShaderObject> {
        Box::new(self.clone())
    }

    fn equals(&self, other: &dyn ShaderObject) -> bool {
        if let Some(other_wrapper) = <dyn Any>::downcast_ref::<ObjectWrapper>(other) {
            std::ptr::eq(self.root.as_ptr(), other_wrapper.root.as_ptr())
        } else {
            false
        }
    }
}

impl Debug for ObjectWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectWrapper")
            .field("root", &self.root.as_ptr())
            .finish()
    }
}

impl FilterAvm2Ext for Filter {
    fn from_avm2_object<'gc>(
        activation: &mut Activation<'_, 'gc>,
        object: Object<'gc>,
    ) -> Result<Filter, Error<'gc>> {
        let bevel_filter = activation
            .avm2()
            .classes()
            .bevelfilter
            .inner_class_definition();
        if object.is_of_type(bevel_filter) {
            return avm2_to_bevel_filter(activation, object);
        }

        let blur_filter = activation
            .avm2()
            .classes()
            .blurfilter
            .inner_class_definition();
        if object.is_of_type(blur_filter) {
            return avm2_to_blur_filter(activation, object);
        }

        let color_matrix_filter = activation
            .avm2()
            .classes()
            .colormatrixfilter
            .inner_class_definition();
        if object.is_of_type(color_matrix_filter) {
            return avm2_to_color_matrix_filter(activation, object);
        }

        let convolution_filter = activation
            .avm2()
            .classes()
            .convolutionfilter
            .inner_class_definition();
        if object.is_of_type(convolution_filter) {
            return avm2_to_convolution_filter(activation, object);
        }

        let displacement_map_filter = activation
            .avm2()
            .classes()
            .displacementmapfilter
            .inner_class_definition();
        if object.is_of_type(displacement_map_filter) {
            return avm2_to_displacement_map_filter(activation, object);
        }

        let drop_shadow_filter = activation
            .avm2()
            .classes()
            .dropshadowfilter
            .inner_class_definition();
        if object.is_of_type(drop_shadow_filter) {
            return avm2_to_drop_shadow_filter(activation, object);
        }

        let glow_filter = activation
            .avm2()
            .classes()
            .glowfilter
            .inner_class_definition();
        if object.is_of_type(glow_filter) {
            return avm2_to_glow_filter(activation, object);
        }

        let gradient_bevel_filter = activation
            .avm2()
            .classes()
            .gradientbevelfilter
            .inner_class_definition();
        if object.is_of_type(gradient_bevel_filter) {
            return Ok(Filter::GradientBevelFilter(avm2_to_gradient_filter(
                activation, object,
            )?));
        }

        let gradient_glow_filter = activation
            .avm2()
            .classes()
            .gradientglowfilter
            .inner_class_definition();
        if object.is_of_type(gradient_glow_filter) {
            return Ok(Filter::GradientGlowFilter(avm2_to_gradient_filter(
                activation, object,
            )?));
        }

        let shader_filter = activation
            .avm2()
            .classes()
            .shaderfilter
            .inner_class_definition();
        if object.is_of_type(shader_filter) {
            return Ok(Filter::ShaderFilter(avm2_to_shader_filter(
                activation, object,
            )?));
        }

        unreachable!("{object:?} must be of type BitmapFilter")
    }

    fn as_avm2_object<'gc>(
        &self,
        activation: &mut Activation<'_, 'gc>,
    ) -> Result<Value<'gc>, Error<'gc>> {
        match self {
            Filter::BevelFilter(filter) => bevel_filter_to_avm2(activation, filter),
            Filter::BlurFilter(filter) => blur_filter_to_avm2(activation, filter),
            Filter::ColorMatrixFilter(filter) => color_matrix_filter_to_avm2(activation, filter),
            Filter::ConvolutionFilter(filter) => convolution_filter_to_avm2(activation, filter),
            Filter::DisplacementMapFilter(filter) => {
                displacement_map_filter_to_avm2(activation, filter)
            }
            Filter::DropShadowFilter(filter) => drop_shadow_filter_to_avm2(activation, filter),
            Filter::GlowFilter(filter) => glow_filter_to_avm2(activation, filter),
            Filter::GradientBevelFilter(filter) => {
                let gradientbevelfilter = activation.avm2().classes().gradientbevelfilter;
                gradient_filter_to_avm2(activation, filter, gradientbevelfilter)
            }
            Filter::GradientGlowFilter(filter) => {
                let gradientglowfilter = activation.avm2().classes().gradientglowfilter;
                gradient_filter_to_avm2(activation, filter, gradientglowfilter)
            }
            Filter::ShaderFilter(filter) => shader_filter_to_avm2(activation, filter),
        }
    }
}

fn avm2_to_bevel_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Filter, Error<'gc>> {
    let angle = object
        .get_slot(bevel_filter_slots::ANGLE)
        .coerce_to_number(activation)?;
    let blur_x = object
        .get_slot(bevel_filter_slots::BLUR_X)
        .coerce_to_number(activation)?;
    let blur_y = object
        .get_slot(bevel_filter_slots::BLUR_Y)
        .coerce_to_number(activation)?;
    let distance = object
        .get_slot(bevel_filter_slots::DISTANCE)
        .coerce_to_number(activation)?;
    let highlight_alpha = object
        .get_slot(bevel_filter_slots::HIGHLIGHT_ALPHA)
        .coerce_to_number(activation)?;
    let highlight_color = object
        .get_slot(bevel_filter_slots::HIGHLIGHT_COLOR)
        .coerce_to_u32(activation)?;
    let knockout = object
        .get_slot(bevel_filter_slots::KNOCKOUT)
        .coerce_to_boolean();
    let quality = object
        .get_slot(bevel_filter_slots::QUALITY)
        .coerce_to_u32(activation)?;
    let shadow_alpha = object
        .get_slot(bevel_filter_slots::SHADOW_ALPHA)
        .coerce_to_number(activation)?;
    let shadow_color = object
        .get_slot(bevel_filter_slots::SHADOW_COLOR)
        .coerce_to_u32(activation)?;
    let strength = object
        .get_slot(bevel_filter_slots::STRENGTH)
        .coerce_to_number(activation)?;
    let bevel_type = object
        .get_slot(bevel_filter_slots::TYPE)
        .coerce_to_string(activation)?;
    let mut flags = BevelFilterFlags::COMPOSITE_SOURCE;
    if &bevel_type == b"inner" {
        flags |= BevelFilterFlags::INNER_SHADOW;
    } else if &bevel_type != b"outer" {
        flags |= BevelFilterFlags::ON_TOP;
    }
    flags.set(BevelFilterFlags::KNOCKOUT, knockout);
    flags |= BevelFilterFlags::from_passes(quality.clamp(0, 15) as u8);
    Ok(Filter::BevelFilter(BevelFilter {
        shadow_color: Color::from_rgb(shadow_color, (shadow_alpha * 255.0) as u8),
        highlight_color: Color::from_rgb(highlight_color, (highlight_alpha * 255.0) as u8),
        blur_x: Fixed16::from_f64(blur_x.max(0.0)),
        blur_y: Fixed16::from_f64(blur_y.max(0.0)),
        angle: Fixed16::from_f64(angle.to_radians()),
        distance: Fixed16::from_f64(distance),
        strength: Fixed8::from_f64(strength.clamp(0.0, 255.0)),
        flags,
    }))
}

fn bevel_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &BevelFilter,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.avm2().classes().bevelfilter.construct(
        activation,
        &[
            filter.distance.to_f64().into(),
            filter.angle.to_f64().to_degrees().into(),
            filter.highlight_color.to_rgb().into(),
            (f64::from(filter.highlight_color.a) / 255.0).into(),
            filter.shadow_color.to_rgb().into(),
            (f64::from(filter.shadow_color.a) / 255.0).into(),
            filter.blur_x.to_f64().into(),
            filter.blur_y.to_f64().into(),
            filter.strength.to_f64().into(),
            filter.num_passes().into(),
            if filter.is_on_top() {
                istr!("full")
            } else if filter.is_inner() {
                istr!("inner")
            } else {
                istr!("outer")
            }
            .into(),
            filter.is_knockout().into(),
        ],
    )
}

fn avm2_to_blur_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Filter, Error<'gc>> {
    let blur_x = object
        .get_slot(blur_filter_slots::BLUR_X)
        .coerce_to_number(activation)?;
    let blur_y = object
        .get_slot(blur_filter_slots::BLUR_Y)
        .coerce_to_number(activation)?;
    let quality = object
        .get_slot(blur_filter_slots::QUALITY)
        .coerce_to_u32(activation)?;
    Ok(Filter::BlurFilter(BlurFilter {
        blur_x: Fixed16::from_f64(blur_x.max(0.0)),
        blur_y: Fixed16::from_f64(blur_y.max(0.0)),
        flags: BlurFilterFlags::from_passes(quality.clamp(0, 15) as u8),
    }))
}

fn blur_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &BlurFilter,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.avm2().classes().blurfilter.construct(
        activation,
        &[
            filter.blur_x.to_f64().into(),
            filter.blur_y.to_f64().into(),
            filter.num_passes().into(),
        ],
    )
}

fn avm2_to_color_matrix_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Filter, Error<'gc>> {
    let mut matrix = [0.0; 20];
    if let Some(matrix_object) = object
        .get_slot(color_matrix_filter_slots::_MATRIX)
        .as_object()
    {
        if let Some(array) = matrix_object.as_array_storage() {
            for i in 0..matrix.len().min(array.length()) {
                matrix[i] = array
                    .get(i)
                    .expect("Length was already checked at this point")
                    .coerce_to_number(activation)? as f32;
            }
        }
    }
    Ok(Filter::ColorMatrixFilter(ColorMatrixFilter { matrix }))
}

fn color_matrix_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &ColorMatrixFilter,
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = ArrayObject::from_storage(
        activation,
        filter.matrix.iter().map(|v| Value::from(*v)).collect(),
    );
    activation
        .avm2()
        .classes()
        .colormatrixfilter
        .construct(activation, &[matrix.into()])
}

fn avm2_to_convolution_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Filter, Error<'gc>> {
    let mut matrix = vec![];
    if let Some(matrix_object) = object
        .get_slot(convolution_filter_slots::MATRIX)
        .as_object()
    {
        if let Some(array) = matrix_object.as_array_storage() {
            for value in array.iter() {
                matrix.push(
                    value
                        .unwrap_or(Value::Undefined)
                        .coerce_to_number(activation)? as f32,
                );
            }
        }
    }
    let alpha = object
        .get_slot(convolution_filter_slots::ALPHA)
        .coerce_to_number(activation)?;
    let bias = object
        .get_slot(convolution_filter_slots::BIAS)
        .coerce_to_number(activation)?;
    let clamp = object
        .get_slot(convolution_filter_slots::CLAMP)
        .coerce_to_boolean();
    let color = object
        .get_slot(convolution_filter_slots::COLOR)
        .coerce_to_u32(activation)?;
    let divisor = object
        .get_slot(convolution_filter_slots::DIVISOR)
        .coerce_to_number(activation)?;
    let matrix_x = object
        .get_slot(convolution_filter_slots::MATRIX_X)
        .coerce_to_u32(activation)?;
    let matrix_y = object
        .get_slot(convolution_filter_slots::MATRIX_Y)
        .coerce_to_u32(activation)?;
    let preserve_alpha = object
        .get_slot(convolution_filter_slots::PRESERVE_ALPHA)
        .coerce_to_boolean();
    let mut flags = ConvolutionFilterFlags::empty();
    flags.set(ConvolutionFilterFlags::CLAMP, clamp);
    if preserve_alpha {
        flags |= ConvolutionFilterFlags::PRESERVE_ALPHA;
    }
    matrix.resize((matrix_x * matrix_y) as usize, 0.0f32);
    Ok(Filter::ConvolutionFilter(ConvolutionFilter {
        bias: bias as f32,
        default_color: Color::from_rgb(color, (alpha * 255.0) as u8),
        divisor: divisor as f32,
        matrix,
        num_matrix_cols: matrix_x.clamp(0, 255) as u8,
        num_matrix_rows: matrix_y.clamp(0, 255) as u8,
        flags,
    }))
}

fn convolution_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &ConvolutionFilter,
) -> Result<Value<'gc>, Error<'gc>> {
    let matrix = ArrayObject::from_storage(
        activation,
        filter
            .matrix
            .iter()
            .map(|v| Value::from(f64::from(*v)))
            .collect(),
    );
    activation.avm2().classes().convolutionfilter.construct(
        activation,
        &[
            filter.num_matrix_cols.into(),
            filter.num_matrix_rows.into(),
            matrix.into(),
            filter.divisor.into(),
            filter.bias.into(),
            filter.is_preserve_alpha().into(),
            filter.is_clamped().into(),
            filter.default_color.to_rgb().into(),
            (f64::from(filter.default_color.a) / 255.0).into(),
        ],
    )
}

fn avm2_to_displacement_map_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Filter, Error<'gc>> {
    let alpha = object
        .get_slot(displacement_map_filter_slots::ALPHA)
        .coerce_to_number(activation)?;
    let color = object
        .get_slot(displacement_map_filter_slots::COLOR)
        .coerce_to_u32(activation)?;
    let component_x = object
        .get_slot(displacement_map_filter_slots::COMPONENT_X)
        .coerce_to_u32(activation)?;
    let component_y = object
        .get_slot(displacement_map_filter_slots::COMPONENT_Y)
        .coerce_to_u32(activation)?;
    let map_point =
        if let Value::Object(point) = object.get_slot(displacement_map_filter_slots::MAP_POINT) {
            (
                point.get_slot(point_slots::X).coerce_to_i32(activation)?,
                point.get_slot(point_slots::Y).coerce_to_i32(activation)?,
            )
        } else {
            (0, 0)
        };
    let mode = if let Value::String(mode) = object.get_slot(displacement_map_filter_slots::MODE) {
        if &mode == b"clamp" {
            DisplacementMapFilterMode::Clamp
        } else if &mode == b"ignore" {
            DisplacementMapFilterMode::Ignore
        } else if &mode == b"color" {
            DisplacementMapFilterMode::Color
        } else if &mode == b"wrap" {
            DisplacementMapFilterMode::Wrap
        } else {
            return Err(make_error_2008(activation, "mode"));
        }
    } else {
        DisplacementMapFilterMode::Wrap
    };
    let scale_x = object
        .get_slot(displacement_map_filter_slots::SCALE_X)
        .coerce_to_number(activation)?;
    let scale_y = object
        .get_slot(displacement_map_filter_slots::SCALE_Y)
        .coerce_to_number(activation)?;
    let map_bitmap =
        if let Value::Object(bitmap) = object.get_slot(displacement_map_filter_slots::MAP_BITMAP) {
            Some(
                bitmap
                    .as_bitmap_data()
                    .unwrap()
                    .bitmap_handle(activation.gc(), activation.context.renderer),
            )
        } else {
            None
        };
    Ok(Filter::DisplacementMapFilter(DisplacementMapFilter {
        color: Color::from_rgb(color, (alpha * 255.0) as u8),
        component_x: component_x as u8,
        component_y: component_y as u8,
        map_bitmap,
        map_point,
        mode,
        scale_x: scale_x as f32,
        scale_y: scale_y as f32,
        viewscale_x: 1.0,
        viewscale_y: 1.0,
    }))
}

fn displacement_map_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &DisplacementMapFilter,
) -> Result<Value<'gc>, Error<'gc>> {
    let point = activation.avm2().classes().point;
    let map_point = point.construct(
        activation,
        &[filter.map_point.0.into(), filter.map_point.1.into()],
    )?;
    let mode = match filter.mode {
        DisplacementMapFilterMode::Clamp => istr!("clamp"),
        DisplacementMapFilterMode::Color => istr!("color"),
        DisplacementMapFilterMode::Ignore => istr!("ignore"),
        DisplacementMapFilterMode::Wrap => istr!("wrap"),
    };
    activation.avm2().classes().displacementmapfilter.construct(
        activation,
        &[
            Value::Null, // TODO: This should be a BitmapData...
            map_point,
            filter.component_x.into(),
            filter.component_y.into(),
            filter.scale_x.into(),
            filter.scale_y.into(),
            mode.into(),
            filter.color.to_rgb().into(),
            (f64::from(filter.color.a) / 255.0).into(),
        ],
    )
}

fn avm2_to_drop_shadow_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Filter, Error<'gc>> {
    let alpha = object
        .get_slot(drop_shadow_filter_slots::ALPHA)
        .coerce_to_number(activation)?;
    let angle = object
        .get_slot(drop_shadow_filter_slots::ANGLE)
        .coerce_to_number(activation)?;
    let blur_x = object
        .get_slot(drop_shadow_filter_slots::BLUR_X)
        .coerce_to_number(activation)?;
    let blur_y = object
        .get_slot(drop_shadow_filter_slots::BLUR_Y)
        .coerce_to_number(activation)?;
    let color = object
        .get_slot(drop_shadow_filter_slots::COLOR)
        .coerce_to_u32(activation)?;
    let distance = object
        .get_slot(drop_shadow_filter_slots::DISTANCE)
        .coerce_to_number(activation)?;
    let hide_object = object
        .get_slot(drop_shadow_filter_slots::HIDE_OBJECT)
        .coerce_to_boolean();
    let inner = object
        .get_slot(drop_shadow_filter_slots::INNER)
        .coerce_to_boolean();
    let knockout = object
        .get_slot(drop_shadow_filter_slots::KNOCKOUT)
        .coerce_to_boolean();
    let quality = object
        .get_slot(drop_shadow_filter_slots::QUALITY)
        .coerce_to_u32(activation)?;
    let strength = object
        .get_slot(drop_shadow_filter_slots::STRENGTH)
        .coerce_to_number(activation)?;
    let mut flags = DropShadowFilterFlags::empty();
    if !hide_object {
        flags |= DropShadowFilterFlags::COMPOSITE_SOURCE;
    }
    flags.set(DropShadowFilterFlags::INNER_SHADOW, inner);
    flags.set(DropShadowFilterFlags::KNOCKOUT, knockout);
    flags |= DropShadowFilterFlags::from_passes(quality.clamp(0, 15) as u8);
    Ok(Filter::DropShadowFilter(DropShadowFilter {
        color: Color::from_rgb(color, (alpha * 255.0) as u8),
        angle: Fixed16::from_f64(angle.to_radians()),
        blur_x: Fixed16::from_f64(blur_x.max(0.0)),
        blur_y: Fixed16::from_f64(blur_y.max(0.0)),
        distance: Fixed16::from_f64(distance),
        strength: Fixed8::from_f64(strength.clamp(0.0, 255.0)),
        flags,
    }))
}

fn drop_shadow_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &DropShadowFilter,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.avm2().classes().dropshadowfilter.construct(
        activation,
        &[
            filter.distance.to_f64().into(),
            filter.angle.to_f64().to_degrees().into(),
            filter.color.to_rgb().into(),
            (f64::from(filter.color.a) / 255.0).into(),
            filter.blur_x.to_f64().into(),
            filter.blur_y.to_f64().into(),
            filter.strength.to_f64().into(),
            filter.num_passes().into(),
            filter.is_inner().into(),
            filter.is_knockout().into(),
            filter.hide_object().into(),
        ],
    )
}

fn avm2_to_glow_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Filter, Error<'gc>> {
    let alpha = object
        .get_slot(glow_filter_slots::ALPHA)
        .coerce_to_number(activation)?;
    let blur_x = object
        .get_slot(glow_filter_slots::BLUR_X)
        .coerce_to_number(activation)?;
    let blur_y = object
        .get_slot(glow_filter_slots::BLUR_Y)
        .coerce_to_number(activation)?;
    let color = object
        .get_slot(glow_filter_slots::COLOR)
        .coerce_to_u32(activation)?;
    let inner = object
        .get_slot(glow_filter_slots::INNER)
        .coerce_to_boolean();
    let knockout = object
        .get_slot(glow_filter_slots::KNOCKOUT)
        .coerce_to_boolean();
    let quality = object
        .get_slot(glow_filter_slots::QUALITY)
        .coerce_to_u32(activation)?;
    let strength = object
        .get_slot(glow_filter_slots::STRENGTH)
        .coerce_to_number(activation)?;
    let mut flags = GlowFilterFlags::COMPOSITE_SOURCE;
    flags.set(GlowFilterFlags::INNER_GLOW, inner);
    flags.set(GlowFilterFlags::KNOCKOUT, knockout);
    flags |= GlowFilterFlags::from_passes(quality.clamp(0, 15) as u8);
    Ok(Filter::GlowFilter(GlowFilter {
        color: Color::from_rgb(color, (alpha * 255.0) as u8),
        blur_x: Fixed16::from_f64(blur_x.max(0.0)),
        blur_y: Fixed16::from_f64(blur_y.max(0.0)),
        strength: Fixed8::from_f64(strength.clamp(0.0, 255.0)),
        flags,
    }))
}

fn glow_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &GlowFilter,
) -> Result<Value<'gc>, Error<'gc>> {
    activation.avm2().classes().glowfilter.construct(
        activation,
        &[
            filter.color.to_rgb().into(),
            (f64::from(filter.color.a) / 255.0).into(),
            filter.blur_x.to_f64().into(),
            filter.blur_y.to_f64().into(),
            filter.strength.to_f64().into(),
            filter.num_passes().into(),
            filter.is_inner().into(),
            filter.is_knockout().into(),
        ],
    )
}

fn avm2_to_gradient_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<GradientFilter, Error<'gc>> {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(gradient_bevel_filter_slots::_ANGLE == gradient_glow_filter_slots::_ANGLE);
        assert!(gradient_bevel_filter_slots::_BLUR_X == gradient_glow_filter_slots::_BLUR_X);
        assert!(gradient_bevel_filter_slots::_BLUR_Y == gradient_glow_filter_slots::_BLUR_Y);
        assert!(gradient_bevel_filter_slots::_DISTANCE == gradient_glow_filter_slots::_DISTANCE);
        assert!(gradient_bevel_filter_slots::_KNOCKOUT == gradient_glow_filter_slots::_KNOCKOUT);
        assert!(gradient_bevel_filter_slots::_QUALITY == gradient_glow_filter_slots::_QUALITY);
        assert!(gradient_bevel_filter_slots::_STRENGTH == gradient_glow_filter_slots::_STRENGTH);
        assert!(gradient_bevel_filter_slots::_TYPE == gradient_glow_filter_slots::_TYPE);

        assert!(gradient_bevel_filter_slots::_COLORS == gradient_glow_filter_slots::_COLORS);
        assert!(gradient_bevel_filter_slots::_ALPHAS == gradient_glow_filter_slots::_ALPHAS);
        assert!(gradient_bevel_filter_slots::_RATIOS == gradient_glow_filter_slots::_RATIOS);
    }

    let angle = object
        .get_slot(gradient_bevel_filter_slots::_ANGLE)
        .coerce_to_number(activation)?;
    let blur_x = object
        .get_slot(gradient_bevel_filter_slots::_BLUR_X)
        .coerce_to_number(activation)?;
    let blur_y = object
        .get_slot(gradient_bevel_filter_slots::_BLUR_Y)
        .coerce_to_number(activation)?;
    let distance = object
        .get_slot(gradient_bevel_filter_slots::_DISTANCE)
        .coerce_to_number(activation)?;
    let knockout = object
        .get_slot(gradient_bevel_filter_slots::_KNOCKOUT)
        .coerce_to_boolean();
    let quality = object
        .get_slot(gradient_bevel_filter_slots::_QUALITY)
        .coerce_to_u32(activation)?;
    let strength = object
        .get_slot(gradient_bevel_filter_slots::_STRENGTH)
        .coerce_to_number(activation)?;
    let bevel_type = object
        .get_slot(gradient_bevel_filter_slots::_TYPE)
        .coerce_to_string(activation)?;
    let colors = get_gradient_colors(activation, object)?;
    let mut flags = GradientFilterFlags::COMPOSITE_SOURCE;
    flags.set(GradientFilterFlags::KNOCKOUT, knockout);
    if &bevel_type == b"inner" {
        flags |= GradientFilterFlags::INNER_SHADOW;
    } else if &bevel_type != b"outer" {
        flags |= GradientFilterFlags::ON_TOP;
    }
    flags |= GradientFilterFlags::from_passes(quality.clamp(0, 15) as u8);
    Ok(GradientFilter {
        colors,
        blur_x: Fixed16::from_f64(blur_x.max(0.0)),
        blur_y: Fixed16::from_f64(blur_y.max(0.0)),
        angle: Fixed16::from_f64(angle.to_radians()),
        distance: Fixed16::from_f64(distance),
        strength: Fixed8::from_f64(strength.clamp(0.0, 255.0)),
        flags,
    })
}

fn gradient_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &GradientFilter,
    class: ClassObject<'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let colors = ArrayObject::from_storage(
        activation,
        filter
            .colors
            .iter()
            .map(|v| Value::from(v.color.to_rgb()))
            .collect(),
    );
    let alphas = ArrayObject::from_storage(
        activation,
        filter
            .colors
            .iter()
            .map(|v| Value::from(f64::from(v.color.a) / 255.0))
            .collect(),
    );
    let ratios = ArrayObject::from_storage(
        activation,
        filter.colors.iter().map(|v| Value::from(v.ratio)).collect(),
    );
    class.construct(
        activation,
        &[
            filter.distance.to_f64().into(),
            filter.angle.to_f64().to_degrees().into(),
            colors.into(),
            alphas.into(),
            ratios.into(),
            filter.blur_x.to_f64().into(),
            filter.blur_y.to_f64().into(),
            filter.strength.to_f64().into(),
            filter.num_passes().into(),
            if filter.is_on_top() {
                istr!("full")
            } else if filter.is_inner() {
                istr!("inner")
            } else {
                istr!("outer")
            }
            .into(),
            filter.is_knockout().into(),
        ],
    )
}

fn avm2_to_shader_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<ShaderFilter<'static>, Error<'gc>> {
    let bottom_extension = object
        .get_slot(shader_filter_slots::_BOTTOM_EXTENSION)
        .coerce_to_i32(activation)?;

    let left_extension = object
        .get_slot(shader_filter_slots::_LEFT_EXTENSION)
        .coerce_to_i32(activation)?;

    let right_extension = object
        .get_slot(shader_filter_slots::_RIGHT_EXTENSION)
        .coerce_to_i32(activation)?;

    let top_extension = object
        .get_slot(shader_filter_slots::_TOP_EXTENSION)
        .coerce_to_i32(activation)?;

    let shader_obj = object
        .get_slot(shader_filter_slots::_SHADER)
        .as_object()
        .unwrap();

    let dyn_root = activation
        .context
        .dynamic_root
        .stash(activation.gc(), Gc::new(activation.gc(), shader_obj));

    let (shader_handle, shader_args) = get_shader_args(shader_obj, activation)?;

    Ok(ShaderFilter {
        shader_object: Box::new(ObjectWrapper { root: dyn_root }),
        shader: shader_handle,
        shader_args,
        bottom_extension,
        left_extension,
        right_extension,
        top_extension,
    })
}

fn shader_filter_to_avm2<'gc>(
    activation: &mut Activation<'_, 'gc>,
    filter: &ShaderFilter<'static>,
) -> Result<Value<'gc>, Error<'gc>> {
    let object_wrapper: &ObjectWrapper = <dyn Any>::downcast_ref(filter.shader_object.as_ref())
        .expect("ShaderObject was not an ObjectWrapper");

    let obj = *activation.context.dynamic_root.fetch(&object_wrapper.root);
    activation
        .avm2()
        .classes()
        .shaderfilter
        .construct(activation, &[obj.into()])
}

fn get_gradient_colors<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Vec<GradientRecord>, Error<'gc>> {
    let mut colors = vec![];
    if let Some(colors_object) = object
        .get_slot(gradient_bevel_filter_slots::_COLORS)
        .as_object()
    {
        if let Some(colors_array) = colors_object.as_array_storage() {
            if let Some(alphas_object) = object
                .get_slot(gradient_bevel_filter_slots::_ALPHAS)
                .as_object()
            {
                if let Some(alphas_array) = alphas_object.as_array_storage() {
                    if let Some(ratios_object) = object
                        .get_slot(gradient_bevel_filter_slots::_RATIOS)
                        .as_object()
                    {
                        if let Some(ratios_array) = ratios_object.as_array_storage() {
                            // Flash only keeps the elements from any array until the lowest index in each array
                            for i in 0..ratios_array
                                .length()
                                .min(alphas_array.length())
                                .min(colors_array.length())
                            {
                                let color = colors_array
                                    .get(i)
                                    .expect("Length was already checked at this point")
                                    .coerce_to_u32(activation)?;
                                let alpha = colors_array
                                    .get(i)
                                    .expect("Length was already checked at this point")
                                    .coerce_to_number(activation)?
                                    as f32;
                                let ratio = colors_array
                                    .get(i)
                                    .expect("Length was already checked at this point")
                                    .coerce_to_u32(activation)?;
                                colors.push(GradientRecord {
                                    ratio: ratio.clamp(0, 255) as u8,
                                    color: Color::from_rgb(color, (alpha * 255.0) as u8),
                                })
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(colors)
}
