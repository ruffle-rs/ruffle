//! `flash.display.BitmapData` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::class::{Class, ClassAttributes};
use crate::avm2::method::{Method, NativeMethodImpl};
use crate::avm2::object::{bitmapdata_allocator, BitmapDataObject, Object, TObject};
use crate::avm2::value::Value;
use crate::avm2::Error;
use crate::avm2::Multiname;
use crate::avm2::Namespace;
use crate::avm2::QName;
use crate::bitmap::bitmap_data::IBitmapDrawable;
use crate::bitmap::bitmap_data::{BitmapData, ChannelOptions, Color};
use crate::bitmap::is_size_valid;
use crate::character::Character;
use crate::display_object::Bitmap;
use crate::swf::BlendMode;
use gc_arena::{GcCell, MutationContext};
use ruffle_render::transform::Transform;
use std::str::FromStr;

/// Copy the static data from a given Bitmap into a new BitmapData.
///
/// `bd` is assumed to be an uninstantiated library symbol, associated with the
/// class named by `name`.
pub fn fill_bitmap_data_from_symbol<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    bd: Bitmap<'gc>,
    new_bitmap_data: GcCell<'gc, BitmapData<'gc>>,
    pixels: Vec<i32>,
) {
    new_bitmap_data
        .write(activation.context.gc_context)
        .set_pixels(
            bd.width().into(),
            bd.height().into(),
            true,
            pixels.into_iter().map(|p| p.into()).collect(),
        );
}

/// Implements `flash.display.BitmapData`'s instance constructor.
pub fn instance_init<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(this) = this {
        activation.super_init(this, &[])?;

        if this.as_bitmap_data().is_none() {
            let name = this.instance_of_class_definition().map(|c| c.read().name());
            let character = this
                .instance_of()
                .and_then(|t| {
                    activation
                        .context
                        .library
                        .avm2_class_registry()
                        .class_symbol(t)
                })
                .and_then(|(movie, chara_id)| {
                    activation
                        .context
                        .library
                        .library_for_movie_mut(movie)
                        .character_by_id(chara_id)
                        .cloned()
                });

            let new_bitmap_data =
                GcCell::allocate(activation.context.gc_context, BitmapData::default());

            if let Some(Character::Bitmap {
                bitmap,
                initial_data,
            }) = character
            {
                // Instantiating BitmapData from an Animate-style bitmap asset
                fill_bitmap_data_from_symbol(activation, bitmap, new_bitmap_data, initial_data);
            } else {
                if character.is_some() {
                    //TODO: Determine if mismatched symbols will still work as a
                    //regular BitmapData subclass, or if this should throw
                    log::warn!(
                        "BitmapData subclass {:?} is associated with a non-bitmap symbol",
                        name
                    );
                }

                let width = args
                    .get(0)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_i32(activation)? as u32;
                let height = args
                    .get(1)
                    .unwrap_or(&Value::Undefined)
                    .coerce_to_i32(activation)? as u32;
                let transparency = args
                    .get(2)
                    .unwrap_or(&Value::Bool(true))
                    .coerce_to_boolean();
                let fill_color = if let Some(value) = args.get(3) {
                    value.coerce_to_u32(activation)?
                } else {
                    0xFFFFFFFF
                };

                if !is_size_valid(activation.context.swf.version(), width, height) {
                    return Err("Bitmap size is not valid".into());
                }

                new_bitmap_data
                    .write(activation.context.gc_context)
                    .init_pixels(width, height, transparency, fill_color as i32);
            }

            new_bitmap_data
                .write(activation.context.gc_context)
                .init_object2(this);
            this.init_bitmap_data(activation.context.gc_context, new_bitmap_data);
        }
    }

    Ok(Value::Undefined)
}

/// Implements `flash.display.BitmapData`'s class constructor.
pub fn class_init<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(Value::Undefined)
}

