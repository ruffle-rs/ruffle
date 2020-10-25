//! flash.display.BitmapData object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::bitmap_data::{BitmapDataObject, Color};
use crate::avm1::{Object, TObject, Value};
use crate::character::Character;
use enumset::EnumSet;
use gc_arena::MutationContext;
use rand::Rng;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    //TODO: if either width or height is missing then the constructor should fail

    let width = args
        .get(0)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_u32(activation)?;

    let height = args
        .get(1)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_u32(activation)?;

    log::warn!("New bitmap data {}x{}", width, height);

    if width > 2880 || height > 2880 || width <= 0 || height <= 0 {
        log::warn!("Invalid BitmapData size {}x{}", width, height);
        return Ok(Value::Undefined);
    }

    let transparency = args
        .get(2)
        .unwrap_or(&Value::Bool(true))
        .as_bool(activation.current_swf_version());

    let fill_color = args
        .get(3)
        // can't write this in hex
        // 0xFFFFFFFF as f64;
        .unwrap_or(&Value::Number(4294967295_f64))
        .coerce_to_i32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.init_pixels(activation.context.gc_context, width, height, fill_color);
    bitmap_data.set_transparency(activation.context.gc_context, transparency);

    Ok(Value::Undefined)
}

pub fn get_height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    Ok(this.as_bitmap_data_object().unwrap().get_height().into())
}

pub fn get_width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    Ok(this.as_bitmap_data_object().unwrap().get_width().into())
}

pub fn get_transparent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    Ok(this
        .as_bitmap_data_object()
        .unwrap()
        .get_transparency()
        .into())
}

pub fn get_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let proto = activation.context.system_prototypes.rectangle_constructor;
    let rect = proto.construct(
        activation,
        &[
            0.into(),
            0.into(),
            bitmap_data.get_width().into(),
            bitmap_data.get_height().into(),
        ],
    )?;
    Ok(rect.into())
}

pub fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let x = args.get(0).and_then(|x| x.coerce_to_i32(activation).ok());
    let y = args.get(1).and_then(|x| x.coerce_to_i32(activation).ok());

    if let Some((x, y)) = x.zip(y) {
        Ok(bitmap_data.get_pixel(x, y).into())
    } else {
        Ok((-1).into())
    }
}

pub fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let x = args.get(0).and_then(|x| x.coerce_to_i32(activation).ok());
    let y = args.get(1).and_then(|x| x.coerce_to_i32(activation).ok());

    if let Some((x, y)) = x.zip(y) {
        let asdf: i32 = bitmap_data.get_pixel32(x, y).into();
        Ok(asdf.into())
    } else {
        Ok((-1).into())
    }
}

pub fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let x = args.get(0).and_then(|v| v.coerce_to_u32(activation).ok());

    let y = args.get(1).and_then(|v| v.coerce_to_u32(activation).ok());

    let color = args.get(2).and_then(|v| v.coerce_to_i32(activation).ok());

    if let Some(((x, y), color)) = x.zip(y).zip(color) {
        bitmap_data.set_pixel(activation.context.gc_context, x, y, color.into());
    }

    Ok(Value::Undefined)
}

pub fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let x = args.get(0).and_then(|v| v.coerce_to_i32(activation).ok());

    let y = args.get(1).and_then(|v| v.coerce_to_i32(activation).ok());

    let color = args.get(2).and_then(|v| v.coerce_to_i32(activation).ok());

    if let Some(((x, y), color)) = x.zip(y).zip(color) {
        bitmap_data.set_pixel32(activation.context.gc_context, x, y, color.into());
    }

    Ok(Value::Undefined)
}

//TODO: missing args / out of bounds
pub fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("copy channel not fully implemented");
    let source_bitmap = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        //TODO: unwrap
        .coerce_to_object(activation);

    let source_rect = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        //TODO: unwrap
        .coerce_to_object(activation);

    let dest_point = args
        .get(2)
        .unwrap_or(&Value::Undefined)
        //TODO: unwrap
        .coerce_to_object(activation);

    let source_channel = args
        .get(3)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    let dest_channel = args
        .get(4)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    if let Some(source_bitmap) = source_bitmap.as_bitmap_data_object() {
        bitmap_data.copy_channel(
            activation.context.gc_context,
            source_bitmap,
            source_channel as u8,
            dest_channel as u8,
        );
    }

    Ok(Value::Undefined)
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

    let color = args.get(1).and_then(|v| v.coerce_to_i32(activation).ok());

    if let Some(color) = color {
        let x = rectangle.get("x", activation)?.coerce_to_u32(activation)?;
        let y = rectangle.get("y", activation)?.coerce_to_u32(activation)?;
        let width = rectangle
            .get("width", activation)?
            .coerce_to_u32(activation)?;
        let height = rectangle
            .get("height", activation)?
            .coerce_to_u32(activation)?;

        let bitmap_data = this.as_bitmap_data_object().unwrap();
        bitmap_data.fill_rect(
            activation.context.gc_context,
            x,
            y,
            width,
            height,
            color.into(),
        );
    }

    Ok(Value::Undefined)
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if bitmap_data.get_disposed() {
            return Ok((-1).into());
        }

        let proto = activation.context.system_prototypes.bitmap_data_constructor;
        let new_bitmap_data = proto.construct(
            activation,
            &[
                bitmap_data.get_width().into(),
                bitmap_data.get_height().into(),
                bitmap_data.get_transparency().into(),
                0xFFFFFF.into(),
            ],
        )?;
        let new_bitmap_data_object = new_bitmap_data.as_bitmap_data_object().unwrap();

        new_bitmap_data_object.set_pixels(activation.context.gc_context, bitmap_data.get_pixels());

        Ok(new_bitmap_data.into())
    } else {
        Ok((-1).into())
    }
}

