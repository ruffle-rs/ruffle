//! `flash.display.BitmapData` builtin/prototype

use crate::avm2::activation::Activation;
use crate::avm2::bytearray::ByteArrayStorage;
use crate::avm2::error::{
    argument_error, make_error_2004, make_error_2007, make_error_2008, range_error, Error2004Type,
};
use crate::avm2::filters::FilterAvm2Ext;
pub use crate::avm2::object::bitmap_data_allocator;
use crate::avm2::object::{BitmapDataObject, ByteArrayObject, Object, TObject, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::avm2::vector::VectorStorage;
use crate::avm2::Error;
use crate::avm2_stub_method;
use crate::bitmap::bitmap_data::{
    BitmapData, BitmapDataWrapper, ChannelOptions, ThresholdOperation,
};
use crate::bitmap::bitmap_data::{BitmapDataDrawError, IBitmapDrawable};
use crate::bitmap::{is_size_valid, operations};
use crate::character::{Character, CompressedBitmap};
use crate::display_object::TDisplayObject;
use crate::ecma_conversions::round_to_even;
use crate::swf::BlendMode;
use gc_arena::GcCell;
use ruffle_render::filters::Filter;
use ruffle_render::transform::Transform;
use std::str::FromStr;
use swf::{Rectangle, Twips};

// Computes the integer x,y,width,height values from
// the given `Rectangle`. This method performs `x + width`
// and `y + height` as floating point operations before
// `round_to_even`, which is needed to match Flash Player's
// rounding behavior.
fn get_rectangle_x_y_width_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    rectangle: Object<'gc>,
) -> Result<(i32, i32, i32, i32), Error<'gc>> {
    let x = rectangle
        .get_public_property("x", activation)?
        .coerce_to_number(activation)?;
    let y = rectangle
        .get_public_property("y", activation)?
        .coerce_to_number(activation)?;
    let width = rectangle
        .get_public_property("width", activation)?
        .coerce_to_number(activation)?;
    let height = rectangle
        .get_public_property("height", activation)?
        .coerce_to_number(activation)?;

    let x_max = round_to_even(x + width);
    let y_max = round_to_even(y + height);

    let x_int = round_to_even(x);
    let y_int = round_to_even(y);

    let width_int = x_max - x_int;
    let height_int = y_max - y_int;

    Ok((x_int, y_int, width_int, height_int))
}

/// Copy the static data from a given Bitmap into a new BitmapData.
///
/// `bd` is assumed to be an uninstantiated library symbol, associated with the
/// class named by `name`.
pub fn fill_bitmap_data_from_symbol<'gc>(
    activation: &mut Activation<'_, 'gc>,
    bd: &CompressedBitmap,
) -> BitmapDataWrapper<'gc> {
    let bitmap = bd.decode().expect("Failed to decode BitmapData");
    let new_bitmap_data = GcCell::new(
        activation.context.gc_context,
        BitmapData::new_with_pixels(
            bitmap.width(),
            bitmap.height(),
            true,
            bitmap
                .as_colors()
                .map(crate::bitmap::bitmap_data::Color::from)
                .collect(),
        ),
    );
    BitmapDataWrapper::new(new_bitmap_data)
}

/// Implements `flash.display.BitmapData`'s 'init' method (invoked from the AS3 constructor)
pub fn init<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // We set the underlying BitmapData instance - we start out with a dummy BitmapDataWrapper,
    // which makes custom classes see a disposed BitmapData before they call super()
    let name = this.instance_class().name();
    let character = activation
        .context
        .library
        .avm2_class_registry()
        .class_symbol(this.instance_class())
        .and_then(|(movie, chara_id)| {
            activation
                .context
                .library
                .library_for_movie_mut(movie)
                .character_by_id(chara_id)
                .cloned()
        });

    let new_bitmap_data = if let Some(Character::Bitmap {
        compressed,
        avm2_bitmapdata_class: _,
        handle: _,
    }) = character
    {
        // Instantiating BitmapData from an Animate-style bitmap asset
        fill_bitmap_data_from_symbol(activation, &compressed)
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
            return Err(Error::AvmError(argument_error(
                activation,
                "Error #2015: Invalid BitmapData.",
                2015,
            )?));
        }

        let new_bitmap_data = BitmapData::new(width, height, transparency, fill_color);
        BitmapDataWrapper::new(GcCell::new(activation.context.gc_context, new_bitmap_data))
    };

    new_bitmap_data.init_object2(activation.context.gc_context, this);
    this.init_bitmap_data(activation.context.gc_context, new_bitmap_data);

    Ok(Value::Undefined)
}

