//! flash.display.BitmapData object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::bitmap_data::{BitmapDataObject, Color};
use crate::avm1::{Object, TObject, Value};
use crate::character::Character;
use crate::display_object::TDisplayObject;
use enumset::EnumSet;
use gc_arena::MutationContext;
use rand::Rng;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let width = args
        .get(0)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_i32(activation)?;

    let height = args
        .get(1)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_i32(activation)?;

    if width > 2880 || height > 2880 || width <= 0 || height <= 0 {
        log::warn!("Invalid BitmapData size {}x{}", width, height);
        return Err(Error::ConstructorFailure);
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

    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        bitmap_data.init_pixels(
            activation.context.gc_context,
            width as u32,
            height as u32,
            fill_color,
        );
        bitmap_data.set_transparency(activation.context.gc_context, transparency);
    }

    Ok(Value::Undefined)
}

pub fn height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            return Ok(bitmap_data.height().into());
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
            return Ok(bitmap_data.width().into());
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
            return Ok(bitmap_data.transparency().into());
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
            let proto = activation.context.system_prototypes.rectangle_constructor;
            let rect = proto.construct(
                activation,
                &[
                    0.into(),
                    0.into(),
                    bitmap_data.width().into(),
                    bitmap_data.height().into(),
                ],
            )?;
            return Ok(rect.into());
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
                return Ok(bitmap_data.get_pixel(x, y).into());
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
                let col: i32 = bitmap_data.get_pixel32(x, y).into();
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

                bitmap_data.set_pixel(activation.context.gc_context, x, y, color.into());

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

                bitmap_data.set_pixel32(activation.context.gc_context, x, y, color.into());
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
                    .min(bitmap_data.width());
                let min_y = dest_point
                    .get("y", activation)?
                    .coerce_to_u32(activation)?
                    .min(bitmap_data.height());

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

                for x in src_min_x.max(0)..src_max_x.min(source_bitmap.width()) {
                    for y in src_min_y.max(0)..src_max_y.min(source_bitmap.height()) {
                        if bitmap_data.is_point_in_bounds((x + min_x) as i32, (y + min_y) as i32) {
                            let original_color: u32 = bitmap_data
                                .get_pixel_raw((x + min_x) as u32, (y + min_y) as u32)
                                .unwrap_or_else(|| 0.into())
                                .into();
                            let source_color: u32 = source_bitmap
                                .get_pixel_raw(x, y)
                                .unwrap_or_else(|| 0.into())
                                .into();

                            let channel_shift: u32 = match source_channel {
                                // Alpha
                                8 => 24,
                                // red
                                1 => 16,
                                // green
                                2 => 8,
                                // blue
                                4 => 0,
                                _ => 0,
                            };

                            let source_part = (source_color >> channel_shift) & 0xFF;

                            let result_color: u32 = match dest_channel {
                                // Alpha
                                8 => (original_color & 0x00FFFFFF) | source_part << 24,
                                // red
                                1 => (original_color & 0xFF00FFFF) | source_part << 16,
                                // green
                                2 => (original_color & 0xFFFF00FF) | source_part << 8,
                                // blue
                                4 => (original_color & 0xFFFFFF00) | source_part,
                                _ => original_color,
                            };

                            bitmap_data.set_pixel32_raw(
                                activation.context.gc_context,
                                (x + min_x) as u32,
                                (y + min_y) as u32,
                                (result_color as i32).into(),
                            );
                        }
                    }
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

                for x_offset in 0..width {
                    for y_offset in 0..height {
                        bitmap_data.set_pixel32(
                            activation.context.gc_context,
                            (x + x_offset) as i32,
                            (y + y_offset) as i32,
                            color.into(),
                        )
                    }
                }
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
            let proto = activation.context.system_prototypes.bitmap_data_constructor;
            let new_bitmap_data = proto.construct(
                activation,
                &[
                    bitmap_data.width().into(),
                    bitmap_data.height().into(),
                    bitmap_data.transparency().into(),
                    0xFFFFFF.into(),
                ],
            )?;
            let new_bitmap_data_object = new_bitmap_data.as_bitmap_data_object().unwrap();

            new_bitmap_data_object.set_pixels(
                activation.context.gc_context,
                bitmap_data.get_pixels().to_vec(),
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
            bitmap_data.dispose(activation.context.gc_context);
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

                let mut pending = Vec::new();
                pending.push((x, y));

                let color: Color = color.into();
                let color: Color = color.to_premultiplied_alpha(bitmap_data.transparency());

                let width = bitmap_data.width();
                let height = bitmap_data.height();

                let expected_color = bitmap_data.get_pixel_raw(x, y).unwrap_or_else(|| 0.into());

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
                                bitmap_data.set_pixel32_raw(
                                    activation.context.gc_context,
                                    x,
                                    y,
                                    color,
                                );
                            }
                        }
                    }
                }
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

    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            if let Some(random_seed_val) = args.get(0) {
                let _random_seed = random_seed_val.coerce_to_u32(activation)?;

                let width = bitmap_data.width();
                let height = bitmap_data.height();
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

                        bitmap_data.set_pixel32_raw(
                            activation.context.gc_context,
                            x,
                            y,
                            pixel_color,
                        );
                    }
                }
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

            let x = rectangle.get("x", activation)?.coerce_to_i32(activation)?;
            let y = rectangle.get("y", activation)?.coerce_to_i32(activation)?;
            let width = rectangle
                .get("width", activation)?
                .coerce_to_i32(activation)?;
            let height = rectangle
                .get("height", activation)?
                .coerce_to_i32(activation)?;

            let min_x = x.max(0) as u32;
            let end_x = (x + width) as u32;
            let min_y = y.max(0) as u32;
            let end_y = (y + height) as u32;

            if let Some(color_transform) = color_transform.as_color_transform_object() {
                for x in min_x..end_x.min(bitmap_data.width()) {
                    for y in min_y..end_y.min(bitmap_data.height()) {
                        let color = bitmap_data
                            .get_pixel_raw(x, y)
                            .unwrap_or_else(|| 0.into())
                            .to_un_multiplied_alpha();

                        let alpha = ((color.alpha() as f32
                            * color_transform.get_alpha_multiplier() as f32)
                            + color_transform.get_alpha_offset() as f32)
                            as u8;
                        let red = ((color.red() as f32
                            * color_transform.get_red_multiplier() as f32)
                            + color_transform.get_red_offset() as f32)
                            as u8;
                        let green = ((color.green() as f32
                            * color_transform.get_green_multiplier() as f32)
                            + color_transform.get_green_offset() as f32)
                            as u8;
                        let blue = ((color.blue() as f32
                            * color_transform.get_blue_multiplier() as f32)
                            + color_transform.get_blue_offset() as f32)
                            as u8;

                        bitmap_data.set_pixel32_raw(
                            activation.context.gc_context,
                            x,
                            y,
                            Color::argb(alpha, red, green, blue)
                                .to_premultiplied_alpha(bitmap_data.transparency()),
                        )
                    }
                }
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
                .unwrap_or(&Value::Bool(true))
                .as_bool(activation.current_swf_version());

            if let (Some(mask_val), Some(color_val)) = (args.get(0), args.get(1)) {
                let mask = mask_val.coerce_to_i32(activation)?;
                let color = color_val.coerce_to_i32(activation)?;

                let mut min_x = Option::<i32>::None;
                let mut max_x = Option::<i32>::None;
                let mut min_y = Option::<i32>::None;
                let mut max_y = Option::<i32>::None;

                for x in 0..bitmap_data.width() {
                    for y in 0..bitmap_data.height() {
                        let pixel_raw: i32 = bitmap_data
                            .get_pixel_raw(x, y)
                            .unwrap_or_else(|| 0.into())
                            .into();
                        let color_matches = if find_color {
                            (pixel_raw & mask) == color
                        } else {
                            (pixel_raw & mask) != color
                        };

                        if color_matches {
                            if (x as i32) < min_x.unwrap_or(bitmap_data.width() as i32) {
                                min_x = Some(x as i32)
                            }
                            if (x as i32) > max_x.unwrap_or(-1) {
                                max_x = Some(x as i32 + 1)
                            }

                            if (y as i32) < min_y.unwrap_or(bitmap_data.height() as i32) {
                                min_y = Some(y as i32)
                            }
                            if (y as i32) > max_y.unwrap_or(-1) {
                                max_y = Some(y as i32 + 1)
                            }
                        }
                    }
                }

                let min_x = min_x.unwrap_or(0);
                let min_y = min_y.unwrap_or(0);
                let max_x = max_x.unwrap_or(0);
                let max_y = max_y.unwrap_or(0);

                let x = min_x as u32;
                let y = min_y as u32;
                let w = (max_x - min_x) as u32;
                let h = (max_y - min_y) as u32;

                let proto = activation.context.system_prototypes.rectangle_constructor;
                let rect =
                    proto.construct(activation, &[x.into(), y.into(), w.into(), h.into()])?;
                return Ok(rect.into());
            }
        }
    }

    Ok((-1).into())
}

