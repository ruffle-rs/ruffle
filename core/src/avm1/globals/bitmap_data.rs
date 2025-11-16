//! flash.display.BitmapData object

use super::matrix::object_to_matrix;
use crate::avm1::globals::bitmap_filter;
use crate::avm1::globals::color_transform::ColorTransformObject;
use crate::avm1::object::NativeObject;
use crate::avm1::parameters::{ParametersExt, UndefinedAs};
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{Activation, Attribute, Error, Object, Value};
use crate::bitmap::bitmap_data::BitmapData;
use crate::bitmap::bitmap_data::{BitmapDataDrawError, IBitmapDrawable};
use crate::bitmap::bitmap_data::{ChannelOptions, ThresholdOperation};
use crate::bitmap::{is_size_valid, operations};
use crate::character::Character;
use crate::display_object::DisplayObject;
use crate::swf::BlendMode;
use crate::{avm1_stub, avm_error};
use ruffle_macros::istr;
use ruffle_render::transform::Transform;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "height" => property(height);
    "width" => property(width);
    "transparent" => property(get_transparent);
    "rectangle" => property(get_rectangle);
    "getPixel" => method(get_pixel);
    "getPixel32" => method(get_pixel32);
    "setPixel" => method(set_pixel);
    "setPixel32" => method(set_pixel32);
    "copyChannel" => method(copy_channel);
    "fillRect" => method(fill_rect);
    "clone" => method(clone);
    "dispose" => method(dispose);
    "floodFill" => method(flood_fill);
    "noise" => method(noise);
    "colorTransform" => method(color_transform);
    "getColorBoundsRect" => method(get_color_bounds_rect);
    "perlinNoise" => method(perlin_noise);
    "applyFilter" => method(apply_filter);
    "draw" => method(draw);
    "hitTest" => method(hit_test);
    "generateFilterRect" => method(generate_filter_rect);
    "copyPixels" => method(copy_pixels);
    "merge" => method(merge);
    "paletteMap" => method(palette_map);
    "pixelDissolve" => method(pixel_dissolve);
    "scroll" => method(scroll);
    "threshold" => method(threshold);
    "compare" => method(compare);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "loadBitmap" => method(load_bitmap);
    "RED_CHANNEL" => int(1);
    "GREEN_CHANNEL" => int(2);
    "BLUE_CHANNEL" => int(4);
    "ALPHA_CHANNEL" => int(8);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(constructor, None, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    context.define_properties_on(class.constr, OBJECT_DECLS);
    class
}

fn new_bitmap_data<'gc>(
    proto: Option<Value<'gc>>,
    bitmap_data: BitmapData<'gc>,
    activation: &mut Activation<'_, 'gc>,
) -> Object<'gc> {
    let gc_context = activation.gc();

    let object = Object::new_without_proto(gc_context);
    // Set `__proto__` manually since `Object::new()` doesn't support primitive prototypes.
    // TODO: Pass `proto` to `Object::new()` once possible.
    if let Some(proto) = proto {
        object.define_value(
            gc_context,
            istr!("__proto__"),
            proto,
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );
    }
    object.set_native(gc_context, NativeObject::BitmapData(bitmap_data));
    object
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok(Value::Undefined);
    }
    let width = args.get_u32(activation, 0)?;
    let height = args.get_u32(activation, 1)?;
    let transparency = args
        .try_get_bool(activation, 2, UndefinedAs::Some)
        .unwrap_or(true);
    let fill_color = args
        .try_get_u32(activation, 3, UndefinedAs::Some)?
        .unwrap_or(u32::MAX);

    if !is_size_valid(activation.swf_version(), width, height) {
        tracing::warn!("Invalid BitmapData size: {}x{}", width, height);
        return Ok(Value::Undefined);
    }

    let bitmap_data = BitmapData::new(
        activation.context.gc_context,
        width,
        height,
        transparency,
        fill_color,
    );
    this.set_native(
        activation.context.gc_context,
        NativeObject::BitmapData(bitmap_data),
    );
    Ok(this.into())
}

