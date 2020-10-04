//! flash.display.BitmapData object

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use enumset::EnumSet;
use gc_arena::MutationContext;
use crate::avm1::function::{FunctionObject, Executable};
use crate::display_object::TDisplayObject;
use crate::avm1::object::bitmap_data::BitmapDataObject;

pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    //TODO: check default for this and height
    let width = args.get(0)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_u32(activation)?;

    let height = args.get(1)
        .unwrap_or(&Value::Number(0.0))
        .coerce_to_u32(activation)?;

    let transparent = args.get(2)
        .unwrap_or(&Value::Bool(true))
        .as_bool(activation.current_swf_version());

    //TODO: check types

    //Hmm can't write this in hex
    // 0xFFFFFFFF as f64;
    let fill_color = args.get(3)
        .unwrap_or(&Value::Number(4294967295_f64))
        .coerce_to_u32(activation)?;

    //TODO: respect size limits

    log::warn!("BitmapData constructor w: {}, h: {}, t: {}, fc:{}", width, height, transparent, fill_color);

    //TODO: respect transparency (can we maybe use less memory when disabled?)

    let bitmap_data = this.as_bitmap_data_object().unwrap();
    bitmap_data.init_pixels(activation.context.gc_context, width, height, fill_color);



    Ok(Value::Undefined)
}

pub fn load_bitmap<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
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

pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let mut object = BitmapDataObject::empty_object(gc_context, Some(proto));

    object.into()
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
