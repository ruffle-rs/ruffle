//! flash.display.BitmapData object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::bitmap_data::BitmapDataObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::bitmap::bitmap_data::{BitmapData, ChannelOptions, Color};
use crate::bitmap::is_size_valid;
use crate::character::Character;
use crate::display_object::TDisplayObject;
use gc_arena::{GcCell, MutationContext};

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

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let width = args.get(0).unwrap_or(&0.into()).coerce_to_i32(activation)? as u32;

    let height = args.get(1).unwrap_or(&0.into()).coerce_to_i32(activation)? as u32;

    let transparency = args
        .get(2)
        .unwrap_or(&true.into())
        .as_bool(activation.swf_version());

    let fill_color = args
        .get(3)
        .unwrap_or(&(-1).into())
        .coerce_to_i32(activation)?;

    if !is_size_valid(activation.swf_version(), width, height) {
        log::warn!("Invalid BitmapData size: {}x{}", width, height);
        return Ok(Value::Undefined);
    }

    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        bitmap_data
            .bitmap_data()
            .write(activation.context.gc_context)
            .init_pixels(width, height, transparency, fill_color);
    }

    Ok(this.into())
}

pub fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            return Ok(bitmap_data.bitmap_data().read().height().into());
        }
    }

    Ok((-1).into())
}

pub fn width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            return Ok(bitmap_data.bitmap_data().read().width().into());
        }
    }

    Ok((-1).into())
}

pub fn get_transparent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            return Ok(bitmap_data.bitmap_data().read().transparency().into());
        }
    }

    Ok((-1).into())
}

pub fn get_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            let proto = activation.context.avm1.prototypes().rectangle_constructor;
            let rect = proto.construct(
                activation,
                &[
                    0.into(),
                    0.into(),
                    bitmap_data.bitmap_data().read().width().into(),
                    bitmap_data.bitmap_data().read().height().into(),
                ],
            )?;
            return Ok(rect);
        }
    }

    Ok((-1).into())
}

pub fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val)) = (args.get(0), args.get(1)) {
                let x = x_val.coerce_to_i32(activation)?;
                let y = y_val.coerce_to_i32(activation)?;
                return Ok(bitmap_data.bitmap_data().read().get_pixel(x, y).into());
            }
        }
    }

    Ok((-1).into())
}

pub fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val)) = (args.get(0), args.get(1)) {
                let x = x_val.coerce_to_i32(activation)?;
                let y = y_val.coerce_to_i32(activation)?;
                let col: i32 = bitmap_data.bitmap_data().read().get_pixel32(x, y).into();
                return Ok(col.into());
            }
        }
    }

    Ok((-1).into())
}

pub fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val), Some(color_val)) =
                (args.get(0), args.get(1), args.get(2))
            {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                let color = color_val.coerce_to_i32(activation)?;

                bitmap_data
                    .bitmap_data()
                    .write(activation.context.gc_context)
                    .set_pixel(x, y, color.into());

                return Ok(Value::Undefined);
            }
        }
    }

    Ok((-1).into())
}