#[derive(Debug, Copy, Clone)]
enum BitmapDataResult<'gc> {
    Valid(BitmapData<'gc>),
    Disposed,
    NotBitmapData(Object<'gc>),
}

fn get_bitmap_data(object: Object) -> BitmapDataResult {
    if let NativeObject::BitmapData(bitmap_data) = object.native() {
        if bitmap_data.disposed() {
            BitmapDataResult::Disposed
        } else {
            BitmapDataResult::Valid(bitmap_data)
        }
    } else {
        BitmapDataResult::NotBitmapData(object)
    }
}

fn try_get_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    object: Object<'gc>,
) -> Result<Option<(i32, i32, i32, i32)>, Error<'gc>> {
    if !object.has_property(activation, istr!("x"))
        || !object.has_property(activation, istr!("y"))
        || !object.has_property(activation, istr!("width"))
        || !object.has_property(activation, istr!("height"))
    {
        return Ok(None);
    }
    let x = object
        .get(istr!("x"), activation)?
        .coerce_to_i32(activation)?;
    let y = object
        .get(istr!("y"), activation)?
        .coerce_to_i32(activation)?;
    let width = object
        .get(istr!("width"), activation)?
        .coerce_to_i32(activation)?;
    let height = object
        .get(istr!("height"), activation)?
        .coerce_to_i32(activation)?;
    Ok(Some((x, y, width, height)))
}

fn height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    Ok(bitmap_data.height().into())
}

fn width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    Ok(bitmap_data.width().into())
}

fn get_transparent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    Ok(bitmap_data.transparency().into())
}

fn get_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let proto = activation.prototypes().rectangle_constructor;
    let rect = proto.construct(
        activation,
        &[
            0.into(),
            0.into(),
            bitmap_data.width().into(),
            bitmap_data.height().into(),
        ],
    )?;

    Ok(rect)
}

fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let x = args.get_u32(activation, 0)?;
    let y = args.get_u32(activation, 1)?;
    // AVM1 returns a signed int, so we need to convert it.
    let col = operations::get_pixel(bitmap_data, activation.context.renderer, x, y) as i32;
    Ok(col.into())
}

fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let x = args.get_u32(activation, 0)?;
    let y = args.get_u32(activation, 1)?;
    // AVM1 returns a signed int, so we need to convert it.
    let col = operations::get_pixel32(bitmap_data, activation.context.renderer, x, y) as i32;
    Ok(col.into())
}

fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 3 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let x = args.get_u32(activation, 0)?;
    let y = args.get_u32(activation, 1)?;
    let color = args.get_u32(activation, 2)?;
    operations::set_pixel(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        x,
        y,
        color.into(),
    );
    Ok(0.into())
}

fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 3 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let x = args.get_u32(activation, 0)?;
    let y = args.get_u32(activation, 1)?;
    let color = args.get_u32(activation, 2)?;
    operations::set_pixel32(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        x,
        y,
        color,
    );
    Ok(0.into())
}

fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 5 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let source_bitmap = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => s,
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(_) => return Ok((-2).into()),
    };

    let source_rect = args.get_object(activation, 1);
    let Some((src_min_x, src_min_y, src_width, src_height)) =
        try_get_rect(activation, source_rect)?
    else {
        return Ok((-4).into());
    };

    let dest_point = args.get_object(activation, 2);
    let source_channel = args.get_i32(activation, 3)?;
    let dest_channel = args.get_i32(activation, 4)?;

    let min_x = dest_point
        .get(istr!("x"), activation)?
        .coerce_to_i32(activation)?;
    let min_y = dest_point
        .get(istr!("y"), activation)?
        .coerce_to_i32(activation)?;

    operations::copy_channel(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        (min_x, min_y),
        (src_min_x, src_min_y, src_width, src_height),
        source_bitmap,
        source_channel,
        dest_channel,
    );

    Ok((-1).into())
}

fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let source_rect = args.get_object(activation, 0);
    let Some((x, y, width, height)) = try_get_rect(activation, source_rect)? else {
        return Ok((-1).into());
    };
    let color = args.get_u32(activation, 1)?;

    operations::fill_rect(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        x,
        y,
        width,
        height,
        color,
    );
    Ok(0.into())
}

fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    Ok(new_bitmap_data(
        this.get_local_stored(istr!("__proto__"), activation),
        bitmap_data.clone_data(activation.context.gc_context, activation.context.renderer),
        activation,
    )
    .into())
}

fn dispose<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    bitmap_data.dispose(activation.gc());
    Ok(Value::Undefined)
}

fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 3 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };
    let x = args.get_u32(activation, 0)?;
    let y = args.get_u32(activation, 1)?;
    let color = args.get_u32(activation, 2)?;

    if operations::flood_fill(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        x,
        y,
        color,
    ) {
        Ok(1.into())
    } else {
        Ok(0.into())
    }
}

fn noise<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.is_empty() {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let random_seed = args.get_i32(activation, 0)?;
    // Normal u8 coercion would see negative numbers as 255, but here it allows and clamps them
    let low = args.get_i32(activation, 1)?.clamp(0, 255) as u8;
    let high = args
        .try_get_i32(activation, 2, UndefinedAs::Some)?
        .unwrap_or(0xFF)
        .clamp(0, 255) as u8;

    let channel_options = if let Some(c) = args.try_get_u8(activation, 3, UndefinedAs::Some)? {
        ChannelOptions::from_bits_truncate(c)
    } else {
        ChannelOptions::RGB
    };

    let gray_scale = args.get_bool(activation, 4);

    operations::noise(
        activation.gc(),
        bitmap_data,
        random_seed,
        low,
        high.max(low),
        channel_options,
        gray_scale,
    );

    Ok(0.into())
}

fn draw<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let matrix = args
        .get(1)
        .map(|o| o.coerce_to_object(activation))
        .and_then(|o| object_to_matrix(o, activation).ok())
        .unwrap_or_default();

    let color_transform = args
        .get(2)
        .and_then(|v| ColorTransformObject::cast(*v))
        .map(|color_transform| (*color_transform).clone().into())
        .unwrap_or_default();

    let mut blend_mode = BlendMode::Normal;
    if let Some(value) = args.get(3) {
        if let Some(mode) = value.as_blend_mode() {
            blend_mode = mode;
        } else {
            tracing::error!("Unknown blend mode {value:?}");
        }
    }

    if args.get(4).is_some() {
        avm1_stub!(activation, "BitmapData", "draw", "with clip rect");
    }
    let smoothing = args
        .get(5)
        .unwrap_or(&false.into())
        .as_bool(activation.swf_version());

    let source = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => IBitmapDrawable::BitmapData(s),
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(source) => {
            if let Some(source_object) = source.as_display_object() {
                IBitmapDrawable::DisplayObject(source_object)
            } else {
                return Ok((-2).into());
            }
        }
    };

    // Do this last, so that we only call `overwrite_cpu_pixels_from_gpu`
    // if we're actually going to draw something.
    let quality = activation.context.stage.quality();
    match operations::draw(
        activation.context,
        bitmap_data,
        source,
        Transform {
            matrix,
            color_transform,
            perspective_projection: None,
        },
        smoothing,
        blend_mode,
        None,
        quality,
    ) {
        Ok(()) => {}
        Err(BitmapDataDrawError::Unimplemented) => {
            avm_error!(
                activation,
                "Render backend does not support BitmapData.draw"
            );
        }
    }

    Ok(Value::Undefined)
}

fn apply_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let source = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => s,
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(_) => return Ok((-2).into()),
    };

    let source_rect = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let src_min_x = source_rect
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)? as u32;
    let src_min_y = source_rect
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)? as u32;
    let src_width = source_rect
        .get(istr!("width"), activation)?
        .coerce_to_f64(activation)? as u32;
    let src_height = source_rect
        .get(istr!("height"), activation)?
        .coerce_to_f64(activation)? as u32;

    let dest_point = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let dest_x = dest_point
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)? as u32;
    let dest_y = dest_point
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)? as u32;

    let filter_object = args
        .get(3)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);
    let filter = bitmap_filter::avm1_to_filter(filter_object, activation.context);

    if let Some(filter) = filter {
        operations::apply_filter(
            activation.context,
            bitmap_data,
            source,
            (src_min_x, src_min_y),
            (src_width, src_height),
            (dest_x, dest_y),
            filter,
        );
        return Ok(0.into());
    }

    Ok((-1).into())
}

