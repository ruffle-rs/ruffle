//! `flash.display.BitmapData` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::error::argument_error;
use crate::avm2::filters::FilterAvm2Ext;
use crate::avm2::object::{BitmapDataObject, ByteArrayObject, Object, TObject, VectorObject};
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::avm2_stub_method;
use crate::bitmap::bitmap_data::{BitmapData, ChannelOptions, Color};
use crate::bitmap::bitmap_data::{BitmapDataDrawError, IBitmapDrawable};
use crate::bitmap::is_size_valid;
use crate::character::Character;
use crate::display_object::Bitmap;
use crate::swf::BlendMode;
use gc_arena::GcCell;
use ruffle_render::filters::Filter;
use ruffle_render::transform::Transform;
use std::str::FromStr;

pub use crate::avm2::object::bitmap_data_allocator;
use crate::avm2::parameters::ParametersExt;

/// Copy the static data from a given Bitmap into a new BitmapData.
///
/// `bd` is assumed to be an uninstantiated library symbol, associated with the
/// class named by `name`.
pub fn fill_bitmap_data_from_symbol<'gc>(
    activation: &mut Activation<'_, 'gc>,
    bd: Bitmap<'gc>,
    new_bitmap_data: GcCell<'gc, BitmapData<'gc>>,
) {
    new_bitmap_data
        .write(activation.context.gc_context)
        .set_pixels(
            bd.width().into(),
            bd.height().into(),
            true,
            bd.bitmap_data().read().pixels().to_vec(),
        );
}