/// Implements `BitmapData.width`'s getter.
pub fn get_width<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        return Ok((bitmap_data.width() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.height`'s getter.
pub fn get_height<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        return Ok((bitmap_data.height() as i32).into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.transparent`'s getter.
pub fn get_transparent<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        return Ok(bitmap_data.transparency().into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.scroll`.
pub fn scroll<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let x = args.get_i32(activation, 0)?;
        let y = args.get_i32(activation, 1)?;

        operations::scroll(
            activation.context.gc_context,
            activation.context.renderer,
            bitmap_data,
            x,
            y,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.copyPixels`.
pub fn copy_pixels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let source_bitmap = args.get_object(activation, 0, "sourceBitmapData")?;

        let source_rect = args.get_object(activation, 1, "sourceRect")?;

        let (src_min_x, src_min_y, src_width, src_height) =
            get_rectangle_x_y_width_height(activation, source_rect)?;

        let dest_point = args.get_object(activation, 2, "destPoint")?;

        let dest_x = dest_point
            .get_public_property("x", activation)?
            .coerce_to_i32(activation)?;
        let dest_y = dest_point
            .get_public_property("y", activation)?
            .coerce_to_i32(activation)?;

        if let Some(src_bitmap) = source_bitmap.as_bitmap_data() {
            src_bitmap.check_valid(activation)?;

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
                operations::copy_pixels_with_alpha_source(
                    activation.context,
                    bitmap_data,
                    src_bitmap,
                    (src_min_x, src_min_y, src_width, src_height),
                    (dest_x, dest_y),
                    alpha_bitmap,
                    alpha_point,
                    merge_alpha,
                );
            } else {
                operations::copy_pixels(
                    activation.context,
                    bitmap_data,
                    src_bitmap,
                    (src_min_x, src_min_y, src_width, src_height),
                    (dest_x, dest_y),
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let rectangle = args.get_object(activation, 0, "rect")?;
        let (x, y, width, height) = get_rectangle_x_y_width_height(activation, rectangle)?;
        let mut storage = ByteArrayStorage::new();

        operations::get_pixels_as_byte_array(
            activation,
            bitmap_data,
            x,
            y,
            width,
            height,
            &mut storage,
        )?;
        let bytearray = ByteArrayObject::from_storage(activation, storage)?;
        return Ok(bytearray.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.copyPixelsToByteArray`.
pub fn copy_pixels_to_byte_array<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let rectangle = args.get_object(activation, 0, "rect")?;
        let storage = args.get_object(activation, 1, "data")?;
        let mut storage = storage.as_bytearray_mut().unwrap();
        let (x, y, width, height) = get_rectangle_x_y_width_height(activation, rectangle)?;
        operations::get_pixels_as_byte_array(
            activation,
            bitmap_data,
            x,
            y,
            width,
            height,
            &mut storage,
        )?;
    }

    Ok(Value::Undefined)
}

pub fn get_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let rectangle = args.get_object(activation, 0, "rect")?;
        let (x, y, width, height) = get_rectangle_x_y_width_height(activation, rectangle)?;

        let pixels = operations::get_vector(
            bitmap_data,
            activation.context.renderer,
            x,
            y,
            width,
            height,
        );

        let value_type = activation.avm2().class_defs().uint;
        let new_storage = VectorStorage::from_values(pixels, false, Some(value_type));

        return Ok(VectorObject::from_vector(new_storage, activation)?.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.getPixel`.
pub fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let x = args.get_u32(activation, 0)?;
        let y = args.get_u32(activation, 1)?;
        let col = operations::get_pixel(bitmap_data, activation.context.renderer, x, y);
        return Ok(col.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.getPixel32`.
pub fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let x = args.get_u32(activation, 0)?;
        let y = args.get_u32(activation, 1)?;
        let pixel = operations::get_pixel32(bitmap_data, activation.context.renderer, x, y);
        return Ok(pixel.into());
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixel`.
pub fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        let x = args.get_u32(activation, 0)?;
        let y = args.get_u32(activation, 1)?;
        let color = args.get_u32(activation, 2)?;
        operations::set_pixel(
            activation.context.gc_context,
            activation.context.renderer,
            bitmap_data,
            x,
            y,
            color.into(),
        );
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixel32`.
pub fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;

        let x = args.get_u32(activation, 0)?;
        let y = args.get_u32(activation, 1)?;
        let color = args.get_u32(activation, 2)?;

        operations::set_pixel32(
            activation.context.gc_context,
            activation.context.renderer,
            bitmap_data,
            x,
            y,
            color,
        );
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setPixels`.
pub fn set_pixels<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args.get_object(activation, 0, "rect")?;
    let bytearray = args.get_object(activation, 0, "inputByteArray")?;

    if let Some(bitmap_data) = this.as_bitmap_data() {
        let (x, y, width, height) = get_rectangle_x_y_width_height(activation, rectangle)?;

        let mut ba_write = bytearray
            .as_bytearray_mut()
            .expect("Parameter must be a bytearray");

        operations::set_pixels_from_byte_array(
            activation.context.gc_context,
            activation.context.renderer,
            bitmap_data,
            x,
            y,
            width,
            height,
            &mut ba_write,
        )
        .map_err(|e| e.to_avm(activation))?;
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.setVector`.
pub fn set_vector<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args.get_object(activation, 0, "rect")?;
    // Note - flash player misspells this as 'imputVector'.
    let vec = args.get_object(activation, 1, "imputVector")?;
    if let Some(bitmap_data) = this.as_bitmap_data() {
        let x = rectangle
            .get_public_property("x", activation)?
            .coerce_to_number(activation)?;
        let y = rectangle
            .get_public_property("y", activation)?
            .coerce_to_number(activation)?;
        let width = rectangle
            .get_public_property("width", activation)?
            .coerce_to_number(activation)?;
        let height = rectangle
            .get_public_property("height", activation)?
            .coerce_to_number(activation)?;

        // Clamp to bitmap rect.
        let bitmap_width = f64::from(bitmap_data.width());
        let bitmap_height = f64::from(bitmap_data.height());
        let x_min = x.clamp(0.0, bitmap_width);
        let y_min = y.clamp(0.0, bitmap_height);
        let x_max = (x + width).clamp(x_min, bitmap_width);
        let y_max = (y + height).clamp(y_min, bitmap_height);

        let x_min = x_min as u32;
        let x_max = x_max as u32;
        let y_min = y_min as u32;
        let y_max = y_max as u32;

        let vec_read = vec
            .as_vector_storage()
            .expect("BitmapData.setVector: Expected vector");

        operations::set_vector(
            activation,
            bitmap_data,
            x_min,
            y_min,
            x_max,
            y_max,
            &vec_read,
        )?;
    }

    Ok(Value::Undefined)
}

/// Implements `BitmapData.copyChannel`.
pub fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;

        let source_bitmap = args.get_object(activation, 0, "sourceBitmapData")?;
        let source_rect = args.get_object(activation, 1, "sourceRect")?;
        let dest_point = args.get_object(activation, 2, "destPoint")?;

        let dest_x = dest_point
            .get_public_property("x", activation)?
            .coerce_to_i32(activation)?;
        let dest_y = dest_point
            .get_public_property("y", activation)?
            .coerce_to_i32(activation)?;

        let source_channel = args.get_i32(activation, 3)?;

        let dest_channel = args.get_i32(activation, 4)?;

        if let Some(source_bitmap) = source_bitmap.as_bitmap_data() {
            //TODO: what if source is disposed

            let (src_min_x, src_min_y, src_width, src_height) =
                get_rectangle_x_y_width_height(activation, source_rect)?;

            operations::copy_channel(
                activation.context.gc_context,
                activation.context.renderer,
                bitmap_data,
                (dest_x, dest_y),
                (src_min_x, src_min_y, src_width, src_height),
                source_bitmap,
                source_channel,
                dest_channel,
            );
        }
    }
    Ok(Value::Undefined)
}

pub fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
            let x = args.get_u32(activation, 0)?;
            let y = args.get_u32(activation, 1)?;
            let color = args.get_u32(activation, 2)?;

            operations::flood_fill(
                activation.context.gc_context,
                activation.context.renderer,
                bitmap_data,
                x,
                y,
                color,
            );
        }
    }

    Ok(Value::Undefined)
}

pub fn noise<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let low = args.get_u32(activation, 1)? as u8;

    let high = args.get_u32(activation, 2)? as u8;

    let channel_options = ChannelOptions::from_bits_truncate(args.get_u32(activation, 3)? as u8);

    let gray_scale = args.get_bool(4);

    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let random_seed = args.get_i32(activation, 0)?;
        operations::noise(
            activation.context.gc_context,
            bitmap_data,
            random_seed,
            low,
            high.max(low),
            channel_options,
            gray_scale,
        );
    }
    Ok(Value::Undefined)
}

pub fn color_transform<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
            // TODO: Re-use `object_to_rectangle` in `movie_clip.rs`.
            let rectangle = args.get_object(activation, 0, "rect")?;
            let (x, y, width, height) = get_rectangle_x_y_width_height(activation, rectangle)?;

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

            operations::color_transform(
                activation.context.gc_context,
                activation.context.renderer,
                bitmap_data,
                x_min,
                y_min,
                x_max,
                y_max,
                &color_transform,
            );
        }
    }

    Ok(Value::Undefined)
}

pub fn get_color_bounds_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
            let find_color = args.get_bool(2);

            let mask = args.get_u32(activation, 0)?;
            let color = args.get_u32(activation, 1)?;

            let (x, y, w, h) = operations::color_bounds_rect(
                activation.context.renderer,
                bitmap_data,
                find_color,
                mask,
                color,
            );

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
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // `BitmapData.lock` tells Flash Player to temporarily stop updating the player's
    // dirty region for any Bitmap stage instances displaying this BitmapData.
    // Normally, each call to `setPixel` etc. causes Flash to update the player dirty
    // region with the changed area.
    //
    // Note that `lock` has no effect on future `BitmapData` operations, they will always
    // see the latest pixel data. Instead, it potentially delays the re-rendering of `Bitmap`
    // instances on the stage, based on how the player decides to update its dirty region
    // ("Show Redraw Regions" in Flash Player debugger context menu).
    //
    // Ruffle has no concept of a player dirty region for now, so this has no effect.
    Ok(Value::Undefined)
}

pub fn unlock<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // No effect (see comments for `lock`).
    Ok(Value::Undefined)
}

pub fn hit_test<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
            let first_point = args.get_object(activation, 0, "firstPoint")?;
            let top_left = (
                first_point
                    .get_public_property("x", activation)?
                    .coerce_to_i32(activation)?,
                first_point
                    .get_public_property("y", activation)?
                    .coerce_to_i32(activation)?,
            );
            let source_threshold = args.get_u32(activation, 1)?.clamp(0, u8::MAX.into()) as u8;
            let compare_object = args.get_object(activation, 2, "secondObject")?;
            let point_class = activation.avm2().classes().point.inner_class_definition();
            let rectangle_class = activation
                .avm2()
                .classes()
                .rectangle
                .inner_class_definition();

            if compare_object.is_of_type(point_class) {
                let test_point = (
                    compare_object
                        .get_public_property("x", activation)?
                        .coerce_to_i32(activation)?
                        - top_left.0,
                    compare_object
                        .get_public_property("y", activation)?
                        .coerce_to_i32(activation)?
                        - top_left.1,
                );
                return Ok(Value::Bool(operations::hit_test_point(
                    activation.context.renderer,
                    bitmap_data,
                    source_threshold,
                    test_point,
                )));
            } else if compare_object.is_of_type(rectangle_class) {
                let test_point = (
                    compare_object
                        .get_public_property("x", activation)?
                        .coerce_to_i32(activation)?
                        - top_left.0,
                    compare_object
                        .get_public_property("y", activation)?
                        .coerce_to_i32(activation)?
                        - top_left.1,
                );
                let size = (
                    compare_object
                        .get_public_property("width", activation)?
                        .coerce_to_i32(activation)?,
                    compare_object
                        .get_public_property("height", activation)?
                        .coerce_to_i32(activation)?,
                );
                return Ok(Value::Bool(operations::hit_test_rectangle(
                    activation.context.renderer,
                    bitmap_data,
                    source_threshold,
                    test_point,
                    size,
                )));
            } else if let Some(other_bmd) = compare_object.as_bitmap_data() {
                other_bmd.check_valid(activation)?;
                let second_point = args.get_object(activation, 3, "secondBitmapDataPoint")?;
                let second_point = (
                    second_point
                        .get_public_property("x", activation)?
                        .coerce_to_i32(activation)?,
                    second_point
                        .get_public_property("y", activation)?
                        .coerce_to_i32(activation)?,
                );
                let second_threshold = args.get_u32(activation, 4)?.clamp(0, u8::MAX.into()) as u8;

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
            } else if let Some(bitmap) = compare_object
                .as_display_object()
                .and_then(|dobj| dobj.as_bitmap())
            {
                let other_bmd = bitmap.bitmap_data_wrapper();
                other_bmd.check_valid(activation)?;
                let second_point = args.get_object(activation, 3, "secondBitmapDataPoint")?;
                let second_point = (
                    second_point
                        .get_public_property("x", activation)?
                        .coerce_to_i32(activation)?,
                    second_point
                        .get_public_property("y", activation)?
                        .coerce_to_i32(activation)?,
                );
                let second_threshold = args.get_u32(activation, 4)?.clamp(0, u8::MAX.into()) as u8;

                return Ok(Value::Bool(operations::hit_test_bitmapdata(
                    activation.context.renderer,
                    bitmap_data,
                    top_left,
                    source_threshold,
                    other_bmd,
                    second_point,
                    second_threshold,
                )));
            } else {
                // This is the error message Flash Player produces. Even though it's misleading.
                return Err(Error::AvmError(argument_error(
                    activation,
                    "Parameter 0 is of the incorrect type. Should be type BitmapData.",
                    2005,
                )?));
            }
        }
    }

    Ok(false.into())
}

/// Implements `BitmapData.draw`
pub fn draw<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
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
                return Err(make_error_2008(activation, "blendMode"));
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
        } else if let Some(source_bitmap) = source.as_bitmap_data() {
            IBitmapDrawable::BitmapData(source_bitmap)
        } else {
            return Err(format!("BitmapData.draw: unexpected source {source:?}").into());
        };

        // If the bitmapdata is invalid, it's fine to return early, since the pixels
        // are inaccessible
        bitmap_data.check_valid(activation)?;

        // Do this last, so that we only call `overwrite_cpu_pixels_from_gpu`
        // if we're actually going to draw something.
        let quality = activation.context.stage.quality();
        match operations::draw(
            activation.context,
            bitmap_data,
            source,
            transform,
            smoothing,
            blend_mode,
            clip_rect,
            quality,
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
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
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
                return Err(make_error_2008(activation, "blendMode"));
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
        } else if let Some(source_bitmap) = source.as_bitmap_data() {
            IBitmapDrawable::BitmapData(source_bitmap)
        } else {
            return Err(format!("BitmapData.drawWithQuality: unexpected source {source:?}").into());
        };

        // Unknown quality defaults to stage's quality
        let quality = if let Some(quality) = args.try_get_string(activation, 6)? {
            match quality.parse() {
                Ok(quality) => quality,
                Err(_) => return Err(make_error_2004(activation, Error2004Type::ArgumentError)),
            }
        } else {
            activation.context.stage.quality()
        };

        match operations::draw(
            activation.context,
            bitmap_data,
            source,
            transform,
            smoothing,
            blend_mode,
            clip_rect,
            quality,
        ) {
            Ok(()) => {}
            Err(BitmapDataDrawError::Unimplemented) => {
                return Err("Render backend does not support BitmapData.draw".into());
            }
        };
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.fillRect`
pub fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args.get_object(activation, 0, "rect")?;

    let color = args.get_u32(activation, 1)?;

    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let (x, y, width, height) = get_rectangle_x_y_width_height(activation, rectangle)?;

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
    Ok(Value::Undefined)
}

/// Implements `BitmapData.dispose`
pub fn dispose<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        // Don't check if we've already disposed this BitmapData - 'BitmapData.dispose()' can be called
        // multiple times
        bitmap_data.dispose(activation.context.gc_context);
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.rect`
pub fn get_rect<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        return Ok(activation
            .avm2()
            .classes()
            .rectangle
            .construct(
                activation,
                &[
                    0.into(),
                    0.into(),
                    bitmap_data.width().into(),
                    bitmap_data.height().into(),
                ],
            )?
            .into());
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.applyFilter`
pub fn apply_filter<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(dest_bitmap) = this.as_bitmap_data() {
        let source_bitmap = args.get_object(activation, 0, "sourceBitmapData")?
            .as_bitmap_data()
            .ok_or_else(|| {
                Error::from(format!("TypeError: Error #1034: Type Coercion failed: cannot convert {} to flash.display.BitmapData.", args[0].coerce_to_string(activation).unwrap_or_default()))
            })?;
        let source_rect = args.get_object(activation, 1, "sourceRect")?;
        let mut source_rect = super::display_object::object_to_rectangle(activation, source_rect)?;
        let filter = args.get_object(activation, 3, "filter")?;
        let filter = Filter::from_avm2_object(activation, filter)?;

        if matches!(filter, Filter::ShaderFilter(_)) {
            let source_bitmap_rect = Rectangle {
                x_min: Twips::ZERO,
                x_max: Twips::from_pixels(source_bitmap.width() as f64),
                y_min: Twips::ZERO,
                y_max: Twips::from_pixels(source_bitmap.height() as f64),
            };
            // Flash performs an odd translation/cropping behavior when sourceRect
            // has a non-zero x or y starting value, which I haven't yet managed to reproduce.
            //
            // Additionally, when both x and y are 0, the 'width' and 'height' seem to
            // be ignored completely in favor of the using the dimensions of the source
            // image (even if a larger or smaller rect is passed in)
            //
            // To make matters worse, the behavior of ShaderFilter seems platform-dependent
            // (or at least resolution-dependent). The test
            // 'tests/tests/swfs/avm2/pixelbender_effect_glassDisplace_shaderfilter/test.swf'
            // renders slightly differently in Linux vs a Windows VM (part of the mandelbrot fractal
            // in the top image is cut off in the Windows Flash Player, but not in the Linux Flash Player)
            if source_rect != source_bitmap_rect {
                avm2_stub_method!(
                    activation,
                    "flash.display.BitmapData",
                    "applyFilter",
                    "ShaderFilter with non-standard sourceRect"
                );
                source_rect = source_bitmap_rect;
            }
        }

        let source_point = (
            source_rect.x_min.to_pixels().floor() as u32,
            source_rect.y_min.to_pixels().floor() as u32,
        );
        let source_size = (
            source_rect.width().to_pixels().ceil() as u32,
            source_rect.height().to_pixels().ceil() as u32,
        );
        let dest_point = args.get_object(activation, 2, "destPoint")?;
        let dest_point = (
            dest_point
                .get_public_property("x", activation)?
                .coerce_to_u32(activation)?,
            dest_point
                .get_public_property("y", activation)?
                .coerce_to_u32(activation)?,
        );

        operations::apply_filter(
            activation.context,
            dest_bitmap,
            source_bitmap,
            source_point,
            source_size,
            dest_point,
            filter,
        );
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.clone`
pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
            let new_bitmap_data = bitmap_data.clone_data(activation.context.renderer);

            let class = activation.avm2().classes().bitmapdata;
            let new_bitmap_data_object = BitmapDataObject::from_bitmap_data_internal(
                activation,
                BitmapDataWrapper::new(GcCell::new(activation.context.gc_context, new_bitmap_data)),
                class,
            )?;

            return Ok(new_bitmap_data_object.into());
        }
    }
    Ok(Value::Undefined)
}

/// Implement `BitmapData.paletteMap`
pub fn palette_map<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;
        let source_bitmap = args
            .get_object(activation, 0, "sourceBitmapData")?
            .as_bitmap_data()
            .unwrap();

        let source_rect = args.get_object(activation, 1, "sourceRect")?;
        let source_rect = super::display_object::object_to_rectangle(activation, source_rect)?;
        let source_point = (
            source_rect.x_min.to_pixels().floor() as i32,
            source_rect.y_min.to_pixels().floor() as i32,
        );
        let source_size = (
            source_rect.width().to_pixels().ceil() as i32,
            source_rect.height().to_pixels().ceil() as i32,
        );
        let dest_point = args.get_object(activation, 2, "destPoint")?;
        let dest_point = (
            dest_point
                .get_public_property("x", activation)?
                .coerce_to_i32(activation)?,
            dest_point
                .get_public_property("x", activation)?
                .coerce_to_i32(activation)?,
        );

        let mut get_channel = |index: usize, shift: usize| -> Result<[u32; 256], Error<'gc>> {
            let arg = args.get(index).unwrap_or(&Value::Null);
            let mut array = [0_u32; 256];
            for (i, item) in array.iter_mut().enumerate() {
                *item = if let Value::Object(arg) = arg {
                    arg.get_enumerant_value(i as u32, activation)?
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

        operations::palette_map(
            activation.context.gc_context,
            activation.context.renderer,
            bitmap_data,
            source_bitmap,
            (source_point.0, source_point.1, source_size.0, source_size.1),
            dest_point,
            (red_array, green_array, blue_array, alpha_array),
        );
    }

    Ok(Value::Undefined)
}

/// Implement `BitmapData.perlinNoise`
pub fn perlin_noise<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
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

    Ok(Value::Undefined)
}

/// Implement `BitmapData.threshold`
pub fn threshold<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
            let src_bitmap = args.get_object(activation, 0, "sourceBitmapData")?;
            let source_rect = args.get_object(activation, 1, "sourceRect")?;
            let dest_point = args.get_object(activation, 2, "destPoint")?;
            let dest_point = (
                dest_point
                    .get_public_property("x", activation)?
                    .coerce_to_i32(activation)?,
                dest_point
                    .get_public_property("y", activation)?
                    .coerce_to_i32(activation)?,
            );
            let operation = args.try_get_string(activation, 3)?;
            let threshold = args.get_u32(activation, 4)?;
            let color = args.get_u32(activation, 5)?;
            let mask = args.get_u32(activation, 6)?;
            let copy_source = args.get_bool(7);

            let operation = if let Some(operation) = operation {
                if let Some(operation) = ThresholdOperation::from_wstr(&operation) {
                    operation
                } else {
                    // It's wrong but this is what Flash says.
                    return Err(Error::AvmError(argument_error(
                        activation,
                        "Parameter 0 is of the incorrect type. Should be type Operation.",
                        2005,
                    )?));
                }
            } else {
                return Err(make_error_2007(activation, "operation"));
            };

            let (src_min_x, src_min_y, src_width, src_height) =
                get_rectangle_x_y_width_height(activation, source_rect)?;

            if let Some(src_bitmap) = src_bitmap.as_bitmap_data() {
                src_bitmap.check_valid(activation)?;

                return Ok(operations::threshold(
                    activation.context.gc_context,
                    activation.context.renderer,
                    bitmap_data,
                    src_bitmap,
                    (src_min_x, src_min_y, src_width, src_height),
                    dest_point,
                    operation,
                    threshold,
                    color,
                    mask,
                    copy_source,
                )
                .into());
            }
        }
    }

    Ok(Value::Undefined)
}

/// Implement `BitmapData.compare`
pub fn compare<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    const EQUIVALENT: i32 = 0;
    const NOT_BITMAP: i32 = -1;
    const BITMAP_DISPOSED: i32 = -2;
    const DIFFERENT_WIDTHS: i32 = -3;
    const DIFFERENT_HEIGHTS: i32 = -4;

    let this_bitmap_data = if let Some(this_bitmap_data) = this.as_bitmap_data() {
        this_bitmap_data
    } else {
        return Ok(NOT_BITMAP.into());
    };
    this_bitmap_data.check_valid(activation)?;

    if this_bitmap_data.disposed() {
        // The documentation says that -2 should be returned here, but -1 is actually returned.
        return Ok(NOT_BITMAP.into());
    }

    let other_bitmap_data = if let Some(other_bitmap_data) = args
        .get_object(activation, 0, "otherBitmapData")?
        .as_bitmap_data()
    {
        other_bitmap_data
    } else {
        // The documentation for AVM1 says that -1 should be returned here,
        // but -2 is actually returned.
        // TODO: For AVM2, this branch should never get reached, since
        //   AVM2 checks types.
        return Ok(BITMAP_DISPOSED.into());
    };
    other_bitmap_data.check_valid(activation)?;

    // TODO: Given the above check with `other_bitmap_data.check_valid`, this branch will
    //   presumably never get executed.
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
        Some(bitmap_data) => {
            let class = activation.avm2().classes().bitmapdata;
            Ok(BitmapDataObject::from_bitmap_data_internal(
                activation,
                BitmapDataWrapper::new(GcCell::new(activation.context.gc_context, bitmap_data)),
                class,
            )?
            .into())
        }
        None => Ok(EQUIVALENT.into()),
    }
}

/// Implements `BitmapData.pixelDissolve`.
pub fn pixel_dissolve<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        bitmap_data.check_valid(activation)?;

        let src_bitmap_data = args.get_object(activation, 0, "sourceBitmapData")?;

        let source_rect = args.get_object(activation, 1, "sourceRect")?;

        let (src_min_x, src_min_y, src_width, src_height) =
            get_rectangle_x_y_width_height(activation, source_rect)?;

        let dest_point = args.get_object(activation, 2, "destPoint")?;
        let dest_point = (
            dest_point
                .get_public_property("x", activation)?
                .coerce_to_i32(activation)?,
            dest_point
                .get_public_property("y", activation)?
                .coerce_to_i32(activation)?,
        );

        let random_seed = args.get_i32(activation, 3)?;

        let num_pixels = args.get_i32(activation, 4)?;
        if num_pixels < 0 {
            return Err(Error::AvmError(range_error(
                activation,
                &format!("Error #2027: Parameter numPixels must be a non-negative number; got {num_pixels}."),
                2027,
            )?));
        }

        let fill_color = args.get_u32(activation, 5)?;

        // Apparently, if this check fails, a type error for `null` is given.
        if let Some(src_bitmap_data) = src_bitmap_data.as_bitmap_data() {
            src_bitmap_data.check_valid(activation)?;

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

    Ok(Value::Undefined)
}

// Implements `BitmapData.merge`.
pub fn merge<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data() {
        if !bitmap_data.disposed() {
            let src_bitmap = args.get_object(activation, 0, "sourceBitmapData")?;

            let (src_min_x, src_min_y, src_width, src_height) = {
                let source_rect = args.get_object(activation, 1, "sourceRect")?;
                get_rectangle_x_y_width_height(activation, source_rect)?
            };

            let dest_point = {
                let dest_point = args.get_object(activation, 2, "destPoint")?;

                let x = dest_point
                    .get_public_property("x", activation)?
                    .coerce_to_i32(activation)?;

                let y = dest_point
                    .get_public_property("y", activation)?
                    .coerce_to_i32(activation)?;

                (x, y)
            };

            let red_mult = args.get_i32(activation, 3)?;
            let green_mult = args.get_i32(activation, 4)?;
            let blue_mult = args.get_i32(activation, 5)?;
            let alpha_mult = args.get_i32(activation, 6)?;

            if let Some(src_bitmap) = src_bitmap.as_bitmap_data() {
                if !src_bitmap.disposed() {
                    operations::merge(
                        activation.context.gc_context,
                        activation.context.renderer,
                        bitmap_data,
                        src_bitmap,
                        (src_min_x, src_min_y, src_width, src_height),
                        dest_point,
                        (red_mult, green_mult, blue_mult, alpha_mult),
                    );
                }
            }
        }
    }

    Ok(Value::Undefined)
}
