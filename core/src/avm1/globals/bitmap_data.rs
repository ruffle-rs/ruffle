//! flash.display.BitmapData object

use super::matrix::object_to_matrix;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::globals::bitmap_filter;
use crate::avm1::globals::color_transform::ColorTransformObject;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Attribute, Error, Object, ScriptObject, TObject, Value};
use crate::bitmap::bitmap_data::{BitmapData, BitmapDataWrapper};
use crate::bitmap::bitmap_data::{BitmapDataDrawError, IBitmapDrawable};
use crate::bitmap::bitmap_data::{ChannelOptions, ThresholdOperation};
use crate::bitmap::{is_size_valid, operations};
use crate::character::Character;
use crate::context::GcContext;
use crate::display_object::DisplayObject;
use crate::swf::BlendMode;
use crate::{avm1_stub, avm_error};
use gc_arena::{GcCell, Mutation};
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
};

fn new_bitmap_data<'gc>(
    gc_context: &Mutation<'gc>,
    proto: Option<Value<'gc>>,
    bitmap_data: BitmapData<'gc>,
) -> ScriptObject<'gc> {
    let object = ScriptObject::new(gc_context, None);
    // Set `__proto__` manually since `ScriptObject::new()` doesn't support primitive prototypes.
    // TODO: Pass `proto` to `ScriptObject::new()` once possible.
    if let Some(proto) = proto {
        object.define_value(
            gc_context,
            "__proto__",
            proto,
            Attribute::DONT_ENUM | Attribute::DONT_DELETE,
        );
    }
    object.set_native(
        gc_context,
        NativeObject::BitmapData(BitmapDataWrapper::new(GcCell::new(gc_context, bitmap_data))),
    );
    object
}

fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let (width, height) = match args {
        [width, height, ..] => (
            width.coerce_to_u32(activation)?,
            height.coerce_to_u32(activation)?,
        ),
        [] | [_] => return Ok(Value::Undefined),
    };
    let transparency = match args.get(2) {
        Some(transparency) => transparency.as_bool(activation.swf_version()),
        None => true,
    };
    let fill_color = match args.get(3) {
        Some(fill_color) => fill_color.coerce_to_u32(activation)?,
        None => u32::MAX,
    };

    if !is_size_valid(activation.swf_version(), width, height) {
        tracing::warn!("Invalid BitmapData size: {}x{}", width, height);
        return Ok(Value::Undefined);
    }

    let bitmap_data = BitmapData::new(width, height, transparency, fill_color);
    this.set_native(
        activation.context.gc_context,
        NativeObject::BitmapData(BitmapDataWrapper::new(GcCell::new(
            activation.context.gc_context,
            bitmap_data,
        ))),
    );
    Ok(this.into())
}

fn height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            return Ok(bitmap_data.height().into());
        }
    }

    Ok((-1).into())
}

fn width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            return Ok(bitmap_data.width().into());
        }
    }

    Ok((-1).into())
}

fn get_transparent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            return Ok(bitmap_data.transparency().into());
        }
    }

    Ok((-1).into())
}

fn get_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let proto = activation.context.avm1.prototypes().rectangle_constructor;
            let rect = proto.construct(
                activation,
                &[
                    0.into(),
                    0.into(),
                    bitmap_data.width().into(),
                    bitmap_data.height().into(),
                ],
            )?;
            return Ok(rect);
        }
    }

    Ok((-1).into())
}

fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val)) = (args.get(0), args.get(1)) {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                // AVM1 returns a signed int, so we need to convert it.
                let col =
                    operations::get_pixel(bitmap_data, activation.context.renderer, x, y) as i32;
                return Ok(col.into());
            }
        }
    }

    Ok((-1).into())
}

fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val)) = (args.get(0), args.get(1)) {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                // AVM1 returns a signed int, so we need to convert it.
                let col =
                    operations::get_pixel32(bitmap_data, activation.context.renderer, x, y) as i32;
                return Ok(col.into());
            }
        }
    }

    Ok((-1).into())
}

fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val), Some(color_val)) =
                (args.get(0), args.get(1), args.get(2))
            {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                let color = color_val.coerce_to_u32(activation)?;

                operations::set_pixel(
                    activation.context.gc_context,
                    activation.context.renderer,
                    bitmap_data,
                    x,
                    y,
                    color.into(),
                );

                return Ok(Value::Undefined);
            }
        }
    }

    Ok((-1).into())
}

fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val), Some(color_val)) =
                (args.get(0), args.get(1), args.get(2))
            {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                let color = color_val.coerce_to_u32(activation)?;

                operations::set_pixel32(
                    activation.context.gc_context,
                    activation.context.renderer,
                    bitmap_data,
                    x,
                    y,
                    color,
                );
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let source_bitmap = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let source_rect = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let dest_point = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let source_channel = args
        .get(3)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    let dest_channel = args
        .get(4)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let NativeObject::BitmapData(source_bitmap) = source_bitmap.native() {
                //TODO: what if source is disposed
                let min_x = dest_point.get("x", activation)?.coerce_to_i32(activation)?;
                let min_y = dest_point.get("y", activation)?.coerce_to_i32(activation)?;

                let src_min_x = source_rect
                    .get("x", activation)?
                    .coerce_to_i32(activation)?;
                let src_min_y = source_rect
                    .get("y", activation)?
                    .coerce_to_i32(activation)?;
                let src_width = source_rect
                    .get("width", activation)?
                    .coerce_to_i32(activation)?;
                let src_height = source_rect
                    .get("height", activation)?
                    .coerce_to_i32(activation)?;

                operations::copy_channel(
                    activation.context.gc_context,
                    activation.context.renderer,
                    bitmap_data,
                    (min_x, min_y),
                    (src_min_x, src_min_y, src_width, src_height),
                    source_bitmap,
                    source_channel,
                    dest_channel,
                );
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let Some(color_val) = args.get(1) {
                let color = color_val.coerce_to_u32(activation)?;

                let x = rectangle.get("x", activation)?.coerce_to_i32(activation)?;
                let y = rectangle.get("y", activation)?.coerce_to_i32(activation)?;
                let width = rectangle
                    .get("width", activation)?
                    .coerce_to_i32(activation)?;
                let height = rectangle
                    .get("height", activation)?
                    .coerce_to_i32(activation)?;

                operations::fill_rect(
                    activation.context.gc_context,
                    activation.context.renderer,
                    bitmap_data,
                    x,
                    y,
                    width,
                    height,
                    color,
                );
            }
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            return Ok(new_bitmap_data(
                activation.context.gc_context,
                this.get_local_stored("__proto__", activation, false),
                bitmap_data.clone_data(activation.context.renderer),
            )
            .into());
        }
    }

    Ok((-1).into())
}

fn dispose<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            bitmap_data.dispose(activation.context.gc_context);
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val), Some(color_val)) =
                (args.get(0), args.get(1), args.get(2))
            {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                let color = color_val.coerce_to_u32(activation)?;

                operations::flood_fill(
                    activation.context.gc_context,
                    activation.context.renderer,
                    bitmap_data,
                    x,
                    y,
                    color,
                );
            }
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn noise<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let low = args.get(1).unwrap_or(&0.into()).coerce_to_u32(activation)? as u8;

    let high = args
        .get(2)
        .unwrap_or(&0xFF.into())
        .coerce_to_u32(activation)? as u8;

    let channel_options = if let Some(c) = args.get(3) {
        ChannelOptions::from_bits_truncate(c.coerce_to_u32(activation)? as u8)
    } else {
        ChannelOptions::RGB
    };

    let gray_scale = args
        .get(4)
        .unwrap_or(&false.into())
        .as_bool(activation.swf_version());

    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let Some(random_seed_val) = args.get(0) {
                let random_seed = random_seed_val.coerce_to_i32(activation)?;
                operations::noise(
                    activation.context.gc_context,
                    bitmap_data,
                    random_seed,
                    low,
                    high.max(low),
                    channel_options,
                    gray_scale,
                )
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn draw<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let matrix = args
                .get(1)
                .map(|o| o.coerce_to_object(activation))
                .and_then(|o| object_to_matrix(o, activation).ok())
                .unwrap_or_default();

            let color_transform = args
                .get(2)
                .and_then(|v| ColorTransformObject::cast(*v))
                .map(|color_transform| color_transform.read().clone().into())
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

            let source = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);
            let source = if let Some(source_object) = source.as_display_object() {
                IBitmapDrawable::DisplayObject(source_object)
            } else if let NativeObject::BitmapData(source_bitmap) = source.native() {
                IBitmapDrawable::BitmapData(source_bitmap)
            } else {
                avm_error!(
                    activation,
                    "BitmapData.draw: Unexpected source {:?} {:?}",
                    source,
                    args.get(0)
                );
                return Ok(Value::Undefined);
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
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn apply_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let source = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);
            let source = if let NativeObject::BitmapData(source_bitmap) = source.native() {
                source_bitmap
            } else {
                tracing::warn!(
                    "Invalid bitmapdata source for apply_filter: got {:?}",
                    source
                );
                return Ok((-1).into());
            };

            let source_rect = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let src_min_x = source_rect
                .get("x", activation)?
                .coerce_to_f64(activation)? as u32;
            let src_min_y = source_rect
                .get("y", activation)?
                .coerce_to_f64(activation)? as u32;
            let src_width = source_rect
                .get("width", activation)?
                .coerce_to_f64(activation)? as u32;
            let src_height = source_rect
                .get("height", activation)?
                .coerce_to_f64(activation)? as u32;

            let dest_point = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let dest_x = dest_point.get("x", activation)?.coerce_to_f64(activation)? as u32;
            let dest_y = dest_point.get("y", activation)?.coerce_to_f64(activation)? as u32;

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
        }
    }

    Ok((-1).into())
}