/// Implements `flash.display.BitmapData`'s 'init' method (invoked from the AS3 constructor)
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
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

            if let Some(Character::Bitmap(bitmap)) = character {
                // Instantiating BitmapData from an Animate-style bitmap asset
                fill_bitmap_data_from_symbol(activation, bitmap, new_bitmap_data);
            } else {
                if character.is_some() {
                    //TODO: Determine if mismatched symbols will still work as a
                    //regular BitmapData subclass, or if this should throw
                    tracing::warn!(
                        "BitmapData subclass {:?} is associated with a non-bitmap symbol",
                        name
                    );
                }

                let width = args.get_u32(activation, 0)?;
                let height = args.get_u32(activation, 1)?;
                let transparency = args.get_bool(2);
                let fill_color = args.get_u32(activation, 3)?;

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

/// Implements `BitmapData.width`'s getter.
pub fn get_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
pub fn get_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
pub fn get_transparent<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let x = args.get_i32(activation, 0)?;
        let y = args.get_i32(activation, 1)?;

        bitmap_data
            .write(activation.context.gc_context)
            .scroll(x, y);
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.copyPixels`.
pub fn copy_pixels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let source_bitmap = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let source_rect = args.get_object(activation, 1, "sourceRect")?;

        let src_min_x = source_rect
            .get_public_property("x", activation)?
            .coerce_to_i32(activation)?;
        let src_min_y = source_rect
            .get_public_property("y", activation)?
            .coerce_to_i32(activation)?;
        let src_width = source_rect
            .get_public_property("width", activation)?
            .coerce_to_i32(activation)?;
        let src_height = source_rect
            .get_public_property("height", activation)?
            .coerce_to_i32(activation)?;

        let dest_point = args.get_object(activation, 2, "destPoint")?;

        let dest_x = dest_point
            .get_public_property("x", activation)?
            .coerce_to_i32(activation)?;
        let dest_y = dest_point
            .get_public_property("y", activation)?
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

                    if let Some(alpha_point) = args.try_get_object(activation, 4) {
                        x = alpha_point
                            .get_public_property("x", activation)?
                            .coerce_to_i32(activation)?;
                        y = alpha_point
                            .get_public_property("y", activation)?
                            .coerce_to_i32(activation)?;
                    }

                    alpha_source = Some((alpha_bitmap, (x, y)));
                }
            }

            let merge_alpha = args.get_bool(5);

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

/// Implements `BitmapData.getPixels`.
pub fn get_pixels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let rectangle = args.get_object(activation, 0, "rect")?;
        let x = rectangle
            .get_public_property("x", activation)?
            .coerce_to_i32(activation)?;
        let y = rectangle
            .get_public_property("y", activation)?
            .coerce_to_i32(activation)?;
        let width = rectangle
            .get_public_property("width", activation)?
            .coerce_to_i32(activation)?;
        let height = rectangle
            .get_public_property("height", activation)?
            .coerce_to_i32(activation)?;
        let bytearray = ByteArrayObject::from_storage(
            activation,
            bitmap_data.read().get_pixels(x, y, width, height)?,
        )?;
        return Ok(bytearray.into());
    }

    Ok(Value::Undefined)
}

pub fn get_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let rectangle = args.get_object(activation, 0, "rect")?;
        let x = rectangle
            .get_public_property("x", activation)?
            .coerce_to_i32(activation)?;
        let y = rectangle
            .get_public_property("y", activation)?
            .coerce_to_i32(activation)?;
        let width = rectangle
            .get_public_property("width", activation)?
            .coerce_to_i32(activation)?;
        let height = rectangle
            .get_public_property("height", activation)?
            .coerce_to_i32(activation)?;

        let pixels = bitmap_data.read().get_vector(x, y, width, height);

        let value_type = activation.avm2().classes().uint;
        let new_storage = VectorStorage::from_values(pixels, false, value_type);

        return Ok(VectorObject::from_vector(new_storage, activation)?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.getPixel`.
pub fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let x = args.get_i32(activation, 0)?;
        let y = args.get_i32(activation, 1)?;
        return Ok((bitmap_data.read().get_pixel(x, y) as u32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.getPixel32`.
pub fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = args.get_i32(activation, 0)?;
        let y = args.get_i32(activation, 1)?;
        let pixel = i32::from(bitmap_data.read().get_pixel32(x, y));
        return Ok((pixel as u32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixel`.
pub fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = args.get_u32(activation, 0)?;
        let y = args.get_u32(activation, 1)?;
        let color = args.get_i32(activation, 2)?;
        bitmap_data
            .write(activation.context.gc_context)
            .set_pixel(x, y, color.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixel32`.
pub fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = args.get_i32(activation, 0)?;
        let y = args.get_i32(activation, 1)?;
        let color = args.get_i32(activation, 2)?;
        bitmap_data
            .write(activation.context.gc_context)
            .set_pixel32(x, y, color.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixels`.
pub fn set_pixels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args.get_object(activation, 0, "rect")?;

    let bytearray = args
        .get(1)
        .unwrap_or(&Value::Undefined)
        .coerce_to_object(activation)?;
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let x = rectangle
            .get_public_property("x", activation)?
            .coerce_to_u32(activation)?;
        let y = rectangle
            .get_public_property("y", activation)?
            .coerce_to_u32(activation)?;
        let width = rectangle
            .get_public_property("width", activation)?
            .coerce_to_u32(activation)?;
        let height = rectangle
            .get_public_property("height", activation)?
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
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        bitmap_data.read().check_valid(activation)?;
        let source_bitmap = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_object(activation)?;

        let source_rect = args.get_object(activation, 1, "sourceRect")?;

        let dest_point = args.get_object(activation, 2, "destPoint")?;

        let dest_x = dest_point
            .get_public_property("x", activation)?
            .coerce_to_u32(activation)?;
        let dest_y = dest_point
            .get_public_property("y", activation)?
            .coerce_to_u32(activation)?;

        let source_channel = args.get_i32(activation, 3)?;

        let dest_channel = args.get_i32(activation, 4)?;

        if let Some(source_bitmap) = source_bitmap.as_bitmap_data() {
            //TODO: what if source is disposed
            let src_min_x = source_rect
                .get_public_property("x", activation)?
                .coerce_to_u32(activation)?;
            let src_min_y = source_rect
                .get_public_property("y", activation)?
                .coerce_to_u32(activation)?;
            let src_width = source_rect
                .get_public_property("width", activation)?
                .coerce_to_u32(activation)?;
            let src_height = source_rect
                .get_public_property("height", activation)?
                .coerce_to_u32(activation)?;
            let src_max_x = src_min_x + src_width;
            let src_max_y = src_min_y + src_height;

            if GcCell::ptr_eq(bitmap_data, source_bitmap) {
                let src_bitmap_data_clone = source_bitmap.read().clone();
                let mut bitmap_data_write = bitmap_data.write(activation.context.gc_context);
                bitmap_data_write.copy_channel(
                    (dest_x, dest_y),
                    (src_min_x, src_min_y, src_max_x, src_max_y),
                    &src_bitmap_data_clone,
                    source_channel,
                    dest_channel,
                );
            } else {
                let mut bitmap_data_write = bitmap_data.write(activation.context.gc_context);
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
    Ok(Value::Undefined)
}

pub fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        if !bitmap_data.disposed() {
            let x = args.get_u32(activation, 0)?;
            let y = args.get_u32(activation, 1)?;
            let color = args.get_i32(activation, 2)?;

            let color: Color = color.into();
            let color: Color = color.to_premultiplied_alpha(bitmap_data.transparency());

            bitmap_data.flood_fill(x, y, color);
        }
    }

    Ok(Value::Undefined)
}

pub fn noise<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let low = args.get_u32(activation, 1)? as u8;

    let high = args.get_u32(activation, 2)? as u8;

    let channel_options = ChannelOptions::from_bits_truncate(args.get_u32(activation, 3)? as u8);

    let gray_scale = args.get_bool(4);

    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        if !bitmap_data.disposed() {
            let random_seed = args.get_i32(activation, 0)?;
            bitmap_data.noise(random_seed, low, high.max(low), channel_options, gray_scale)
        }
    }
    Ok(Value::Undefined)
}

pub fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        if !bitmap_data.disposed() {
            // TODO: Re-use `object_to_rectangle` in `movie_clip.rs`.
            let rectangle = args.get_object(activation, 0, "rect")?;
            let x = rectangle
                .get_public_property("x", activation)?
                .coerce_to_i32(activation)?;
            let y = rectangle
                .get_public_property("y", activation)?
                .coerce_to_i32(activation)?;
            let width = rectangle
                .get_public_property("width", activation)?
                .coerce_to_i32(activation)?;
            let height = rectangle
                .get_public_property("height", activation)?
                .coerce_to_i32(activation)?;

            let x_min = x.max(0) as u32;
            let x_max = (x + width) as u32;
            let y_min = y.max(0) as u32;
            let y_max = (y + height) as u32;

            let color_transform = args.get_object(activation, 1, "colorTransform")?;
            let color_transform =
                crate::avm2::globals::flash::geom::transform::object_to_color_transform(
                    color_transform,
                    activation,
                )?;

            bitmap_data.color_transform(x_min, y_min, x_max, y_max, color_transform);
        }
    }

    Ok(Value::Undefined)
}