pub fn perlin_noise<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.perlinNoise - not yet implemented");
            return Ok(Value::Undefined);
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
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.copyPixels - not yet implemented");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn merge<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.merge - not yet implemented");
            return Ok(Value::Undefined);
        }
    }

    Ok((-1).into())
}

pub fn palette_map<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.paletteMap - not yet implemented");
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
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
        if !bitmap_data.disposed() {
            log::warn!("BitmapData.scroll - not yet implemented");
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
            Executable::Native(height),
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
            Executable::Native(width),
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
        .and_then(|l| l.get_character_by_export_name(name.as_str()));

    if let Some(Character::Bitmap(bitmap_object)) = character {
        if let Some(bitmap) = renderer.get_bitmap_pixels(bitmap_object.bitmap_handle()) {
            let proto = activation.context.system_prototypes.bitmap_data_constructor;
            let new_bitmap =
                proto.construct(activation, &[bitmap.width.into(), bitmap.height.into()])?;
            let new_bitmap_object = new_bitmap.as_bitmap_data_object().unwrap();

            let pixels: Vec<i32> = bitmap.data.into();

            new_bitmap_object.set_pixels(
                activation.context.gc_context,
                pixels.into_iter().map(|p| p.into()).collect(),
            );

            return Ok(new_bitmap.into());
        }
    }

    Ok(Value::Undefined)
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