fn generate_filter_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            avm1_stub!(activation, "BitmapData", "generateFilterRect");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            if let [rectangle, color_transform, ..] = args {
                // TODO: Re-use `object_to_rectangle` in `movie_clip.rs`.
                let rectangle = rectangle.coerce_to_object(activation);
                let x = rectangle.get("x", activation)?.coerce_to_f64(activation)? as i32;
                let y = rectangle.get("y", activation)?.coerce_to_f64(activation)? as i32;
                let width = rectangle
                    .get("width", activation)?
                    .coerce_to_f64(activation)? as i32;
                let height = rectangle
                    .get("height", activation)?
                    .coerce_to_f64(activation)? as i32;

                let x_min = x.max(0) as u32;
                let x_max = (x + width) as u32;
                let y_min = y.max(0) as u32;
                let y_max = (y + height) as u32;

                let color_transform = match ColorTransformObject::cast(*color_transform) {
                    Some(color_transform) => color_transform.read().clone(),
                    None => return Ok((-3).into()),
                };

                operations::color_transform(
                    activation.context.gc_context,
                    activation.context.renderer,
                    bitmap_data,
                    x_min,
                    y_min,
                    x_max,
                    y_max,
                    &color_transform.into(),
                );
            }
        }
    }

    Ok((-1).into())
}

fn get_color_bounds_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let find_color = args
                .get(2)
                .unwrap_or(&true.into())
                .as_bool(activation.swf_version());

            if let (Some(mask_val), Some(color_val)) = (args.get(0), args.get(1)) {
                let mask = mask_val.coerce_to_u32(activation)?;
                let color = color_val.coerce_to_u32(activation)?;

                let (x, y, w, h) = operations::color_bounds_rect(
                    activation.context.renderer,
                    bitmap_data,
                    find_color,
                    mask,
                    color,
                );

                let proto = activation.context.avm1.prototypes().rectangle_constructor;
                let rect =
                    proto.construct(activation, &[x.into(), y.into(), w.into(), h.into()])?;
                return Ok(rect);
            }
        }
    }

    Ok((-1).into())
}

fn perlin_noise<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let base_x = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_f64(activation)?;
            let base_y = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_f64(activation)?;
            let num_octaves = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_u32(activation)? as usize;
            let seed = args
                .get(3)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)? as i64;
            let stitch = args
                .get(4)
                .unwrap_or(&Value::Undefined)
                .as_bool(activation.swf_version());
            let fractal_noise = args
                .get(5)
                .unwrap_or(&Value::Undefined)
                .as_bool(activation.swf_version());
            let channel_options = if let Some(c) = args.get(6) {
                ChannelOptions::from_bits_truncate(c.coerce_to_i16(activation)? as u8)
            } else {
                ChannelOptions::RGB
            };
            let grayscale = args
                .get(7)
                .unwrap_or(&Value::Undefined)
                .as_bool(activation.swf_version());
            let offsets = args
                .get(8)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let octave_offsets: Result<Vec<_>, Error<'gc>> = (0..num_octaves)
                .map(|i| {
                    if let Value::Object(e) = offsets.get_element(activation, i as i32) {
                        let x = e.get("x", activation)?.coerce_to_f64(activation)?;
                        let y = e.get("y", activation)?.coerce_to_f64(activation)?;
                        Ok((x, y))
                    } else {
                        Ok((0.0, 0.0))
                    }
                })
                .collect();
            let octave_offsets = octave_offsets?;

            operations::perlin_noise(
                activation.context.gc_context,
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
        }
    }

    Ok((-1).into())
}