pub fn dispose<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    bitmap_data.dispose(activation.context.gc_context);
    Ok(Value::Undefined)
}

pub fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let x = args.get(0).and_then(|v| v.coerce_to_u32(activation).ok());

    let y = args.get(1).and_then(|v| v.coerce_to_u32(activation).ok());

    let color = args.get(2).and_then(|v| v.coerce_to_i32(activation).ok());

    if let Some(((x, y), color)) = x.zip(y).zip(color) {
        let mut pending = Vec::new();
        pending.push((x, y));

        let color: Color = color.into();
        let color: Color = color.to_premultiplied_alpha(bitmap_data.get_transparency());

        let width = bitmap_data.get_width();
        let height = bitmap_data.get_height();

        let expected_color = bitmap_data.get_pixel_raw(x, y).unwrap_or(0.into());

        while !pending.is_empty() {
            if let Some((x, y)) = pending.pop() {
                if let Some(old_color) = bitmap_data.get_pixel_raw(x, y) {
                    if old_color == expected_color {
                        if x > 0 {
                            pending.push((x - 1, y));
                        }
                        if y > 0 {
                            pending.push((x, y - 1));
                        }
                        if x < width - 1 {
                            pending.push((x + 1, y))
                        }
                        if y < height - 1 {
                            pending.push((x, y + 1));
                        }
                        bitmap_data.set_pixel32_raw(activation.context.gc_context, x, y, color);
                    }
                }
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn noise<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let random_seed = args.get(0).and_then(|v| v.coerce_to_u32(activation).ok());

    if let Some(_random_seed) = random_seed {
        let low = args
            .get(1)
            .unwrap_or(&Value::Number(0.0))
            .coerce_to_u32(activation)? as u8;

        let high = args
            .get(2)
            .unwrap_or(&Value::Number(255.0))
            .coerce_to_u32(activation)? as u8;

        let channel_options = args
            .get(3)
            .unwrap_or(&Value::Number((1 | 2 | 4) as f64))
            .coerce_to_u32(activation)?;

        let gray_scale = args
            .get(4)
            .unwrap_or(&Value::Bool(false))
            .as_bool(activation.current_swf_version());

        let width = bitmap_data.get_width();
        let height = bitmap_data.get_height();
        for x in 0..width {
            for y in 0..height {
                let pixel_color = if gray_scale {
                    let gray = activation.context.rng.gen_range(low, high);
                    Color::argb(
                        if channel_options & 8 == 8 {
                            activation.context.rng.gen_range(low, high)
                        } else {
                            255
                        },
                        gray,
                        gray,
                        gray,
                    )
                } else {
                    Color::argb(
                        if channel_options & 8 == 8 {
                            activation.context.rng.gen_range(low, high)
                        } else {
                            255
                        },
                        if channel_options & 1 == 1 {
                            activation.context.rng.gen_range(low, high)
                        } else {
                            0
                        },
                        if channel_options & 2 == 2 {
                            activation.context.rng.gen_range(low, high)
                        } else {
                            0
                        },
                        if channel_options & 4 == 4 {
                            activation.context.rng.gen_range(low, high)
                        } else {
                            0
                        },
                    )
                };

                bitmap_data.set_pixel32_raw(activation.context.gc_context, x, y, pixel_color);
            }
        }
    }

    Ok(Value::Undefined)
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
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.draw - not yet implemented");
    Ok(Value::Undefined)
}

pub fn generate_filter_rect<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.generateFilterRect - not yet implemented");
    Ok(Value::Undefined)
}

pub fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let rectangle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let color_transform = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation);

    let x = rectangle.get("x", activation)?.coerce_to_i32(activation)?;
    let y = rectangle.get("y", activation)?.coerce_to_i32(activation)?;
    let width = rectangle
        .get("width", activation)?
        .coerce_to_i32(activation)?;
    let height = rectangle
        .get("height", activation)?
        .coerce_to_i32(activation)?;

    //TODO: casting
    if let Some(color_transform) = color_transform.as_color_transform_object() {
        bitmap_data.color_transform(
            activation.context.gc_context,
            x as u32,
            y as u32,
            (x + width) as u32,
            (y + height) as u32,
            color_transform.get_alpha_multiplier() as f32,
            color_transform.get_alpha_offset() as f32,
            color_transform.get_red_multiplier() as f32,
            color_transform.get_red_offset() as f32,
            color_transform.get_green_multiplier() as f32,
            color_transform.get_green_offset() as f32,
            color_transform.get_blue_multiplier() as f32,
            color_transform.get_blue_offset() as f32,
        );
    }

    Ok(Value::Undefined)
}