pub fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val), Some(color_val)) =
                (args.get(0), args.get(1), args.get(2))
            {
                let x = x_val.coerce_to_i32(activation)?;
                let y = y_val.coerce_to_i32(activation)?;
                let color = color_val.coerce_to_i32(activation)?;

                bitmap_data
                    .bitmap_data()
                    .write(activation.context.gc_context)
                    .set_pixel32(x, y, color.into());
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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

    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let Some(source_bitmap) = source_bitmap.as_bitmap_data_object() {
                //TODO: what if source is disposed
                let min_x = dest_point
                    .get("x", activation)?
                    .coerce_to_u32(activation)?
                    .min(bitmap_data.bitmap_data().read().width());
                let min_y = dest_point
                    .get("y", activation)?
                    .coerce_to_u32(activation)?
                    .min(bitmap_data.bitmap_data().read().height());

                let src_min_x = source_rect
                    .get("x", activation)?
                    .coerce_to_u32(activation)?;
                let src_min_y = source_rect
                    .get("y", activation)?
                    .coerce_to_u32(activation)?;
                let src_width = source_rect
                    .get("width", activation)?
                    .coerce_to_u32(activation)?;
                let src_height = source_rect
                    .get("height", activation)?
                    .coerce_to_u32(activation)?;
                let src_max_x = src_min_x + src_width;
                let src_max_y = src_min_y + src_height;

                let src_bitmap_data = source_bitmap.bitmap_data();

                if GcCell::ptr_eq(bitmap_data.bitmap_data(), src_bitmap_data) {
                    let src_bitmap_data_clone = src_bitmap_data.read().clone();
                    bitmap_data
                        .bitmap_data()
                        .write(activation.context.gc_context)
                        .copy_channel(
                            (min_x, min_y),
                            (src_min_x, src_min_y, src_max_x, src_max_y),
                            &src_bitmap_data_clone,
                            source_channel,
                            dest_channel,
                        );
                } else {
                    bitmap_data
                        .bitmap_data()
                        .write(activation.context.gc_context)
                        .copy_channel(
                            (min_x, min_y),
                            (src_min_x, src_min_y, src_max_x, src_max_y),
                            &src_bitmap_data.read(),
                            source_channel,
                            dest_channel,
                        );
                }
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let Some(color_val) = args.get(1) {
                let color = color_val.coerce_to_i32(activation)?;

                let x = rectangle.get("x", activation)?.coerce_to_u32(activation)?;
                let y = rectangle.get("y", activation)?.coerce_to_u32(activation)?;
                let width = rectangle
                    .get("width", activation)?
                    .coerce_to_u32(activation)?;
                let height = rectangle
                    .get("height", activation)?
                    .coerce_to_u32(activation)?;

                bitmap_data
                    .bitmap_data()
                    .write(activation.context.gc_context)
                    .fill_rect(x, y, width, height, color.into());
            }
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            let new_bitmap_data = BitmapDataObject::empty_object(
                activation.context.gc_context,
                Some(activation.context.avm1.prototypes().bitmap_data),
            );

            new_bitmap_data
                .as_bitmap_data_object()
                .unwrap()
                .bitmap_data()
                .write(activation.context.gc_context)
                .set_pixels(
                    bitmap_data.bitmap_data().read().width(),
                    bitmap_data.bitmap_data().read().height(),
                    bitmap_data.bitmap_data().read().transparency(),
                    bitmap_data.bitmap_data().read().pixels().to_vec(),
                );

            return Ok(new_bitmap_data.into());
        }
    }

    Ok((-1).into())
}

pub fn dispose<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            bitmap_data.dispose(&mut activation.context);
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val), Some(color_val)) =
                (args.get(0), args.get(1), args.get(2))
            {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                let color = color_val.coerce_to_i32(activation)?;

                let color: Color = color.into();
                let color: Color =
                    color.to_premultiplied_alpha(bitmap_data.bitmap_data().read().transparency());

                bitmap_data
                    .bitmap_data()
                    .write(activation.context.gc_context)
                    .flood_fill(x, y, color);
            }
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn noise<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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

    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let Some(random_seed_val) = args.get(0) {
                let random_seed = random_seed_val.coerce_to_i32(activation)?;
                bitmap_data
                    .bitmap_data()
                    .write(activation.context.gc_context)
                    .noise(random_seed, low, high.max(low), channel_options, gray_scale)
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn apply_filter<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("BitmapData.applyFilter - not yet implemented");
    Ok((-1).into())
}

pub fn draw<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.draw - not yet implemented");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn generate_filter_rect<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.generateFilterRect - not yet implemented");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            let rectangle = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let color_transform = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let x = rectangle.get("x", activation)?.coerce_to_f64(activation)? as i32;
            let y = rectangle.get("y", activation)?.coerce_to_f64(activation)? as i32;
            let width = rectangle
                .get("width", activation)?
                .coerce_to_f64(activation)? as i32;
            let height = rectangle
                .get("height", activation)?
                .coerce_to_f64(activation)? as i32;

            let min_x = x.max(0) as u32;
            let end_x = (x + width) as u32;
            let min_y = y.max(0) as u32;
            let end_y = (y + height) as u32;

            if let Some(color_transform) = color_transform.as_color_transform_object() {
                let params = color_transform.get_params();
                bitmap_data
                    .bitmap_data()
                    .write(activation.context.gc_context)
                    .color_transform(min_x, min_y, end_x, end_y, &params);
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn get_color_bounds_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            let find_color = args
                .get(2)
                .unwrap_or(&true.into())
                .as_bool(activation.swf_version());

            if let (Some(mask_val), Some(color_val)) = (args.get(0), args.get(1)) {
                let mask = mask_val.coerce_to_i32(activation)?;
                let color = color_val.coerce_to_i32(activation)?;

                let (x, y, w, h) = bitmap_data
                    .bitmap_data()
                    .read()
                    .color_bounds_rect(find_color, mask, color);

                let proto = activation.context.avm1.prototypes().rectangle_constructor;
                let rect =
                    proto.construct(activation, &[x.into(), y.into(), w.into(), h.into()])?;
                return Ok(rect);
            }
        }
    }

    Ok((-1).into())
}