fn hit_test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let first_point = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);
            let top_left = if let (Some(x), Some(y)) = (
                first_point.get_local_stored("x", activation, false),
                first_point.get_local_stored("y", activation, false),
            ) {
                (x.coerce_to_i32(activation)?, y.coerce_to_i32(activation)?)
            } else {
                // Despite the AS docs saying this function returns `Boolean`, it returns a negative int on error conditions.
                // Invalid `firstPoint`.
                return Ok((-2).into());
            };
            let source_threshold = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?
                .clamp(0, u8::MAX.into()) as u8;
            let compare_object = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            // Overload based on the object we are hit-testing against.
            // BitmapData vs. BitmapData
            if let NativeObject::BitmapData(other_bmd) = compare_object.native() {
                if other_bmd.disposed() {
                    return Ok((-3).into());
                }

                let second_point = args
                    .get(3)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_object(activation);
                let second_point = if let (Some(x), Some(y)) = (
                    second_point.get_local_stored("x", activation, false),
                    second_point.get_local_stored("y", activation, false),
                ) {
                    (x.coerce_to_i32(activation)?, y.coerce_to_i32(activation)?)
                } else {
                    // Invalid `secondPoint`.
                    return Ok((-4).into());
                };
                let second_threshold = args
                    .get(4)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_i32(activation)?
                    .clamp(0, u8::MAX.into()) as u8;

                let result = operations::hit_test_bitmapdata(
                    activation.context.renderer,
                    bitmap_data,
                    top_left,
                    source_threshold,
                    other_bmd,
                    second_point,
                    second_threshold,
                );
                return Ok(Value::Bool(result));
            } else {
                // Determine what kind of Object we have, point or rectangle.
                // Duck-typed dumb objects are allowed.
                let compare_fields = (
                    compare_object.get_local_stored("x", activation, false),
                    compare_object.get_local_stored("y", activation, false),
                    compare_object.get_local_stored("width", activation, false),
                    compare_object.get_local_stored("height", activation, false),
                );
                match compare_fields {
                    // BitmapData vs. point
                    (Some(test_x), Some(test_y), None, None) => {
                        let test_point = (
                            test_x.coerce_to_i32(activation)? - top_left.0,
                            test_y.coerce_to_i32(activation)? - top_left.1,
                        );
                        return Ok(Value::Bool(operations::hit_test_point(
                            activation.context.renderer,
                            bitmap_data,
                            source_threshold,
                            test_point,
                        )));
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
                        return Ok(Value::Bool(operations::hit_test_rectangle(
                            activation.context.renderer,
                            bitmap_data,
                            source_threshold,
                            test_point,
                            size,
                        )));
                    }

                    // Invalid compare object.
                    _ => {
                        return Ok((-3).into());
                    }
                }
            }
        }
    }

    // Disposed or invalid bitmap.
    Ok((-1).into())
}