pub fn get_color_bounds_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    let mask = args.get(0).and_then(|v| v.coerce_to_i32(activation).ok());

    let color = args.get(1).and_then(|v| v.coerce_to_i32(activation).ok());

    let find_color = args
        .get(2)
        .unwrap_or(&Value::Bool(true))
        .as_bool(activation.current_swf_version());

    if let Some((mask, color)) = mask.zip(color) {
        let (x, y, w, h) = bitmap_data.get_color_bounds_rect(mask, color, find_color);

        let proto = activation.context.system_prototypes.rectangle_constructor;
        let rect = proto.construct(activation, &[x.into(), y.into(), w.into(), h.into()])?;
        Ok(rect.into())
    } else {
        Ok((-1).into())
    }
}

pub fn perlin_noise<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.perlinNoise - not yet implemented");
    Ok(Value::Undefined)
}

pub fn hit_test<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.hitTest - not yet implemented");
    Ok(Value::Undefined)
}

pub fn copy_pixels<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.copyPixels - not yet implemented");
    Ok(Value::Undefined)
}

pub fn merge<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.merge - not yet implemented");
    Ok(Value::Undefined)
}

pub fn palette_map<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.paletteMap - not yet implemented");
    Ok(Value::Undefined)
}

pub fn pixel_dissolve<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.pixelDissolve - not yet implemented");
    Ok(Value::Undefined)
}

pub fn scroll<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.scroll - not yet implemented");
    Ok(Value::Undefined)
}

pub fn threshold<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into());
    }

    log::warn!("BitmapData.threshold - not yet implemented");
    Ok(Value::Undefined)
}

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let bitmap_data_object = BitmapDataObject::empty_object(gc_context, Some(proto));
    let mut object = bitmap_data_object.as_script_object().unwrap();

    object.add_property(
        gc_context,
        "height",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_height),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "width",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_width),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "transparent",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_transparent),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        EnumSet::empty(),
    );

    object.add_property(
        gc_context,
        "rectangle",
        FunctionObject::function(
            gc_context,
            Executable::Native(get_rectangle),
            Some(fn_proto),
            fn_proto,
        ),
        None,
        EnumSet::empty(),
    );

    object.force_set_function(
        "getPixel",
        get_pixel,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "getPixel32",
        get_pixel32,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "setPixel",
        set_pixel,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "setPixel32",
        set_pixel32,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "copyChannel",
        copy_channel,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "fillRect",
        fill_rect,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function("clone", clone, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function(
        "dispose",
        dispose,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "floodFill",
        flood_fill,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function("noise", noise, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function(
        "colorTransform",
        color_transform,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "getColorBoundsRect",
        get_color_bounds_rect,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "perlinNoise",
        perlin_noise,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "applyFilter",
        apply_filter,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function("draw", draw, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function(
        "hitTest",
        hit_test,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "generateFilterRect",
        generate_filter_rect,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "copyPixels",
        copy_pixels,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function("merge", merge, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function(
        "paletteMap",
        palette_map,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "pixelDissolve",
        pixel_dissolve,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "scroll",
        scroll,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "threshold",
        threshold,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    bitmap_data_object.into()
}

//todo
use crate::display_object::TDisplayObject;

pub fn load_bitmap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    log::warn!("BitmapData.loadBitmap({:?}), not yet implemented", name);

    let swf = activation
        .target_clip_or_root()
        .movie()
        .expect("No movie ?");

    let bh = activation
        .context
        .library
        .library_for_movie(swf)
        .expect("No library for movie")
        .get_character_by_export_name(name.as_str())
        .expect("No character for name");

    let bitmap = match bh {
        Character::Bitmap(b) => b.bitmap_handle(),
        _ => unimplemented!(),
    };
    //TODO: also return bounds?
    let (w, h, pixels) = activation.context.renderer.get_bitmap_pixels(bitmap);
    log::warn!("Got response {} {} {:?}", w, h, pixels);

    let proto = activation.context.system_prototypes.bitmap_data_constructor;
    let new_bitmap = proto.construct(activation, &[w.into(), h.into()])?;
    let new_bitmap_object = new_bitmap.as_bitmap_data_object().unwrap();

    //todo: set w/h
    new_bitmap_object.set_pixels(
        activation.context.gc_context,
        pixels.iter().map(|p| (*p as i32).into()).collect(),
    );

    Ok(new_bitmap.into())
}

pub fn create_bitmap_data_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    bitmap_data_proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let object = FunctionObject::constructor(
        gc_context,
        Executable::Native(constructor),
        fn_proto,
        bitmap_data_proto,
    );
    let mut script_object = object.as_script_object().unwrap();

    script_object.force_set_function(
        "loadBitmap",
        load_bitmap,
        gc_context,
        EnumSet::empty(),
        fn_proto,
    );

    object
}
