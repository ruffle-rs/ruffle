//! flash.display.BitmapData object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;
use crate::avm1::function::{FunctionObject, Executable};
use crate::avm1::object::bitmap_data::BitmapDataObject;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {

    //TODO: if either width or height is missing then the constructor should fail

    let width = args.get(0)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_u32(activation)?;

    let height = args.get(1)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_u32(activation)?;

    if width > 2880 || height > 2880 || width <= 0 || height <= 0{
        log::warn!("Invalid BitmapData size {}x{}", width, height);
        return Ok(Value::Undefined)
    }

    let transparency = args.get(2)
        .unwrap_or(&Value::Bool(true))
        .as_bool(activation.current_swf_version());

    //TODO: check types

    //Hmm can't write this in hex
    // 0xFFFFFFFF as f64;
    let fill_color = args.get(3)
        .unwrap_or(&Value::Number(4294967295_f64))
        .coerce_to_i32(activation)?;

    //TODO: respect transparency (can we maybe use less memory when disabled?)

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
    let name = args.get(0)
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
    Ok(this.as_bitmap_data_object().unwrap().get_height().into())
}

pub fn get_width<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bitmap_data_object().unwrap().get_width().into())
}

pub fn get_transparent<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    Ok(this.as_bitmap_data_object().unwrap().get_transparency().into())
}

pub fn get_rectangle<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let bitmap_data = this.as_bitmap_data_object().unwrap();

    let proto = activation.context.system_prototypes.rectangle_constructor;
    let rect = proto.construct(activation, &[0.into(), 0.into(), bitmap_data.get_width().into(), bitmap_data.get_height().into()])?;
    Ok(rect.into())
}

//TODO: out of bounds / missing args / neg args
pub fn get_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args.get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args.get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    //TODO: move this unwrap to the object
    Ok(bitmap_data.get_pixel(x, y).unwrap_or(0).into())
}

//TODO: out of bounds / missing args / neg args
pub fn set_pixel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args.get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args.get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let color = args.get(2)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_i32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.set_pixel(activation.context.gc_context, x, y, color);

    Ok(Value::Undefined)
}

//TODO: out of bounds / missing args / neg args
pub fn set_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args.get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args.get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let color = args.get(2)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_i32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.set_pixel32(activation.context.gc_context, x, y, color);

    Ok(Value::Undefined)
}

//TODO: out of bounds / missing args / neg args
pub fn get_pixel32<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let x = args.get(0)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let y = args.get(1)
        .unwrap_or(&Value::Number(0_f64))
        .coerce_to_u32(activation)?;

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    //TODO: move this unwrap to the object

    let x = bitmap_data.get_pixel32(x, y).unwrap_or(0);


    Ok(x.into())
}

//TODO: missing args / out of bounds
pub fn copy_channel<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let source_bitmap = args.get(0)
        .unwrap_or(&Value::Undefined)
        //TODO: unwrap
        .coerce_to_object(activation);

    let source_rect = args.get(1)
        .unwrap_or(&Value::Undefined)
        //TODO: unwrap
        .coerce_to_object(activation);

    let dest_point = args.get(2)
        .unwrap_or(&Value::Undefined)
        //TODO: unwrap
        .coerce_to_object(activation);

    let source_channel = args.get(3)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    let dest_channel = args.get(4)
        .unwrap_or(&Value::Undefined)
        .coerce_to_i32(activation)?;

    if let Some(source_bitmap) = source_bitmap.as_bitmap_data_object() {
        let bitmap_data = this.as_bitmap_data_object().unwrap();
        bitmap_data.copy_channel(activation.context.gc_context, source_bitmap, source_channel as u8, dest_channel as u8);
    }

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

    object.force_set_function("getPixel", get_pixel, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("getPixel32", get_pixel32, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("setPixel", set_pixel, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("setPixel32", set_pixel32, gc_context, EnumSet::empty(), Some(fn_proto));
    object.force_set_function("copyChannel", copy_channel, gc_context, EnumSet::empty(), Some(fn_proto));



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

    script_object.force_set_function("loadBitmap", load_bitmap, gc_context, EnumSet::empty(), fn_proto);

    object
}