fn copy_pixels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let source_bitmap = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let source_rect = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let src_min_x = source_rect
                .get("x", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_min_y = source_rect
                .get("y", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_width = source_rect
                .get("width", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_height = source_rect
                .get("height", activation)?
                .coerce_to_f64(activation)? as i32;

            let dest_point = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let dest_x = dest_point.get("x", activation)?.coerce_to_f64(activation)? as i32;
            let dest_y = dest_point.get("y", activation)?.coerce_to_f64(activation)? as i32;

            if let NativeObject::BitmapData(src_bitmap) = source_bitmap.native() {
                if !src_bitmap.disposed() {
                    let merge_alpha = if args.len() >= 6 {
                        Some(
                            args.get(5)
                                .unwrap_or(&Value::Undefined)
                                .as_bool(activation.swf_version()),
                        )
                    } else {
                        None
                    };

                    let alpha_bitmap = args
                        .get(3)
                        .unwrap_or(&Value::Undefined)
                        .coerce_to_object(activation);

                    if let NativeObject::BitmapData(alpha_bitmap) = alpha_bitmap.native() {
                        if !alpha_bitmap.disposed() {
                            let alpha_point = args
                                .get(4)
                                .unwrap_or(&Value::Undefined)
                                .coerce_to_object(activation);

                            let alpha_x = alpha_point
                                .get("x", activation)?
                                .coerce_to_f64(activation)?
                                as i32;

                            let alpha_y = alpha_point
                                .get("y", activation)?
                                .coerce_to_f64(activation)?
                                as i32;

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
                        }
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
                }
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn merge<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let source_bitmap = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let source_rect = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let src_min_x = source_rect
                .get("x", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_min_y = source_rect
                .get("y", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_width = source_rect
                .get("width", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_height = source_rect
                .get("height", activation)?
                .coerce_to_f64(activation)? as i32;

            let dest_point = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let dest_x = dest_point.get("x", activation)?.coerce_to_f64(activation)? as i32;
            let dest_y = dest_point.get("y", activation)?.coerce_to_f64(activation)? as i32;

            let red_mult = args
                .get(3)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;

            let green_mult = args
                .get(4)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;

            let blue_mult = args
                .get(5)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;

            let alpha_mult = args
                .get(6)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;

            if let NativeObject::BitmapData(src_bitmap) = source_bitmap.native() {
                if !src_bitmap.disposed() {
                    operations::merge(
                        activation.context.gc_context,
                        activation.context.renderer,
                        bitmap_data,
                        src_bitmap,
                        (src_min_x, src_min_y, src_width, src_height),
                        (dest_x, dest_y),
                        (red_mult, green_mult, blue_mult, alpha_mult),
                    );
                }
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn palette_map<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let source_bitmap = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let source_rect = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let src_min_x = source_rect
                .get("x", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_min_y = source_rect
                .get("y", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_width = source_rect
                .get("width", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_height = source_rect
                .get("height", activation)?
                .coerce_to_f64(activation)? as i32;

            let dest_point = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let dest_x = dest_point.get("x", activation)?.coerce_to_f64(activation)? as i32;
            let dest_y = dest_point.get("y", activation)?.coerce_to_f64(activation)? as i32;

            let mut get_channel = |index: usize, shift: usize| -> Result<[u32; 256], Error<'gc>> {
                let arg = args.get(index).unwrap_or(&Value::Null);
                let mut array = [0_u32; 256];
                for (i, item) in array.iter_mut().enumerate() {
                    *item = if let Value::Object(arg) = arg {
                        arg.get_element(activation, i as i32)
                            .coerce_to_u32(activation)?
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

            if let NativeObject::BitmapData(src_bitmap) = source_bitmap.native() {
                if !src_bitmap.disposed() {
                    operations::palette_map(
                        activation.context.gc_context,
                        activation.context.renderer,
                        bitmap_data,
                        src_bitmap,
                        (src_min_x, src_min_y, src_width, src_height),
                        (dest_x, dest_y),
                        (red_array, green_array, blue_array, alpha_array),
                    );
                }
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn pixel_dissolve<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let src_bitmap_data = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let source_rect = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);
            let (src_min_x, src_min_y, src_width, src_height) =
                if let (Some(x), Some(y), Some(width), Some(height)) = (
                    source_rect.get_local_stored("x", activation, false),
                    source_rect.get_local_stored("y", activation, false),
                    source_rect.get_local_stored("width", activation, false),
                    source_rect.get_local_stored("height", activation, false),
                ) {
                    (
                        x.coerce_to_f64(activation)? as i32,
                        y.coerce_to_f64(activation)? as i32,
                        width.coerce_to_f64(activation)? as i32,
                        height.coerce_to_f64(activation)? as i32,
                    )
                } else {
                    // Invalid `sourceRect`.
                    return Ok((-4).into());
                };

            if let NativeObject::BitmapData(src_bitmap_data) = src_bitmap_data.native() {
                if !src_bitmap_data.disposed() {
                    let dest_point = args
                        .get(2)
                        .unwrap_or(&Value::Undefined)
                        .coerce_to_object(activation);
                    let dest_x = dest_point.get("x", activation)?.coerce_to_f64(activation)? as i32;
                    let dest_y = dest_point.get("y", activation)?.coerce_to_f64(activation)? as i32;
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

                    return Ok(operations::pixel_dissolve(
                        activation.context.gc_context,
                        activation.context.renderer,
                        bitmap_data,
                        src_bitmap_data,
                        (src_min_x, src_min_y, src_width, src_height),
                        dest_point,
                        random_seed,
                        num_pixels,
                        fill_color,
                    )
                    .into());
                }
            }
        }
    }

    Ok((-1).into())
}

fn scroll<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let x = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;
            let y = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;

            operations::scroll(
                activation.context.gc_context,
                activation.context.renderer,
                bitmap_data,
                x,
                y,
            );

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn threshold<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let NativeObject::BitmapData(bitmap_data) = this.native() {
        if !bitmap_data.disposed() {
            let source_bitmap = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let source_rect = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let src_min_x = source_rect
                .get("x", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_min_y = source_rect
                .get("y", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_width = source_rect
                .get("width", activation)?
                .coerce_to_f64(activation)? as i32;
            let src_height = source_rect
                .get("height", activation)?
                .coerce_to_f64(activation)? as i32;

            let dest_point = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let dest_x = dest_point.get("x", activation)?.coerce_to_f64(activation)? as i32;
            let dest_y = dest_point.get("y", activation)?.coerce_to_f64(activation)? as i32;

            let operation = args.get(3);
            let operation = match ThresholdOperation::from_wstr(
                &operation
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_string(activation)?,
            ) {
                Some(operation) => operation,
                None => return Ok(0.into()),
            };

            let threshold = args
                .get(4)
                .unwrap_or(&Value::Undefined)
                .coerce_to_u32(activation)?;

            let colour = args.get(5).unwrap_or(&0.into()).coerce_to_u32(activation)?;

            let mask = args
                .get(6)
                .unwrap_or(&0xFFFFFFFFu32.into())
                .coerce_to_u32(activation)?;

            let copy_source = args
                .get(7)
                .unwrap_or(&false.into())
                .as_bool(activation.swf_version());

            if let NativeObject::BitmapData(src_bitmap) = source_bitmap.native() {
                if !src_bitmap.disposed() {
                    let modified_count = operations::threshold(
                        activation.context.gc_context,
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

                    return Ok(modified_count.into());
                }
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

fn compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    const EQUIVALENT: i32 = 0;
    const NOT_BITMAP: i32 = -1;
    const BITMAP_DISPOSED: i32 = -2;
    const DIFFERENT_WIDTHS: i32 = -3;
    const DIFFERENT_HEIGHTS: i32 = -4;

    let NativeObject::BitmapData(this_bitmap_data) = this.native() else {
        return Ok(NOT_BITMAP.into());
    };

    if this_bitmap_data.disposed() {
        // The documentation says that -2 should be returned here, but -1 is actually returned.
        return Ok(NOT_BITMAP.into());
    }

    let other = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let NativeObject::BitmapData(other_bitmap_data) = other.native() else {
        // The documentation says that -1 should be returned here, but -2 is actually returned.
        return Ok(BITMAP_DISPOSED.into());
    };

    if other_bitmap_data.disposed() {
        return Ok(BITMAP_DISPOSED.into());
    }

    if this_bitmap_data.width() != other_bitmap_data.width() {
        return Ok(DIFFERENT_WIDTHS.into());
    }

    if this_bitmap_data.height() != other_bitmap_data.height() {
        return Ok(DIFFERENT_HEIGHTS.into());
    }

    match operations::compare(
        activation.context.renderer,
        this_bitmap_data,
        other_bitmap_data,
    ) {
        Some(bitmap_data) => Ok(new_bitmap_data(
            activation.context.gc_context,
            this.get_local_stored("__proto__", activation, false),
            bitmap_data,
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
        &activation.target_clip_or_root(),
    );

    let character = library
        .library_for_movie(movie)
        .and_then(|l| l.character_by_export_name(name));

    let Some((_id, Character::Bitmap { compressed, .. })) = character else {
        return Ok(Value::Undefined);
    };
    let bitmap = compressed.decode().unwrap();

    let transparency = true;
    let bitmap_data = BitmapData::new_with_pixels(
        bitmap.width(),
        bitmap.height(),
        transparency,
        bitmap
            .as_colors()
            .map(crate::bitmap::bitmap_data::Color::from)
            .collect(),
    );
    Ok(new_bitmap_data(
        activation.context.gc_context,
        this.get_local_stored("prototype", activation, false),
        bitmap_data,
    )
    .into())
}

pub fn create_constructor<'gc>(
    context: &mut GcContext<'_, 'gc>,
    proto: ScriptObject<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    define_properties_on(PROTO_DECLS, context, proto, fn_proto);

    let bitmap_data_constructor = FunctionObject::constructor(
        context.gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        fn_proto,
        proto.into(),
    );
    let object = bitmap_data_constructor.raw_script_object();
    define_properties_on(OBJECT_DECLS, context, object, fn_proto);
    bitmap_data_constructor
}