pub fn get_color_bounds_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|t| t.as_bitmap_data()) {
        let bitmap_data = bitmap_data.read();
        if !bitmap_data.disposed() {
            let find_color = args.get_bool(2);

            let mask = args.get_i32(activation, 0)?;
            let color = args.get_i32(activation, 1)?;

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

    Ok(Value::Undefined)
}

pub fn lock<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.display.BitmapData", "lock");
    Ok(Value::Undefined)
}

pub fn unlock<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.display.BitmapData", "unlock");
    Ok(Value::Undefined)
}

pub fn hit_test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_method!(activation, "flash.display.BitmapData", "hitTest");
    Ok(false.into())
}

/// Implements `BitmapData.draw`
pub fn draw<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data_wrapper()) {
        let mut transform = Transform::default();
        let mut blend_mode = BlendMode::Normal;

        if let Some(matrix) = args.try_get_object(activation, 1) {
            transform.matrix =
                crate::avm2::globals::flash::geom::transform::object_to_matrix(matrix, activation)?;
        }

        if let Some(color_transform) = args.try_get_object(activation, 2) {
            transform.color_transform =
                crate::avm2::globals::flash::geom::transform::object_to_color_transform(
                    color_transform,
                    activation,
                )?;
        }

        if let Some(mode) = args.try_get_string(activation, 3)? {
            if let Ok(mode) = BlendMode::from_str(&mode.to_string()) {
                blend_mode = mode;
            } else {
                tracing::error!("Unknown blend mode {:?}", mode);
                return Err("ArgumentError: Error #2008: Parameter blendMode must be one of the accepted values.".into());
            }
        }

        let mut clip_rect = None;

        if let Some(clip_rect_obj) = args.try_get_object(activation, 4) {
            clip_rect = Some(super::display_object::object_to_rectangle(
                activation,
                clip_rect_obj,
            )?);
        }

        let smoothing = args.get_bool(5);

        let source = args.get_object(activation, 0, "source")?;

        let source = if let Some(source_object) = source.as_display_object() {
            IBitmapDrawable::DisplayObject(source_object)
        } else if let Some(source_bitmap) = source.as_bitmap_data_wrapper() {
            IBitmapDrawable::BitmapData(source_bitmap)
        } else {
            return Err(format!("BitmapData.draw: unexpected source {source:?}").into());
        };

        // Drawing onto a BitmapData doesn't use any of the CPU-side pixels
        // Do this last, so that we only call `overwrite_cpu_pixels_from_gpu`
        // if we're actually going to draw something.
        let bitmap_data = bitmap_data.overwrite_cpu_pixels_from_gpu(&mut activation.context);
        // If the bitmapdata is invalid, it's fine to return early, since the pixels
        // are inaccessible
        bitmap_data.read().check_valid(activation)?;
        match bitmap_data.write(activation.context.gc_context).draw(
            source,
            transform,
            smoothing,
            blend_mode,
            clip_rect,
            activation.context.stage.quality(),
            &mut activation.context,
        ) {
            Ok(()) => {}
            Err(BitmapDataDrawError::Unimplemented) => {
                return Err("Render backend does not support BitmapData.draw".into());
            }
        };
    }
    Ok(Value::Undefined)
}

