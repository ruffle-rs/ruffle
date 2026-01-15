//! flash.filters.BlurFilter object
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{Activation, Error, Object, Value};
use gc_arena::{Collect, Gc, Mutation};
use std::cell::Cell;
use swf::{BlurFilterFlags, Fixed16};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct BlurFilterData {
    blur_x: Cell<f64>,
    blur_y: Cell<f64>,
    quality: Cell<i32>,
}

impl Default for BlurFilterData {
    fn default() -> Self {
        Self {
            blur_x: Cell::new(4.0),
            blur_y: Cell::new(4.0),
            quality: Cell::new(1),
        }
    }
}

impl From<&BlurFilterData> for swf::BlurFilter {
    fn from(filter: &BlurFilterData) -> swf::BlurFilter {
        swf::BlurFilter {
            blur_x: Fixed16::from_f64(filter.blur_x.get()),
            blur_y: Fixed16::from_f64(filter.blur_y.get()),
            flags: BlurFilterFlags::from_passes(filter.quality.get() as u8),
        }
    }
}

impl From<swf::BlurFilter> for BlurFilterData {
    fn from(filter: swf::BlurFilter) -> BlurFilterData {
        Self {
            quality: Cell::new(filter.num_passes().into()),
            blur_x: Cell::new(filter.blur_x.into()),
            blur_y: Cell::new(filter.blur_y.into()),
        }
    }
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
#[repr(transparent)]
pub struct BlurFilter<'gc>(Gc<'gc, BlurFilterData>);

impl<'gc> BlurFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let blur_filter = Self(Gc::new(activation.gc(), Default::default()));
        blur_filter.set_blur_x(activation, args.get(0))?;
        blur_filter.set_blur_y(activation, args.get(1))?;
        blur_filter.set_quality(activation, args.get(2))?;
        Ok(blur_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::BlurFilter) -> Self {
        Self(Gc::new(gc_context, filter.into()))
    }

    pub(crate) fn duplicate(self, gc_context: &Mutation<'gc>) -> Self {
        Self(Gc::new(gc_context, self.0.as_ref().clone()))
    }

    fn blur_x(self) -> f64 {
        self.0.blur_x.get()
    }

    fn set_blur_x(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_x = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.0.blur_x.set(blur_x);
        }
        Ok(())
    }

    fn blur_y(self) -> f64 {
        self.0.blur_y.get()
    }

    fn set_blur_y(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_y = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.0.blur_y.set(blur_y);
        }
        Ok(())
    }

    fn quality(self) -> i32 {
        self.0.quality.get()
    }

    fn set_quality(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let quality = value.coerce_to_i32(activation)?.clamp(0, 15);
            self.0.quality.set(quality);
        }
        Ok(())
    }

    pub fn filter(self) -> swf::BlurFilter {
        self.0.as_ref().into()
    }
}

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    use fn method;
    "blurX" => property(GET_BLUR_X, SET_BLUR_X; VERSION_8);
    "blurY" => property(GET_BLUR_Y, SET_BLUR_Y; VERSION_8);
    "quality" => property(GET_QUALITY, SET_QUALITY; VERSION_8);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(table_constructor!(method), None, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    class
}

pub mod method {
    pub const CONSTRUCTOR: u16 = 0;
    pub const GET_BLUR_X: u16 = 1;
    pub const SET_BLUR_X: u16 = 2;
    pub const GET_BLUR_Y: u16 = 3;
    pub const SET_BLUR_Y: u16 = 4;
    pub const GET_QUALITY: u16 = 5;
    pub const SET_QUALITY: u16 = 6;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;

    if index == CONSTRUCTOR {
        let blur_filter = BlurFilter::new(activation, args)?;
        this.set_native(activation.gc(), NativeObject::BlurFilter(blur_filter));
        return Ok(this.into());
    }

    let NativeObject::BlurFilter(this) = this.native() else {
        return Ok(Value::Undefined);
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
