//! flash.filters.BlurFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use gc_arena::{Collect, GcCell, MutationContext};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct BlurFilterObject {
    blur_x: f64,
    blur_y: f64,
    quality: i32,
}

macro_rules! blur_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "blurX" => property(blur_filter_method!(1), blur_filter_method!(2));
    "blurY" => property(blur_filter_method!(3), blur_filter_method!(4));
    "quality" => property(blur_filter_method!(5), blur_filter_method!(6));
};

fn method<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u8,
) -> Result<Value<'gc>, Error<'gc>> {
    const CONSTRUCTOR: u8 = 0;
    const GET_BLUR_X: u8 = 1;
    const SET_BLUR_X: u8 = 2;
    const GET_BLUR_Y: u8 = 3;
    const SET_BLUR_Y: u8 = 4;
    const GET_QUALITY: u8 = 5;
    const SET_QUALITY: u8 = 6;

    if index == CONSTRUCTOR {
        let blur_filter = GcCell::allocate(
            activation.context.gc_context,
            BlurFilterObject {
                blur_x: 4.0,
                blur_y: 4.0,
                quality: 1,
            },
        );
        set_blur_x(activation, blur_filter, args.get(0))?;
        set_blur_y(activation, blur_filter, args.get(1))?;
        set_quality(activation, blur_filter, args.get(2))?;
        this.set_native(
            activation.context.gc_context,
            NativeObject::BlurFilter(blur_filter),
        );
        return Ok(this.into());
    }

    let blur_filter = match this.native() {
        NativeObject::BlurFilter(blur_filter) => blur_filter,
        _ => return Ok(Value::Undefined),
    };

    Ok(match index {
        GET_BLUR_X => blur_filter.read().blur_x.into(),
        SET_BLUR_X => {
            set_blur_x(activation, blur_filter, args.get(0))?;
            Value::Undefined
        }
        GET_BLUR_Y => blur_filter.read().blur_y.into(),
        SET_BLUR_Y => {
            set_blur_y(activation, blur_filter, args.get(0))?;
            Value::Undefined
        }
        GET_QUALITY => blur_filter.read().quality.into(),
        SET_QUALITY => {
            set_quality(activation, blur_filter, args.get(0))?;
            Value::Undefined
        }
        _ => Value::Undefined,
    })
}

fn set_blur_x<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    blur_filter: GcCell<'gc, BlurFilterObject>,
    value: Option<&Value<'gc>>,
) -> Result<(), Error<'gc>> {
    if let Some(value) = value {
        let blur_x = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
        blur_filter.write(activation.context.gc_context).blur_x = blur_x;
    }
    Ok(())
}

fn set_blur_y<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    blur_filter: GcCell<'gc, BlurFilterObject>,
    value: Option<&Value<'gc>>,
) -> Result<(), Error<'gc>> {
    if let Some(value) = value {
        let blur_y = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
        blur_filter.write(activation.context.gc_context).blur_y = blur_y;
    }
    Ok(())
}

fn set_quality<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    blur_filter: GcCell<'gc, BlurFilterObject>,
    value: Option<&Value<'gc>>,
) -> Result<(), Error<'gc>> {
    if let Some(value) = value {
        let quality = value.coerce_to_i32(activation)?.clamp(0, 15);
        blur_filter.write(activation.context.gc_context).quality = quality;
    }
    Ok(())
}

pub fn create_constructor<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let blur_filter_proto = ScriptObject::new(gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, gc_context, blur_filter_proto, fn_proto);
    FunctionObject::constructor(
        gc_context,
        Executable::Native(blur_filter_method!(0)),
        constructor_to_fn!(blur_filter_method!(0)),
        fn_proto,
        blur_filter_proto.into(),
    )
}