/// Implements `BitmapData.drawWithQuality`
pub fn draw_with_quality<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data_wrapper()) {
        // Drawing onto a BitmapData doesn't use any of the CPU-side pixels
        let bitmap_data = bitmap_data.overwrite_cpu_pixels_from_gpu(&mut activation.context);
        bitmap_data.read().check_valid(activation)?;
        let mut transform = Transform::default();
        let mut blend_mode = BlendMode::Normal;

        if let Some(matrix) = args.try_get_object(activation, 1) {
            transform.matrix =
                crate::avm2::globals::flash::geom::transform::object_to_matrix(matrix, activation)?;
        }

        if let Some(color_transform) = args.try_get_object(activation, 2) {
            transform.color_transform =
                crate::avm2::globals::flash::geom::transform::object_to_color_transform(
                    color_transform,
                    activation,
                )?;
        }

        if let Some(mode) = args.try_get_string(activation, 3)? {
            if let Ok(mode) = BlendMode::from_str(&mode.to_string()) {
                blend_mode = mode;
            } else {
                tracing::error!("Unknown blend mode {:?}", mode);
                return Err("ArgumentError: Error #2008: Parameter blendMode must be one of the accepted values.".into());
            }
        }

        let mut clip_rect = None;

        if let Some(clip_rect_obj) = args.try_get_object(activation, 4) {
            clip_rect = Some(super::display_object::object_to_rectangle(
                activation,
                clip_rect_obj,
            )?);
        }

        let mut bitmap_data = bitmap_data.write(activation.context.gc_context);
        let smoothing = args.get_bool(5);

        let source = args.get_object(activation, 0, "source")?;

        let source = if let Some(source_object) = source.as_display_object() {
            IBitmapDrawable::DisplayObject(source_object)
        } else if let Some(source_bitmap) = source.as_bitmap_data_wrapper() {
            IBitmapDrawable::BitmapData(source_bitmap)
        } else {
            return Err(format!("BitmapData.drawWithQuality: unexpected source {source:?}").into());
        };

        // Unknown quality defaults to stage's quality
        let quality = if let Some(quality) = args.try_get_string(activation, 6)? {
            match quality.parse() {
                Ok(quality) => quality,
                Err(_) => {
                    return Err(Error::AvmError(argument_error(
                        activation,
                        "One of the parameters is invalid.",
                        2004,
                    )?));
                }
            }
        } else {
            activation.context.stage.quality()
        };

        match bitmap_data.draw(
            source,
            transform,
            smoothing,
            blend_mode,
            clip_rect,
            quality,
            &mut activation.context,
        ) {
            Ok(()) => {}
            Err(BitmapDataDrawError::Unimplemented) => {
                return Err("Render backend does not support BitmapData.draw".into());
            }
        }
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.fillRect`
pub fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args.get_object(activation, 0, "rect")?;

    let color = args.get_i32(activation, 1)?;

    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        let x = rectangle
            .get_public_property("x", activation)?
            .coerce_to_u32(activation)?;
        let y = rectangle
            .get_public_property("y", activation)?
            .coerce_to_u32(activation)?;
        let width = rectangle
            .get_public_property("width", activation)?
            .coerce_to_u32(activation)?;
        let height = rectangle
            .get_public_property("height", activation)?
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
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        // Don't check if we've already disposed this BitmapData - 'BitmapData.dispose()' can be called
        // multiple times
        bitmap_data.write(activation.context.gc_context).dispose();
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.rect`
pub fn get_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dest_bitmap) = this.and_then(|this| this.as_bitmap_data_wrapper()) {
        let dest_bitmap_data = dest_bitmap.overwrite_cpu_pixels_from_gpu(&mut activation.context);
        dest_bitmap_data.read().check_valid(activation)?;
        let source_bitmap = args.get_object(activation, 0, "sourceBitmapData")?
            .as_bitmap_data()
            .ok_or_else(|| {
                Error::from(format!("TypeError: Error #1034: Type Coercion failed: cannot convert {} to flash.display.BitmapData.", args[0].coerce_to_string(activation).unwrap_or_default()))
            })?;
        let source_handle = match source_bitmap
            .write(activation.context.gc_context)
            .bitmap_handle(activation.context.renderer)
        {
            Some(handle) => handle,
            None => {
                tracing::warn!("Ignoring BitmapData.apply_filter() with an undrawable source");
                return Ok(Value::Undefined);
            }
        };
        let source_rect = args.get_object(activation, 1, "sourceRect")?;
        let source_rect = super::display_object::object_to_rectangle(activation, source_rect)?;
        let source_point = (
            source_rect.x_min.to_pixels().floor() as u32,
            source_rect.y_min.to_pixels().floor() as u32,
        );
        let source_size = (
            source_rect.width().to_pixels().ceil() as u32,
            source_rect.height().to_pixels().ceil() as u32,
        );
        let dest_point = args.get_object(activation, 2, "dstPoint")?;
        let dest_point = (
            dest_point
                .get_public_property("x", activation)?
                .coerce_to_u32(activation)?,
            dest_point
                .get_public_property("x", activation)?
                .coerce_to_u32(activation)?,
        );
        let filter = args.get_object(activation, 3, "filter")?;
        let filter = Filter::from_avm2_object(activation, filter)?;
        let mut dest_bitmap_data = dest_bitmap_data.write(activation.context.gc_context);
        dest_bitmap_data.apply_filter(
            &mut activation.context,
            source_handle,
            source_point,
            source_size,
            dest_point,
            filter,
        )
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.clone`
pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
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
    activation: &mut Activation<'_, 'gc>,
    this: Option<Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.and_then(|this| this.as_bitmap_data()) {
        if !bitmap_data.read().disposed() {
            let base_x = args.get_f64(activation, 0)?;
            let base_y = args.get_f64(activation, 1)?;
            let num_octaves = args.get_u32(activation, 2)? as usize;
            let seed = args.get_i32(activation, 3)? as i64;
            let stitch = args.get_bool(4);
            let fractal_noise = args.get_bool(5);
            let channel_options =
                ChannelOptions::from_bits_truncate(args.get_i32(activation, 6)? as u8);
            let grayscale = args.get_bool(7);
            let offsets = args.try_get_object(activation, 8);

            let octave_offsets: Result<Vec<_>, Error<'gc>> = (0..num_octaves)
                .map(|i| {
                    if let Some(offsets) = offsets {
                        if let Some(offsets) = offsets.as_array_storage() {
                            if let Some(Value::Object(e)) = offsets.get(i) {
                                let x = e
                                    .get_public_property("x", activation)?
                                    .coerce_to_number(activation)?;
                                let y = e
                                    .get_public_property("y", activation)?
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