fn generate_filter_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let BitmapDataResult::Valid(_bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    avm1_stub!(activation, "BitmapData", "generateFilterRect");
    Ok(Value::Undefined)
}

fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let rectangle = args.get_object(activation, 0);
    let Some((x, y, width, height)) = try_get_rect(activation, rectangle)? else {
        return Ok((-2).into());
    };
    let color_transform = args.get_value(1);
    let color_transform = match ColorTransformObject::cast(color_transform) {
        Some(color_transform) => (*color_transform).clone(),
        None => return Ok((-3).into()),
    };

    operations::color_transform(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        x.max(0) as u32,
        y.max(0) as u32,
        (x + width) as u32,
        (y + height) as u32,
        &color_transform.into(),
    );

    Ok((-1).into())
}

fn get_color_bounds_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let mask = args.get_u32(activation, 0)?;
    let color = args.get_u32(activation, 1)?;
    let find_color = args
        .try_get_bool(activation, 2, UndefinedAs::Some)
        .unwrap_or(true);

    let (x, y, w, h) = operations::color_bounds_rect(
        activation.context.renderer,
        bitmap_data,
        find_color,
        mask,
        color,
    );

    let proto = activation.prototypes().rectangle_constructor;
    let rect = proto.construct(activation, &[x.into(), y.into(), w.into(), h.into()])?;
    Ok(rect)
}

fn perlin_noise<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 6 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let base_x = args.get_f64(activation, 0)?;
    let base_y = args.get_f64(activation, 1)?;
    let num_octaves = args.get_i32(activation, 2)?.max(0) as usize;
    let seed = args.get_i32(activation, 3)? as i64;
    let stitch = args.get_bool(activation, 4);
    let fractal_noise = args.get_bool(activation, 5);
    let channel_options = if let Some(c) = args.try_get_u8(activation, 6, UndefinedAs::Some)? {
        ChannelOptions::from_bits_truncate(c)
    } else {
        ChannelOptions::RGB
    };
    let grayscale = args.get_bool(activation, 7);
    let offsets = args.get_object(activation, 8);

    let octave_offsets: Result<Vec<_>, Error<'gc>> = (0..num_octaves)
        .map(|i| {
            if let Value::Object(e) = offsets.get_element(activation, i as i32) {
                let x = e.get(istr!("x"), activation)?.coerce_to_f64(activation)?;
                let y = e.get(istr!("y"), activation)?.coerce_to_f64(activation)?;
                Ok((x, y))
            } else {
                Ok((0.0, 0.0))
            }
        })
        .collect();
    let octave_offsets = octave_offsets?;

    operations::perlin_noise(
        activation.gc(),
        bitmap_data,
        (base_x, base_y),
        num_octaves,
        seed,
        stitch,
        fractal_noise,
        channel_options,
        grayscale,
        octave_offsets,
    );

    Ok(0.into())
}

