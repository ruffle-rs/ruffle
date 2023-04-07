//! flash.filters.BlurFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use gc_arena::{Collect, MutationContext};
use std::cell::RefCell;

#[derive(Clone, Debug)]
struct BlurFilterData {
    blur_x: f64,
    blur_y: f64,
    quality: i32,
}

impl Default for BlurFilterData {
    fn default() -> Self {
        Self {
            blur_x: 4.0,
            blur_y: 4.0,
            quality: 1,
        }
    }
}

#[derive(Clone, Debug, Default, Collect)]
#[collect(require_static)]
#[repr(transparent)]
pub struct BlurFilter(Box<RefCell<BlurFilterData>>);

impl<'gc> BlurFilter {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let blur_filter = Self::default();
        blur_filter.set_blur_x(activation, args.get(0))?;
        blur_filter.set_blur_y(activation, args.get(1))?;
        blur_filter.set_quality(activation, args.get(2))?;
        Ok(blur_filter)
    }

    fn blur_x(&self) -> f64 {
        self.0.borrow().blur_x
    }

    fn set_blur_x(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_x = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.0.borrow_mut().blur_x = blur_x;
        }
        Ok(())
    }

    fn blur_y(&self) -> f64 {
        self.0.borrow().blur_y
    }

    fn set_blur_y(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_y = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.0.borrow_mut().blur_y = blur_y;
        }
        Ok(())
    }

    fn quality(&self) -> i32 {
        self.0.borrow().quality
    }

    fn set_quality(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let quality = value.coerce_to_i32(activation)?.clamp(0, 15);
            self.0.borrow_mut().quality = quality;
        }
        Ok(())
    }
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
    activation: &mut Activation<'_, 'gc>,
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
        let blur_filter = BlurFilter::new(activation, args)?;
        this.set_native(
            activation.context.gc_context,
            NativeObject::BlurFilter(blur_filter),
        );
        return Ok(this.into());
    }

    let native = this.native();
    let this = match native.as_deref() {
        Some(NativeObject::BlurFilter(blur_filter)) => blur_filter,
        _ => return Ok(Value::Undefined),
    };

    Ok(match index {
        GET_BLUR_X => this.blur_x().into(),
        SET_BLUR_X => {
            this.set_blur_x(activation, args.get(0))?;
            Value::Undefined
        }
        GET_BLUR_Y => this.blur_y().into(),
        SET_BLUR_Y => {
            this.set_blur_y(activation, args.get(0))?;
            Value::Undefined
        }
        GET_QUALITY => this.quality().into(),
        SET_QUALITY => {
            this.set_quality(activation, args.get(0))?;
            Value::Undefined
        }
        _ => Value::Undefined,
    })
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
