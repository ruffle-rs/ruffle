//! flash.filters.DropShadowFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::string::StringContext;
use gc_arena::{Collect, Gc, Mutation};
use std::cell::Cell;
use swf::{Color, DropShadowFilterFlags, Fixed16, Fixed8};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct DropShadowFilterData {
    distance: Cell<f64>,
    // TODO: Introduce `Angle<Radians>` struct.
    angle: Cell<f64>,
    color: Cell<Color>,
    quality: Cell<i32>,
    inner: Cell<bool>,
    knockout: Cell<bool>,
    blur_x: Cell<f64>,
    blur_y: Cell<f64>,
    // TODO: Introduce unsigned `Fixed8`?
    strength: Cell<u16>,
    hide_object: Cell<bool>,
}

impl From<&DropShadowFilterData> for swf::DropShadowFilter {
    fn from(filter: &DropShadowFilterData) -> swf::DropShadowFilter {
        let mut flags = DropShadowFilterFlags::empty();
        flags |= DropShadowFilterFlags::from_passes(filter.quality.get() as u8);
        flags.set(DropShadowFilterFlags::KNOCKOUT, filter.knockout.get());
        flags.set(DropShadowFilterFlags::INNER_SHADOW, filter.inner.get());
        flags.set(
            DropShadowFilterFlags::COMPOSITE_SOURCE,
            !filter.hide_object.get(),
        );
        swf::DropShadowFilter {
            color: filter.color.get(),
            blur_x: Fixed16::from_f64(filter.blur_x.get()),
            blur_y: Fixed16::from_f64(filter.blur_y.get()),
            angle: Fixed16::from_f64(filter.angle.get()),
            distance: Fixed16::from_f64(filter.distance.get()),
            strength: Fixed8::from_f64(filter.strength()),
            flags,
        }
    }
}

impl From<swf::DropShadowFilter> for DropShadowFilterData {
    fn from(filter: swf::DropShadowFilter) -> DropShadowFilterData {
        Self {
            distance: Cell::new(filter.distance.into()),
            angle: Cell::new(filter.angle.into()),
            color: Cell::new(filter.color),
            quality: Cell::new(filter.num_passes().into()),
            strength: Cell::new((filter.strength.to_f64() * 256.0) as u16),
            knockout: Cell::new(filter.is_knockout()),
            blur_x: Cell::new(filter.blur_x.into()),
            blur_y: Cell::new(filter.blur_y.into()),
            inner: Cell::new(filter.is_inner()),
            hide_object: Cell::new(filter.hide_object()),
        }
    }
}

impl Default for DropShadowFilterData {
    #[allow(clippy::approx_constant)]
    fn default() -> Self {
        Self {
            distance: Cell::new(4.0),
            angle: Cell::new(0.785398163), // ~45 degrees
            color: Cell::new(Color::BLACK),
            quality: Cell::new(1),
            inner: Cell::new(false),
            knockout: Cell::new(false),
            blur_x: Cell::new(4.0),
            blur_y: Cell::new(4.0),
            strength: Cell::new(1 << 8),
            hide_object: Cell::new(false),
        }
    }
}

impl DropShadowFilterData {
    pub fn strength(&self) -> f64 {
        f64::from(self.strength.get()) / 256.0
    }

    pub fn set_strength(&self, strength: f64) {
        let strength = ((strength * 256.0) as u16).clamp(0, 0xFF00);
        self.strength.set(strength);
    }
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
#[repr(transparent)]
pub struct DropShadowFilter<'gc>(Gc<'gc, DropShadowFilterData>);