fn hit_test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 3 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let first_point = args.get_object(activation, 0);
    let top_left = if let (Some(x), Some(y)) = (
        first_point.get_local_stored(istr!("x"), activation),
        first_point.get_local_stored(istr!("y"), activation),
    ) {
        (x.coerce_to_i32(activation)?, y.coerce_to_i32(activation)?)
    } else {
        // Despite the AS docs saying this function returns `Boolean`, it returns a negative int on error conditions.
        // Invalid `firstPoint`.
        return Ok((-2).into());
    };
    let source_threshold = args.get_i32(activation, 1)?.clamp(0, u8::MAX.into()) as u8;
    let compare_object = args.get_object(activation, 2);

    // Overload based on the object we are hit-testing against.
    // BitmapData vs. BitmapData
    if let NativeObject::BitmapData(other_bmd) = compare_object.native() {
        if other_bmd.disposed() {
            return Ok((-3).into());
        }

        let second_point = args.get_object(activation, 3);
        let second_point = if let (Some(x), Some(y)) = (
            second_point.get_local_stored(istr!("x"), activation),
            second_point.get_local_stored(istr!("y"), activation),
        ) {
            (x.coerce_to_i32(activation)?, y.coerce_to_i32(activation)?)
        } else if args.len() > 3 {
            // Invalid `secondPoint`, but only if it's specified at all
            return Ok((-4).into());
        } else {
            (0, 0)
        };
        let second_threshold = args.get_i32(activation, 4)?.clamp(0, u8::MAX.into()) as u8;

        let result = operations::hit_test_bitmapdata(
            activation.context.renderer,
            bitmap_data,
            top_left,
            source_threshold,
            other_bmd,
            second_point,
            second_threshold,
        );
        Ok(Value::Bool(result))
    } else {
        // Determine what kind of Object we have, point or rectangle.
        // Duck-typed dumb objects are allowed.
        let compare_fields = (
            compare_object.get_local_stored(istr!("x"), activation),
            compare_object.get_local_stored(istr!("y"), activation),
            compare_object.get_local_stored(istr!("width"), activation),
            compare_object.get_local_stored(istr!("height"), activation),
        );
        match compare_fields {
            // BitmapData vs. point
            (Some(test_x), Some(test_y), None, None) => {
                let test_point = (
                    test_x.coerce_to_i32(activation)? - top_left.0,
                    test_y.coerce_to_i32(activation)? - top_left.1,
                );
                Ok(Value::Bool(operations::hit_test_point(
                    activation.context.renderer,
                    bitmap_data,
                    source_threshold,
                    test_point,
                )))
            }

            // BitmapData vs. rectangle
            (Some(test_x), Some(test_y), Some(test_width), Some(test_height)) => {
                let test_point = (
                    test_x.coerce_to_i32(activation)? - top_left.0,
                    test_y.coerce_to_i32(activation)? - top_left.1,
                );
                let size = (
                    test_width.coerce_to_i32(activation)?,
                    test_height.coerce_to_i32(activation)?,
                );
                Ok(Value::Bool(operations::hit_test_rectangle(
                    activation.context.renderer,
                    bitmap_data,
                    source_threshold,
                    test_point,
                    size,
                )))
            }

            // Invalid compare object.
            _ => Ok((-3).into()),
        }
    }
}

fn copy_pixels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 3 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };
    let src_bitmap = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => s,
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(_) => return Ok((-2).into()),
    };

    let source_rect = args.get_object(activation, 1);
    let Some((src_min_x, src_min_y, src_width, src_height)) =
        try_get_rect(activation, source_rect)?
    else {
        return Ok((-4).into());
    };

    let dest_point = args.get_object(activation, 2);

    let dest_x = dest_point
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)? as i32;
    let dest_y = dest_point
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)? as i32;

    let merge_alpha = if args.len() >= 6 {
        Some(args.get_bool(activation, 5))
    } else {
        None
    };

    // TODO: This needs testing, the method seems to do _something_ with a disposed BMD
    // It doesn't error out, but it's also not taking the regular `copy_pixels` path...
    if let BitmapDataResult::Valid(alpha_bitmap) = get_bitmap_data(args.get_object(activation, 3)) {
        let alpha_point = args.get_object(activation, 4);

        let alpha_x = alpha_point
            .get(istr!("x"), activation)?
            .coerce_to_f64(activation)? as i32;

        let alpha_y = alpha_point
            .get(istr!("y"), activation)?
            .coerce_to_f64(activation)? as i32;

        operations::copy_pixels_with_alpha_source(
            activation.context,
            bitmap_data,
            src_bitmap,
            (src_min_x, src_min_y, src_width, src_height),
            (dest_x, dest_y),
            alpha_bitmap,
            (alpha_x, alpha_y),
            merge_alpha.unwrap_or(true),
        );
    } else {
        operations::copy_pixels(
            activation.context,
            bitmap_data,
            src_bitmap,
            (src_min_x, src_min_y, src_width, src_height),
            (dest_x, dest_y),
            // Despite what the docs claim, mergeAlpa appears to be treated as 'false'
            // when no 'alphaBitmap' is specified (e.g. only 3 args are passed)
            merge_alpha.unwrap_or(false),
        );
    }

    Ok(0.into())
}

