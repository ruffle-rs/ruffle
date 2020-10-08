//! flash.display.BitmapData object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::bitmap_data::BitmapDataObject;
use crate::avm1::{Object, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;



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

    if width > 2880 || height > 2880 || width <= 0 || height <= 0 {
        log::warn!("Invalid BitmapData size {}x{}", width, height);
        return Ok(Value::Undefined);
    }

    let transparency = args
        .get(2)
        .unwrap_or(&Value::Bool(true))
        .as_bool(activation.current_swf_version());

    //Hmm can't write this in hex
    // 0xFFFFFFFF as f64;
    let fill_color = args
        .get(3)
        .unwrap_or(&Value::Number(4294967295_f64))
        .coerce_to_i32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.init_pixels(activation.context.gc_context, width, height, fill_color);
    bitmap_data.set_transparency(activation.context.gc_context, transparency);

    Ok(Value::Undefined)
}

pub fn load_bitmap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    //TODO: how does this handle no args
    let name = args
        .get(0)
        .unwrap_or(&Value::Undefined)
        .coerce_to_string(activation)?;

    log::warn!("BitmapData.loadBitmap({:?}), not impl", name);

    //TODO: correct method here? also rename
    let mc = activation.context.swf;

    //TODO: unwrap
    // activation.context.library.library_for_movie(mc.clone()).expect("Unable to get library for movie").instantiate_by_export_name(name.as_str(), activation.context.gc_context).expect("Unable to instantiate");

    //TODO: write tests for props on the return val of this, for now we will just stub

    let proto = activation.context.system_prototypes.bitmap_data_constructor;
    let new_bitmap = proto.construct(activation, &[])?;

    Ok(new_bitmap.into())
}

pub fn get_height<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    if bitmap_data.get_disposed() {
        return Ok((-1).into())
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
        return Ok((-1).into())
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
        return Ok((-1).into())
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
        return Ok((-1).into())
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
        return Ok((-1).into())
    }

    let x = args
        .get(0)
        .and_then(|x| x.coerce_to_i32(activation).ok());
    let y = args
        .get(1)
        .and_then(|x| x.coerce_to_i32(activation).ok());

    if let Some((x, y)) = x.zip(y) {
        Ok(bitmap_data.get_pixel(x, y).into())
    } else {
        Ok((-1).into())
    }
}

//TODO: out of bounds / missing args / neg args
pub fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args
        .get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args
        .get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let color = args
        .get(2)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_i32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.set_pixel(activation.context.gc_context, x, y, color.into());

    Ok(Value::Undefined)
}

//TODO: out of bounds / missing args / neg args
pub fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args
        .get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args
        .get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let color = args
        .get(2)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_i32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.set_pixel32(activation.context.gc_context, x, y, color.into());

    Ok(Value::Undefined)
}

//TODO: out of bounds / missing args / neg args
pub fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args
        .get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args
        .get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    //TODO: move this unwrap to the object

    let x: i32 = bitmap_data.get_pixel32(x, y).unwrap_or(0.into()).into();

    Ok(x.into())
}

//TODO: missing args / out of bounds
pub fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
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
        let bitmap_data = this.as_bitmap_data_object().unwrap();
        bitmap_data.copy_channel(
            activation.context.gc_context,
            source_bitmap,
            source_channel as u8,
            dest_channel as u8,
        );
    }

    Ok(Value::Undefined)
}

//TODO: missing args / out of bounds
pub fn fill_rect<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let rectangle = args
        .get(0)
        //TODO:
        .unwrap()
        .coerce_to_object(activation);

    let color = args
        .get(1)
        //TODO:
        .unwrap()
        .coerce_to_i32(activation)?;

    //TODO: does this only work on actual rectangles or does it work on anything with x/y/w/h
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

    Ok(Value::Undefined)
}

pub fn clone<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(bitmap_data) = this.as_bitmap_data_object() {
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
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.dispose(activation.context.gc_context);
    Ok(Value::Undefined)
}

pub fn flood_fill<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args
        .get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args
        .get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let color = args
        .get(2)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_i32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.flood_fill(activation.context.gc_context, x, y, color.into());
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
    object.force_set_function("dispose", dispose, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("floodFill", flood_fill, gc_context, EnumSet::empty(), Some(fn_proto));



    bitmap_data_object.into()
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
