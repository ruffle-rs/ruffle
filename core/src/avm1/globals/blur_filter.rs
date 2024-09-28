//! flash.filters.BlurFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::string::StringContext;
use gc_arena::{Collect, GcCell, Mutation};
use std::ops::Deref;
use swf::{BlurFilterFlags, Fixed16};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
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

impl From<&BlurFilterData> for swf::BlurFilter {
    fn from(filter: &BlurFilterData) -> swf::BlurFilter {
        swf::BlurFilter {
            blur_x: Fixed16::from_f64(filter.blur_x),
            blur_y: Fixed16::from_f64(filter.blur_y),
            flags: BlurFilterFlags::from_passes(filter.quality as u8),
        }
    }
}

impl From<swf::BlurFilter> for BlurFilterData {
    fn from(filter: swf::BlurFilter) -> BlurFilterData {
        Self {
            quality: filter.num_passes().into(),
            blur_x: filter.blur_x.into(),
            blur_y: filter.blur_y.into(),
        }
    }
}

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
#[repr(transparent)]
pub struct BlurFilter<'gc>(GcCell<'gc, BlurFilterData>);

impl<'gc> BlurFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let blur_filter = Self(GcCell::new(
            activation.context.gc_context,
            Default::default(),
        ));
        blur_filter.set_blur_x(activation, args.get(0))?;
        blur_filter.set_blur_y(activation, args.get(1))?;
        blur_filter.set_quality(activation, args.get(2))?;
        Ok(blur_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::BlurFilter) -> Self {
        Self(GcCell::new(gc_context, filter.into()))
    }

    pub(crate) fn duplicate(&self, gc_context: &Mutation<'gc>) -> Self {
        Self(GcCell::new(gc_context, self.0.read().clone()))
    }

    fn blur_x(&self) -> f64 {
        self.0.read().blur_x
    }

    fn set_blur_x(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_x = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.0.write(activation.context.gc_context).blur_x = blur_x;
        }
        Ok(())
    }

    fn blur_y(&self) -> f64 {
        self.0.read().blur_y
    }

    fn set_blur_y(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_y = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.0.write(activation.context.gc_context).blur_y = blur_y;
        }
        Ok(())
    }

    fn quality(&self) -> i32 {
        self.0.read().quality
    }

    fn set_quality(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let quality = value.coerce_to_i32(activation)?.clamp(0, 15);
            self.0.write(activation.context.gc_context).quality = quality;
        }
        Ok(())
    }

    pub fn filter(&self) -> swf::BlurFilter {
        self.0.read().deref().into()
    }
}

macro_rules! blur_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "blurX" => property(blur_filter_method!(1), blur_filter_method!(2); VERSION_8);
    "blurY" => property(blur_filter_method!(3), blur_filter_method!(4); VERSION_8);
    "quality" => property(blur_filter_method!(5), blur_filter_method!(6); VERSION_8);
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

    let this = match this.native() {
        NativeObject::BlurFilter(blur_filter) => blur_filter,
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

pub fn create_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let blur_filter_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, blur_filter_proto, fn_proto);
    blur_filter_proto.into()
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(blur_filter_method!(0)),
        constructor_to_fn!(blur_filter_method!(0)),
        fn_proto,
        proto,
    )
}