fn merge<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 6 {
        // Docs say 7 args is required, but alpha seems to be optional
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };
    let src_bitmap = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => s,
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(_) => return Ok((-2).into()),
    };

    let source_rect = args.get_object(activation, 1);
    let Some((src_min_x, src_min_y, src_width, src_height)) =
        try_get_rect(activation, source_rect)?
    else {
        return Ok((-4).into());
    };

    let dest_point = args.get_object(activation, 2);

    let dest_x = dest_point
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)? as i32;
    let dest_y = dest_point
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)? as i32;

    let red_mult = args.get_i32(activation, 3)?;
    let green_mult = args.get_i32(activation, 4)?;
    let blue_mult = args.get_i32(activation, 5)?;
    let alpha_mult = args
        .try_get_i32(activation, 6, UndefinedAs::Some)?
        .unwrap_or(0xFF);

    operations::merge(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        src_bitmap,
        (src_min_x, src_min_y, src_width, src_height),
        (dest_x, dest_y),
        (red_mult, green_mult, blue_mult, alpha_mult),
    );

    Ok((-1).into())
}

fn palette_map<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 4 {
        // Flash claims that only 3 arguments are required, but it doesn't do anything without redArray
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };
    let src_bitmap = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => s,
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(_) => return Ok((-2).into()),
    };

    let source_rect = args.get_object(activation, 1);
    let Some((src_min_x, src_min_y, src_width, src_height)) =
        try_get_rect(activation, source_rect)?
    else {
        return Ok((-4).into());
    };

    let dest_point = args.get_object(activation, 2);

    let dest_x = dest_point
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)? as i32;
    let dest_y = dest_point
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)? as i32;

    let mut get_channel = |index: usize, shift: usize| -> Result<[u32; 256], Error<'gc>> {
        let arg = args.get(index).unwrap_or(&Value::Null);
        let mut array = [0_u32; 256];
        for (i, item) in array.iter_mut().enumerate() {
            *item = if let Value::Object(arg) = arg {
                arg.get_element(activation, i as i32)
                    .coerce_to_u32(activation)?
                    & 0xFF
            } else {
                // This is an "identity mapping", fulfilling the part of the spec that
                // says that channels which have no array provided are simply copied.
                (i << shift) as u32
            }
        }
        Ok(array)
    };

    let red_array = get_channel(3, 16)?;
    let green_array = get_channel(4, 8)?;
    let blue_array = get_channel(5, 0)?;
    let alpha_array = get_channel(6, 24)?;

    operations::palette_map(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        src_bitmap,
        (src_min_x, src_min_y, src_width, src_height),
        (dest_x, dest_y),
        (red_array, green_array, blue_array, alpha_array),
    );

    Ok((-1).into())
}

fn pixel_dissolve<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 4 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };
    let src_bitmap_data = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => s,
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(_) => return Ok((-2).into()),
    };

    let source_rect = args.get_object(activation, 1);
    let Some((src_min_x, src_min_y, src_width, src_height)) =
        try_get_rect(activation, source_rect)?
    else {
        // Invalid `sourceRect`.
        return Ok((-4).into());
    };

    let dest_point = args.get_object(activation, 2);
    let dest_x = dest_point
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)? as i32;
    let dest_y = dest_point
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)? as i32;
    let dest_point = (dest_x, dest_y);

    let random_seed = match args.get(3) {
        Some(random_seed) => random_seed.coerce_to_i32(activation)?,
        None => 0,
    };

    let num_pixels = match args.get(4) {
        Some(num_pixels) => num_pixels.coerce_to_i32(activation)?,
        None => return Ok(0.into()),
    };

    let fill_color = match args.get(5) {
        Some(fill_color) => fill_color.coerce_to_u32(activation)?,
        None => 0,
    };

    Ok(operations::pixel_dissolve(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        src_bitmap_data,
        (src_min_x, src_min_y, src_width, src_height),
        dest_point,
        random_seed,
        num_pixels,
        fill_color,
    )
    .into())
}

fn scroll<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 2 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };

    let x = args.get_i32(activation, 0)?;
    let y = args.get_i32(activation, 1)?;

    operations::scroll(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        x,
        y,
    );

    Ok(0.into())
}