/// Implements `BitmapData.width`'s getter.
pub fn width<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        return Ok((bitmap_data.read().width() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.height`'s getter.
pub fn height<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        return Ok((bitmap_data.read().height() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.transparent`'s getter.
pub fn transparent<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        return Ok(bitmap_data.read().transparency().into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.scroll`.
pub fn scroll<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let x = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let y = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;

        bitmap_data
            .write(activation.context.gc_context)
            .scroll(x, y);
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.copyPixels`.
pub fn copy_pixels<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let source_bitmap = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let source_rect = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let src_min_x = source_rect
            .get_property(&Multiname::public("x"), activation)?
            .coerce_to_i32(activation)?;
        let src_min_y = source_rect
            .get_property(&Multiname::public("y"), activation)?
            .coerce_to_i32(activation)?;
        let src_width = source_rect
            .get_property(&Multiname::public("width"), activation)?
            .coerce_to_i32(activation)?;
        let src_height = source_rect
            .get_property(&Multiname::public("height"), activation)?
            .coerce_to_i32(activation)?;

        let dest_point = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let dest_x = dest_point
            .get_property(&Multiname::public("x"), activation)?
            .coerce_to_i32(activation)?;
        let dest_y = dest_point
            .get_property(&Multiname::public("y"), activation)?
            .coerce_to_i32(activation)?;

        if let Some(src_bitmap) = source_bitmap.as_bitmap_data() {
            src_bitmap.read().check_valid(activation)?;
            // dealing with object aliasing...
            let src_bitmap_clone: BitmapData; // only initialized if source is the same object as self
            let src_bitmap_data_cell = src_bitmap;
            let src_bitmap_gc_ref; // only initialized if source is a different object than self
            let source_bitmap_ref = // holds the reference to either of the ones above
                if GcCell::ptr_eq(src_bitmap, bitmap_data) {
                    src_bitmap_clone = src_bitmap_data_cell.read().clone();
                    &src_bitmap_clone
                } else {
                    src_bitmap_gc_ref = src_bitmap_data_cell.read();
                    &src_bitmap_gc_ref
                };

            let mut alpha_source = None;

            if args.len() >= 4 {
                if let Some(alpha_bitmap) = args
                    .get(3)
                    .and_then(|o| o.as_object())
                    .and_then(|o| o.as_bitmap_data())
                {
                    // Testing shows that a null/undefined 'alphaPoint' parameter is treated
                    // as 'new Point(0, 0)'
                    let mut x = 0;
                    let mut y = 0;

                    if let Ok(alpha_point) = args
                        .get(4)
                        .unwrap_or(&Value::Undefined)
                        .coerce_to_object(activation)
                    {
                        x = alpha_point
                            .get_property(&Multiname::public("x"), activation)?
                            .coerce_to_i32(activation)?;
                        y = alpha_point
                            .get_property(&Multiname::public("y"), activation)?
                            .coerce_to_i32(activation)?;
                    }

                    alpha_source = Some((alpha_bitmap, (x, y)));
                }
            }

            let merge_alpha = args
                .get(5)
                .unwrap_or(&Value::Bool(false))
                .coerce_to_boolean();

            if let Some((alpha_bitmap, alpha_point)) = alpha_source {
                bitmap_data
                    .write(activation.context.gc_context)
                    .copy_pixels(
                        source_bitmap_ref,
                        (src_min_x, src_min_y, src_width, src_height),
                        (dest_x, dest_y),
                        Some((&*alpha_bitmap.read(), alpha_point)),
                        merge_alpha,
                    );
            } else {
                bitmap_data
                    .write(activation.context.gc_context)
                    .copy_pixels(
                        source_bitmap_ref,
                        (src_min_x, src_min_y, src_width, src_height),
                        (dest_x, dest_y),
                        None,
                        merge_alpha,
                    );
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.getPixel`.
pub fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let x = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let y = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        return Ok((bitmap_data.read().get_pixel(x, y) as u32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.getPixel32`.
pub fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let y = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let pixel = i32::from(bitmap_data.read().get_pixel32(x, y));
        return Ok((pixel as u32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixel`.
pub fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let y = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_u32(activation)?;
        let color = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        bitmap_data
            .write(activation.context.gc_context)
            .set_pixel(x, y, color.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixel32`.
pub fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let y = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        let color = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;
        bitmap_data
            .write(activation.context.gc_context)
            .set_pixel32(x, y, color.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixels`.
pub fn set_pixels<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let bytearray = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = rectangle
            .get_property(&Multiname::public("x"), activation)?
            .coerce_to_u32(activation)?;
        let y = rectangle
            .get_property(&Multiname::public("y"), activation)?
            .coerce_to_u32(activation)?;
        let width = rectangle
            .get_property(&Multiname::public("width"), activation)?
            .coerce_to_u32(activation)?;
        let height = rectangle
            .get_property(&Multiname::public("height"), activation)?
            .coerce_to_u32(activation)?;

        let ba_read = bytearray
            .as_bytearray()
            .ok_or("ArgumentError: Parameter must be a bytearray")?;

        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        let mut ind = 0;

        for y in y..y + height {
            for x in x..x + width {
                // Copy data from bytearray until EOFError or finished
                if let Ok(color) = ba_read.read_int_at(ind) {
                    bitmap_data.set_pixel32(x as i32, y as i32, color.into());
                    ind += 4;
                } else {
                    return Err(Error::AvmError(crate::avm2::error::eof_error(
                        activation,
                        "Error #2030: End of file was encountered.",
                        2030,
                    )?));
                }
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.copyChannel`.
pub fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let source_bitmap = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let source_rect = args
            .get(1)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let dest_point = args
            .get(2)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let dest_x = dest_point
            .get_property(&Multiname::public("x"), activation)?
            .coerce_to_u32(activation)?;
        let dest_y = dest_point
            .get_property(&Multiname::public("y"), activation)?
            .coerce_to_u32(activation)?;

        let source_channel = args
            .get(3)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;

        let dest_channel = args
            .get(4)
            .unwrap_or(&Value::Undefined)
            .coerce_to_i32(activation)?;

        let mut bitmap_data_write = bitmap_data.write(activation.context.gc_context);
        if !bitmap_data_write.disposed() {
            if let Some(source_bitmap) = source_bitmap.as_bitmap_data() {
                //TODO: what if source is disposed
                let src_min_x = source_rect
                    .get_property(&Multiname::public("x"), activation)?
                    .coerce_to_u32(activation)?;
                let src_min_y = source_rect
                    .get_property(&Multiname::public("y"), activation)?
                    .coerce_to_u32(activation)?;
                let src_width = source_rect
                    .get_property(&Multiname::public("width"), activation)?
                    .coerce_to_u32(activation)?;
                let src_height = source_rect
                    .get_property(&Multiname::public("height"), activation)?
                    .coerce_to_u32(activation)?;
                let src_max_x = src_min_x + src_width;
                let src_max_y = src_min_y + src_height;

                if GcCell::ptr_eq(bitmap_data, source_bitmap) {
                    let src_bitmap_data_clone = source_bitmap.read().clone();
                    bitmap_data_write.copy_channel(
                        (dest_x, dest_y),
                        (src_min_x, src_min_y, src_max_x, src_max_y),
                        &src_bitmap_data_clone,
                        source_channel,
                        dest_channel,
                    );
                } else {
                    bitmap_data_write.copy_channel(
                        (dest_x, dest_y),
                        (src_min_x, src_min_y, src_max_x, src_max_y),
                        &source_bitmap.read(),
                        source_channel,
                        dest_channel,
                    );
                }
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        if !bitmap_data.disposed() {
            if let (Some(x_val), Some(y_val), Some(color_val)) =
                (args.get(0), args.get(1), args.get(2))
            {
                let x = x_val.coerce_to_u32(activation)?;
                let y = y_val.coerce_to_u32(activation)?;
                let color = color_val.coerce_to_i32(activation)?;

                let color: Color = color.into();
                let color: Color = color.to_premultiplied_alpha(bitmap_data.transparency());

                bitmap_data.flood_fill(x, y, color);
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn noise<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
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

    let gray_scale = args.get(4).unwrap_or(&false.into()).coerce_to_boolean();

    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        if !bitmap_data.disposed() {
            if let Some(random_seed_val) = args.get(0) {
                let random_seed = random_seed_val.coerce_to_i32(activation)?;
                bitmap_data.noise(random_seed, low, high.max(low), channel_options, gray_scale)
            }
        }
    }
    Ok(Value::Undefined)
}

pub fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        if !bitmap_data.disposed() {
            if let [rectangle, color_transform, ..] = args {
                // TODO: Re-use `object_to_rectangle` in `movie_clip.rs`.
                let rectangle = rectangle.coerce_to_object(activation)?;
                let x = rectangle
                    .get_property(&Multiname::public("x"), activation)?
                    .coerce_to_i32(activation)?;
                let y = rectangle
                    .get_property(&Multiname::public("y"), activation)?
                    .coerce_to_i32(activation)?;
                let width = rectangle
                    .get_property(&Multiname::public("width"), activation)?
                    .coerce_to_i32(activation)?;
                let height = rectangle
                    .get_property(&Multiname::public("height"), activation)?
                    .coerce_to_i32(activation)?;

                let x_min = x.max(0) as u32;
                let x_max = (x + width) as u32;
                let y_min = y.max(0) as u32;
                let y_max = (y + height) as u32;

                let color_transform =
                    crate::avm2::globals::flash::geom::transform::object_to_color_transform(
                        color_transform.coerce_to_object(activation)?,
                        activation,
                    )?;

                bitmap_data.color_transform(x_min, y_min, x_max, y_max, color_transform);
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn get_color_bounds_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let bitmap_data = bitmap_data.read();
        if !bitmap_data.disposed() {
            let find_color = args.get(2).unwrap_or(&true.into()).coerce_to_boolean();

            if let (Some(mask_val), Some(color_val)) = (args.get(0), args.get(1)) {
                let mask = mask_val.coerce_to_i32(activation)?;
                let color = color_val.coerce_to_i32(activation)?;

                let (x, y, w, h) = bitmap_data.color_bounds_rect(find_color, mask, color);

                let rect = activation
                    .avm2()
                    .classes()
                    .rectangle
                    .construct(activation, &[x.into(), y.into(), w.into(), h.into()])?
                    .into();
                return Ok(rect);
            }
        }
    }

    Ok(Value::Undefined)
}

pub fn lock<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("BitmapData.lock - not yet implemented");
    Ok(Value::Undefined)
}

/// Implements `BitmapData.draw`
pub fn draw<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let mut transform = Transform::default();
        let mut blend_mode = BlendMode::Normal;

        let matrix = args.get(1).unwrap_or(&Value::Null);
        if !matches!(matrix, Value::Null) {
            transform.matrix = crate::avm2::globals::flash::geom::transform::object_to_matrix(
                matrix.coerce_to_object(activation)?,
                activation,
            )?;
        }

        let color_transform = args.get(2).unwrap_or(&Value::Null);
        if !matches!(color_transform, Value::Null) {
            transform.color_transform =
                crate::avm2::globals::flash::geom::transform::object_to_color_transform(
                    color_transform.coerce_to_object(activation)?,
                    activation,
                )?;
        }

        let mode = args.get(3).unwrap_or(&Value::Null);
        if !matches!(mode, Value::Null) {
            if let Ok(mode) = BlendMode::from_str(&mode.coerce_to_string(activation)?.to_string()) {
                blend_mode = mode;
            } else {
                log::error!("Unknown blend mode {:?}", mode);
                return Err("ArgumentError: Error #2008: Parameter blendMode must be one of the accepted values.".into());
            }
        }

        if args.get(4).is_some() {
            log::warn!("BitmapData.draw with clip rect - not implemented")
        }

        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        // FIXME - handle other arguments
        let smoothing = args.get(5).unwrap_or(&false.into()).coerce_to_boolean();

        let source = args
            .get(0)
            .and_then(|v| v.as_object())
            .ok_or_else(|| format!("BitmapData.draw: source {:?} is not an Object", args.get(0)))?;

        let source = if let Some(source_object) = source.as_display_object() {
            IBitmapDrawable::DisplayObject(source_object)
        } else if let Some(source_bitmap) = source.as_bitmap_data() {
            IBitmapDrawable::BitmapData(source_bitmap)
        } else {
            return Err(format!("BitmapData.draw: unexpected source {source:?}").into());
        };

        bitmap_data.draw(
            source,
            transform,
            smoothing,
            blend_mode,
            &mut activation.context,
        );
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.fillRect`
pub fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;

    let color = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_u32(activation)? as i32;

    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        let x = rectangle
            .get_property(&Multiname::public("x"), activation)?
            .coerce_to_u32(activation)?;
        let y = rectangle
            .get_property(&Multiname::public("y"), activation)?
            .coerce_to_u32(activation)?;
        let width = rectangle
            .get_property(&Multiname::public("width"), activation)?
            .coerce_to_u32(activation)?;
        let height = rectangle
            .get_property(&Multiname::public("height"), activation)?
            .coerce_to_u32(activation)?;

        bitmap_data.write(activation.context.gc_context).fill_rect(
            x,
            y,
            width,
            height,
            color.into(),
        );
    }
    Ok(Value::Undefined)
}

/// Implements `BitmapData.dispose`
pub fn dispose<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        // Don't check if we've already disposed this BitmapData - 'BitmapData.dispose()' can be called
        // multiple times
        bitmap_data
            .write(activation.context.gc_context)
            .dispose(activation.context.renderer);
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.rect`
pub fn rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        let bd = bitmap_data.read();
        return Ok(activation
            .avm2()
            .classes()
            .rectangle
            .construct(
                activation,
                &[0.into(), 0.into(), bd.width().into(), bd.height().into()],
            )?
            .into());
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.applyFilter`
pub fn apply_filter<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    log::warn!("BitmapData.applyFilter: Not yet implemented");
    Ok(Value::Undefined)
}

/// Implement `BitmapData.clone`
pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        if !bitmap_data.read().disposed() {
            let new_bitmap_data =
                GcCell::allocate(activation.context.gc_context, BitmapData::default());
            new_bitmap_data
                .write(activation.context.gc_context)
                .set_pixels(
                    bitmap_data.read().width(),
                    bitmap_data.read().height(),
                    bitmap_data.read().transparency(),
                    bitmap_data.read().pixels().to_vec(),
                );

            let class = activation.avm2().classes().bitmapdata;
            let new_bitmap_data_object =
                BitmapDataObject::from_bitmap_data(activation, new_bitmap_data, class)?;

            return Ok(new_bitmap_data_object.into());
        }
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.perlinNoise`
pub fn perlin_noise<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        if !bitmap_data.read().disposed() {
            let base_x = args
                .get(0)
                .unwrap_or(&Value::Undefined)
                .coerce_to_number(activation)?;
            let base_y = args
                .get(1)
                .unwrap_or(&Value::Undefined)
                .coerce_to_number(activation)?;
            let num_octaves = args
                .get(2)
                .unwrap_or(&Value::Undefined)
                .coerce_to_u32(activation)? as usize;
            let seed = args
                .get(3)
                .unwrap_or(&Value::Undefined)
                .coerce_to_i32(activation)? as i64;
            let stitch = args.get(4).unwrap_or(&Value::Undefined).coerce_to_boolean();
            let fractal_noise = args.get(5).unwrap_or(&Value::Undefined).coerce_to_boolean();
            let channel_options = if let Some(c) = args.get(6) {
                ChannelOptions::from_bits_truncate(c.coerce_to_i32(activation)? as u8)
            } else {
                ChannelOptions::RGB
            };
            let grayscale = args.get(7).unwrap_or(&Value::Undefined).coerce_to_boolean();
            let offsets = args
                .get(8)
                .unwrap_or(&Value::Undefined)
                .coerce_to_object(activation);

            let octave_offsets: Result<Vec<_>, Error<'gc>> = (0..num_octaves)
                .map(|i| {
                    if let Ok(offsets) = offsets {
                        if let Some(offsets) = offsets.as_array_storage() {
                            if let Some(Value::Object(e)) = offsets.get(i) {
                                let x = e
                                    .get_property(&Multiname::public("x"), activation)?
                                    .coerce_to_number(activation)?;
                                let y = e
                                    .get_property(&Multiname::public("y"), activation)?
                                    .coerce_to_number(activation)?;
                                Ok((x, y))
                            } else {
                                Ok((0.0, 0.0))
                            }
                        } else {
                            Ok((0.0, 0.0))
                        }
                    } else {
                        Ok((0.0, 0.0))
                    }
                })
                .collect();
            let octave_offsets = octave_offsets?;

            bitmap_data
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

    Ok(Value::Undefined)
}

/// Construct `BitmapData`'s class.
pub fn create_class<'gc>(mc: MutationContext<'gc, '_>) -> GcCell<'gc, Class<'gc>> {
    let class = Class::new(
        QName::new(Namespace::package("flash.display"), "BitmapData"),
        Some(Multiname::new(Namespace::package(""), "Object")),
        Method::from_builtin(instance_init, "<BitmapData instance initializer>", mc),
        Method::from_builtin(class_init, "<BitmapData class initializer>", mc),
        mc,
    );

    let mut write = class.write(mc);

    write.set_attributes(ClassAttributes::SEALED);
    write.set_instance_allocator(bitmapdata_allocator);

    write.implements(Multiname::new(
        Namespace::package("flash.display"),
        "IBitmapDrawable",
    ));

    const PUBLIC_INSTANCE_PROPERTIES: &[(
        &str,
        Option<NativeMethodImpl>,
        Option<NativeMethodImpl>,
    )] = &[
        ("width", Some(width), None),
        ("height", Some(height), None),
        ("rect", Some(rect), None),
        ("transparent", Some(transparent), None),
    ];
    write.define_public_builtin_instance_properties(mc, PUBLIC_INSTANCE_PROPERTIES);

    const PUBLIC_INSTANCE_METHODS: &[(&str, NativeMethodImpl)] = &[
        ("getPixel", get_pixel),
        ("getPixel32", get_pixel32),
        ("setPixel", set_pixel),
        ("setPixel32", set_pixel32),
        ("setPixels", set_pixels),
        ("copyChannel", copy_channel),
        ("floodFill", flood_fill),
        ("noise", noise),
        ("colorTransform", color_transform),
        ("getColorBoundsRect", get_color_bounds_rect),
        ("scroll", scroll),
        ("lock", lock),
        ("unlock", lock), // sic, it's a noop (TODO)
        ("copyPixels", copy_pixels),
        ("draw", draw),
        ("fillRect", fill_rect),
        ("dispose", dispose),
        ("applyFilter", apply_filter),
        ("clone", clone),
        ("perlinNoise", perlin_noise),
    ];
    write.define_public_builtin_instance_methods(mc, PUBLIC_INSTANCE_METHODS);

    class
}