pub fn perlin_noise<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
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

            bitmap_data
                .bitmap_data()
                .write(activation.context.gc_context)
                .perlin_noise(
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

pub fn hit_test<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.hitTest - not yet implemented");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn copy_pixels<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
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

            if let Some(src_bitmap) = source_bitmap.as_bitmap_data_object() {
                if !src_bitmap.disposed() {
                    // dealing with object aliasing...
                    let src_bitmap_clone: BitmapData; // only initialized if source is the same object as self
                    let src_bitmap_data_cell = src_bitmap.bitmap_data();
                    let src_bitmap_gc_ref; // only initialized if source is a different object than self
                    let source_bitmap_ref = // holds the reference to either of the ones above
                        if GcCell::ptr_eq(src_bitmap.bitmap_data(), bitmap_data.bitmap_data()) {
                            src_bitmap_clone = src_bitmap_data_cell.read().clone();
                            &src_bitmap_clone
                        } else {
                            src_bitmap_gc_ref = src_bitmap_data_cell.read();
                            &src_bitmap_gc_ref
                        };

                    if args.len() >= 5 {
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

                        let alpha_bitmap = args
                            .get(3)
                            .unwrap_or(&Value::Undefined)
                            .coerce_to_object(activation);

                        if let Some(alpha_bitmap) = alpha_bitmap.as_bitmap_data_object() {
                            if !alpha_bitmap.disposed() {
                                // dealing with aliasing the same way as for the source
                                let alpha_bitmap_clone: BitmapData;
                                let alpha_bitmap_data_cell = alpha_bitmap.bitmap_data();
                                let alpha_bitmap_gc_ref;
                                let alpha_bitmap_ref = if GcCell::ptr_eq(
                                    alpha_bitmap.bitmap_data(),
                                    bitmap_data.bitmap_data(),
                                ) {
                                    alpha_bitmap_clone = alpha_bitmap_data_cell.read().clone();
                                    &alpha_bitmap_clone
                                } else {
                                    alpha_bitmap_gc_ref = alpha_bitmap_data_cell.read();
                                    &alpha_bitmap_gc_ref
                                };

                                let merge_alpha = if args.len() >= 6 {
                                    args.get(5)
                                        .unwrap_or(&Value::Undefined)
                                        .as_bool(activation.swf_version())
                                } else {
                                    true
                                };

                                bitmap_data
                                    .bitmap_data()
                                    .write(activation.context.gc_context)
                                    .copy_pixels(
                                        source_bitmap_ref,
                                        (src_min_x, src_min_y, src_width, src_height),
                                        (dest_x, dest_y),
                                        Some((alpha_bitmap_ref, (alpha_x, alpha_y), merge_alpha)),
                                    );
                            }
                        }
                    } else {
                        bitmap_data
                            .bitmap_data()
                            .write(activation.context.gc_context)
                            .copy_pixels(
                                source_bitmap_ref,
                                (src_min_x, src_min_y, src_width, src_height),
                                (dest_x, dest_y),
                                None,
                            );
                    }
                }
            }

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn merge<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
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

            if let Some(src_bitmap) = source_bitmap.as_bitmap_data_object() {
                if !src_bitmap.disposed() {
                    // dealing with object aliasing...
                    let src_bitmap_clone: BitmapData; // only initialized if source is the same object as self
                    let src_bitmap_data_cell = src_bitmap.bitmap_data();
                    let src_bitmap_gc_ref; // only initialized if source is a different object than self
                    let source_bitmap_ref = // holds the reference to either of the ones above
                        if GcCell::ptr_eq(src_bitmap.bitmap_data(), bitmap_data.bitmap_data()) {
                            src_bitmap_clone = src_bitmap_data_cell.read().clone();
                            &src_bitmap_clone
                        } else {
                            src_bitmap_gc_ref = src_bitmap_data_cell.read();
                            &src_bitmap_gc_ref
                        };

                    bitmap_data
                        .bitmap_data()
                        .write(activation.context.gc_context)
                        .merge(
                            source_bitmap_ref,
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

pub fn palette_map<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
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

            if let Some(src_bitmap) = source_bitmap.as_bitmap_data_object() {
                if !src_bitmap.disposed() {
                    // dealing with object aliasing...
                    let src_bitmap_data_cell = src_bitmap.bitmap_data();
                    let read;
                    let source: Option<&BitmapData> =
                        if GcCell::ptr_eq(src_bitmap_data_cell, bitmap_data.bitmap_data()) {
                            None
                        } else {
                            read = src_bitmap_data_cell.read();
                            Some(&read)
                        };

                    bitmap_data
                        .bitmap_data()
                        .write(activation.context.gc_context)
                        .palette_map(
                            source,
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

pub fn pixel_dissolve<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.pixelDissolve - not yet implemented");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn scroll<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            let x = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;
            let y = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)?;

            bitmap_data
                .bitmap_data()
                .write(activation.context.gc_context)
                .scroll(x, y);

            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn threshold<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.threshold - not yet implemented");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn compare<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    const EQUIVALENT: i32 = 0;
    const NOT_BITMAP: i32 = -1;
    const BITMAP_DISPOSED: i32 = -2;
    const DIFFERENT_WIDTHS: i32 = -3;
    const DIFFERENT_HEIGHTS: i32 = -4;

    let this_bitmap_data = if let Some(bitmap_data) = this.as_bitmap_data_object() {
        bitmap_data
    } else {
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

    let other_bitmap_data = if let Some(other_bitmap_data) = other.as_bitmap_data_object() {
        other_bitmap_data
    } else {
        // The documentation says that -1 should be returned here, but -2 is actually returned.
        return Ok(BITMAP_DISPOSED.into());
    };

    if other_bitmap_data.disposed() {
        return Ok(BITMAP_DISPOSED.into());
    }

    let this_bitmap_data = this_bitmap_data.bitmap_data();
    let this_bitmap_data = this_bitmap_data.read();
    let other_bitmap_data = other_bitmap_data.bitmap_data();
    let other_bitmap_data = other_bitmap_data.read();

    if this_bitmap_data.width() != other_bitmap_data.width() {
        return Ok(DIFFERENT_WIDTHS.into());
    }

    if this_bitmap_data.height() != other_bitmap_data.height() {
        return Ok(DIFFERENT_HEIGHTS.into());
    }

    match BitmapData::compare(&this_bitmap_data, &other_bitmap_data) {
        Some(bitmap_data) => Ok(BitmapDataObject::with_bitmap_data(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes().bitmap_data),
            bitmap_data,
        )
        .into()),
        None => Ok(EQUIVALENT.into()),
    }
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let bitmap_data_object = BitmapDataObject::empty_object(gc_context, Some(proto));
    let object = bitmap_data_object.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    bitmap_data_object.into()
}

pub fn load_bitmap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    let library = &*activation.context.library;

    let movie = activation.target_clip_or_root().movie();

    let renderer = &mut activation.context.renderer;

    let character = movie
        .and_then(|m| library.library_for_movie(m))
        .and_then(|l| l.character_by_export_name(name));

    if let Some(Character::Bitmap(bitmap_object)) = character {
        if let Some(bitmap_handle) = bitmap_object.bitmap_handle() {
            if let Some(bitmap) = renderer.get_bitmap_pixels(bitmap_handle) {
                let new_bitmap_data = BitmapDataObject::empty_object(
                    activation.context.gc_context,
                    Some(activation.context.avm1.prototypes().bitmap_data),
                );

                let width = bitmap.width();
                let height = bitmap.height();
                let pixels: Vec<i32> = bitmap.into();
                new_bitmap_data
                    .as_bitmap_data_object()
                    .unwrap()
                    .bitmap_data()
                    .write(activation.context.gc_context)
                    .set_pixels(
                        width,
                        height,
                        true,
                        pixels.into_iter().map(|p| p.into()).collect(),
                    );

                return Ok(new_bitmap_data.into());
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn create_bitmap_data_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    bitmap_data_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let bitmap_data = FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        constructor_to_fn!(constructor),
        Some(fn_proto),
        bitmap_data_proto,
    );
    let object = bitmap_data.as_script_object().unwrap();
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    bitmap_data
}