fn threshold<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if args.len() < 5 {
        return Ok((-1).into());
    }
    let BitmapDataResult::Valid(bitmap_data) = get_bitmap_data(this) else {
        return Ok((-1).into());
    };
    let src_bitmap = match get_bitmap_data(args.get_object(activation, 0)) {
        BitmapDataResult::Valid(s) => s,
        BitmapDataResult::Disposed => return Ok((-3).into()),
        BitmapDataResult::NotBitmapData(_) => return Ok((-2).into()),
    };

    let source_rect = args.get_object(activation, 1);
    let Some((src_min_x, src_min_y, src_width, src_height)) =
        try_get_rect(activation, source_rect)?
    else {
        return Ok((-4).into());
    };

    let dest_point = args.get_object(activation, 2);

    let dest_x = dest_point
        .get(istr!("x"), activation)?
        .coerce_to_f64(activation)? as i32;
    let dest_y = dest_point
        .get(istr!("y"), activation)?
        .coerce_to_f64(activation)? as i32;

    let operation = ThresholdOperation::from_wstr(&args.get_string(activation, 3)?)
        .unwrap_or(ThresholdOperation::LessThan);
    let threshold = args.get_u32(activation, 4)?;
    let colour = args.get_u32(activation, 5)?;
    let mask = args
        .try_get_u32(activation, 6, UndefinedAs::Some)?
        .unwrap_or(0xFFFFFFFF);
    let copy_source = args.get_bool(activation, 7);

    let modified_count = operations::threshold(
        activation.gc(),
        activation.context.renderer,
        bitmap_data,
        src_bitmap,
        (src_min_x, src_min_y, src_width, src_height),
        (dest_x, dest_y),
        operation,
        threshold,
        colour,
        mask,
        copy_source,
    );

    Ok(modified_count.into())
}

fn compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Note that these error values are specific to this function, there's no standard between functions
    const EQUIVALENT: i32 = 0;
    const NOT_BITMAP: i32 = -1;
    const BITMAP_DISPOSED: i32 = -2;
    const DIFFERENT_WIDTHS: i32 = -3;
    const DIFFERENT_HEIGHTS: i32 = -4;
    if args.is_empty() {
        return Ok((NOT_BITMAP).into());
    }

    let BitmapDataResult::Valid(this_bitmap_data) = get_bitmap_data(this) else {
        return Ok((NOT_BITMAP).into());
    };

    let BitmapDataResult::Valid(other_bitmap_data) =
        get_bitmap_data(args.get_object(activation, 0))
    else {
        return Ok(BITMAP_DISPOSED.into());
    };

    if this_bitmap_data.width() != other_bitmap_data.width() {
        return Ok(DIFFERENT_WIDTHS.into());
    }

    if this_bitmap_data.height() != other_bitmap_data.height() {
        return Ok(DIFFERENT_HEIGHTS.into());
    }

    match operations::compare(
        activation.context.gc_context,
        activation.context.renderer,
        this_bitmap_data,
        other_bitmap_data,
    ) {
        Some(bitmap_data) => Ok(new_bitmap_data(
            this.get_local_stored(istr!("__proto__"), activation),
            bitmap_data,
            activation,
        )
        .into()),
        None => Ok(EQUIVALENT.into()),
    }
}

fn load_bitmap<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    let library = &*activation.context.library;

    let movie = <DisplayObject as crate::display_object::TDisplayObject>::movie(
        activation.target_clip_or_root(),
    );

    let character = library
        .library_for_movie(movie)
        .and_then(|l| l.character_by_export_name(name));

    let Some((_id, Character::Bitmap(bitmap))) = character else {
        return Ok(Value::Undefined);
    };
    let bitmap = bitmap.compressed().decode().unwrap();

    let transparency = true;
    let bitmap_data = BitmapData::new_with_pixels(
        activation.context.gc_context,
        bitmap.width(),
        bitmap.height(),
        transparency,
        bitmap
            .as_colors()
            .map(crate::bitmap::bitmap_data::Color::from)
            .collect(),
    );
    Ok(new_bitmap_data(
        this.get_local_stored(istr!("prototype"), activation),
        bitmap_data,
        activation,
    )
    .into())
}