impl<'gc> DropShadowFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let drop_shadow_filter = Self(Gc::new(activation.gc(), Default::default()));
        drop_shadow_filter.set_distance(activation, args.get(0))?;
        drop_shadow_filter.set_angle(activation, args.get(1))?;
        drop_shadow_filter.set_color(activation, args.get(2))?;
        drop_shadow_filter.set_alpha(activation, args.get(3))?;
        drop_shadow_filter.set_blur_x(activation, args.get(4))?;
        drop_shadow_filter.set_blur_y(activation, args.get(5))?;
        drop_shadow_filter.set_strength(activation, args.get(6))?;
        drop_shadow_filter.set_quality(activation, args.get(7))?;
        drop_shadow_filter.set_inner(activation, args.get(8))?;
        drop_shadow_filter.set_knockout(activation, args.get(9))?;
        drop_shadow_filter.set_hide_object(activation, args.get(10))?;
        Ok(drop_shadow_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::DropShadowFilter) -> Self {
        Self(Gc::new(gc_context, filter.into()))
    }

    pub(crate) fn duplicate(self, gc_context: &Mutation<'gc>) -> Self {
        Self(Gc::new(gc_context, self.0.as_ref().clone()))
    }

    fn distance(self) -> f64 {
        self.0.distance.get()
    }

    fn set_distance(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let distance = value.coerce_to_f64(activation)?;
            self.0.distance.set(distance);
        }
        Ok(())
    }

    fn angle(self) -> f64 {
        self.0.angle.get().to_degrees()
    }

    fn set_angle(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let angle = (value.coerce_to_f64(activation)? % 360.0).to_radians();
            self.0.angle.set(angle);
        }
        Ok(())
    }

    fn color(self) -> i32 {
        self.0.color.get().to_rgb() as i32
    }

    fn set_color(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let value = value.coerce_to_u32(activation)?;
            let color = self.0.color.get();
            self.0.color.set(Color::from_rgb(value, color.a));
        }
        Ok(())
    }

    fn alpha(self) -> f64 {
        f64::from(self.0.color.get().a) / 255.0
    }

    fn set_alpha(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let alpha = (value.coerce_to_f64(activation)? * 255.0) as u8;
            let mut color = self.0.color.get();
            color.a = alpha;
            self.0.color.set(color);
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

    fn inner(self) -> bool {
        self.0.inner.get()
    }

    fn set_inner(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let inner = value.as_bool(activation.swf_version());
            self.0.inner.set(inner);
        }
        Ok(())
    }

    fn knockout(self) -> bool {
        self.0.knockout.get()
    }

    fn set_knockout(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let knockout = value.as_bool(activation.swf_version());
            self.0.knockout.set(knockout);
        }
        Ok(())
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

    fn strength(self) -> f64 {
        self.0.strength()
    }

    fn set_strength(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            self.0.set_strength(value.coerce_to_f64(activation)?);
        }
        Ok(())
    }

    fn hide_object(self) -> bool {
        self.0.hide_object.get()
    }

    fn set_hide_object(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let hide_object = value.as_bool(activation.swf_version());
            self.0.hide_object.set(hide_object);
        }
        Ok(())
    }

    pub fn filter(self) -> swf::DropShadowFilter {
        self.0.as_ref().into()
    }
}

macro_rules! drop_shadow_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "distance" => property(drop_shadow_filter_method!(1), drop_shadow_filter_method!(2); VERSION_8);
    "angle" => property(drop_shadow_filter_method!(3), drop_shadow_filter_method!(4); VERSION_8);
    "color" => property(drop_shadow_filter_method!(5), drop_shadow_filter_method!(6); VERSION_8);
    "alpha" => property(drop_shadow_filter_method!(7), drop_shadow_filter_method!(8); VERSION_8);
    "quality" => property(drop_shadow_filter_method!(9), drop_shadow_filter_method!(10); VERSION_8);
    "inner" => property(drop_shadow_filter_method!(11), drop_shadow_filter_method!(12); VERSION_8);
    "knockout" => property(drop_shadow_filter_method!(13), drop_shadow_filter_method!(14); VERSION_8);
    "blurX" => property(drop_shadow_filter_method!(15), drop_shadow_filter_method!(16); VERSION_8);
    "blurY" => property(drop_shadow_filter_method!(17), drop_shadow_filter_method!(18); VERSION_8);
    "strength" => property(drop_shadow_filter_method!(19), drop_shadow_filter_method!(20); VERSION_8);
    "hideObject" => property(drop_shadow_filter_method!(21), drop_shadow_filter_method!(22); VERSION_8);
};

fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u8,
) -> Result<Value<'gc>, Error<'gc>> {
    const CONSTRUCTOR: u8 = 0;
    const GET_DISTANCE: u8 = 1;
    const SET_DISTANCE: u8 = 2;
    const GET_ANGLE: u8 = 3;
    const SET_ANGLE: u8 = 4;
    const GET_COLOR: u8 = 5;
    const SET_COLOR: u8 = 6;
    const GET_ALPHA: u8 = 7;
    const SET_ALPHA: u8 = 8;
    const GET_QUALITY: u8 = 9;
    const SET_QUALITY: u8 = 10;
    const GET_INNER: u8 = 11;
    const SET_INNER: u8 = 12;
    const GET_KNOCKOUT: u8 = 13;
    const SET_KNOCKOUT: u8 = 14;
    const GET_BLUR_X: u8 = 15;
    const SET_BLUR_X: u8 = 16;
    const GET_BLUR_Y: u8 = 17;
    const SET_BLUR_Y: u8 = 18;
    const GET_STRENGTH: u8 = 19;
    const SET_STRENGTH: u8 = 20;
    const GET_HIDE_OBJECT: u8 = 21;
    const SET_HIDE_OBJECT: u8 = 22;

    if index == CONSTRUCTOR {
        let drop_shadow_filter = DropShadowFilter::new(activation, args)?;
        this.set_native(
            activation.gc(),
            NativeObject::DropShadowFilter(drop_shadow_filter),
        );
        return Ok(this.into());
    }

    let this = match this.native() {
        NativeObject::DropShadowFilter(drop_shadow_filter) => drop_shadow_filter,
        _ => return Ok(Value::Undefined),
    };

    Ok(match index {
        GET_DISTANCE => this.distance().into(),
        SET_DISTANCE => {
            this.set_distance(activation, args.get(0))?;
            Value::Undefined
        }
        GET_ANGLE => this.angle().into(),
        SET_ANGLE => {
            this.set_angle(activation, args.get(0))?;
            Value::Undefined
        }
        GET_COLOR => this.color().into(),
        SET_COLOR => {
            this.set_color(activation, args.get(0))?;
            Value::Undefined
        }
        GET_ALPHA => this.alpha().into(),
        SET_ALPHA => {
            this.set_alpha(activation, args.get(0))?;
            Value::Undefined
        }
        GET_QUALITY => this.quality().into(),
        SET_QUALITY => {
            this.set_quality(activation, args.get(0))?;
            Value::Undefined
        }
        GET_INNER => this.inner().into(),
        SET_INNER => {
            this.set_inner(activation, args.get(0))?;
            Value::Undefined
        }
        GET_KNOCKOUT => this.knockout().into(),
        SET_KNOCKOUT => {
            this.set_knockout(activation, args.get(0))?;
            Value::Undefined
        }
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
        GET_STRENGTH => this.strength().into(),
        SET_STRENGTH => {
            this.set_strength(activation, args.get(0))?;
            Value::Undefined
        }
        GET_HIDE_OBJECT => this.hide_object().into(),
        SET_HIDE_OBJECT => {
            this.set_hide_object(activation, args.get(0))?;
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
    let drop_shadow_filter_proto = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, drop_shadow_filter_proto, fn_proto);
    drop_shadow_filter_proto.into()
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context,
        Executable::Native(drop_shadow_filter_method!(0)),
        constructor_to_fn!(drop_shadow_filter_method!(0)),
        fn_proto,
        proto,
    )
}
