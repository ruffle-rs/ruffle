//! flash.filters.DropShadowFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::string::StringContext;
use gc_arena::{Collect, GcCell, Mutation};
use std::ops::Deref;
use swf::{Color, DropShadowFilterFlags, Fixed16, Fixed8};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct DropShadowFilterData {
    distance: f64,
    // TODO: Introduce `Angle<Radians>` struct.
    angle: f64,
    color: Color,
    quality: i32,
    inner: bool,
    knockout: bool,
    blur_x: f64,
    blur_y: f64,
    // TODO: Introduce unsigned `Fixed8`?
    strength: u16,
    hide_object: bool,
}

impl From<&DropShadowFilterData> for swf::DropShadowFilter {
    fn from(filter: &DropShadowFilterData) -> swf::DropShadowFilter {
        let mut flags = DropShadowFilterFlags::empty();
        flags |= DropShadowFilterFlags::from_passes(filter.quality as u8);
        flags.set(DropShadowFilterFlags::KNOCKOUT, filter.knockout);
        flags.set(DropShadowFilterFlags::INNER_SHADOW, filter.inner);
        flags.set(DropShadowFilterFlags::COMPOSITE_SOURCE, !filter.hide_object);
        swf::DropShadowFilter {
            color: filter.color,
            blur_x: Fixed16::from_f64(filter.blur_x),
            blur_y: Fixed16::from_f64(filter.blur_y),
            angle: Fixed16::from_f64(filter.angle),
            distance: Fixed16::from_f64(filter.distance),
            strength: Fixed8::from_f64(filter.strength()),
            flags,
        }
    }
}

impl From<swf::DropShadowFilter> for DropShadowFilterData {
    fn from(filter: swf::DropShadowFilter) -> DropShadowFilterData {
        Self {
            distance: filter.distance.into(),
            angle: filter.angle.into(),
            color: filter.color,
            quality: filter.num_passes().into(),
            strength: (filter.strength.to_f64() * 256.0) as u16,
            knockout: filter.is_knockout(),
            blur_x: filter.blur_x.into(),
            blur_y: filter.blur_y.into(),
            inner: filter.is_inner(),
            hide_object: filter.hide_object(),
        }
    }
}

impl Default for DropShadowFilterData {
    #[allow(clippy::approx_constant)]
    fn default() -> Self {
        Self {
            distance: 4.0,
            angle: 0.785398163, // ~45 degrees
            color: Color::BLACK,
            quality: 1,
            inner: false,
            knockout: false,
            blur_x: 4.0,
            blur_y: 4.0,
            strength: 1 << 8,
            hide_object: false,
        }
    }
}

impl DropShadowFilterData {
    pub fn strength(&self) -> f64 {
        f64::from(self.strength) / 256.0
    }

    pub fn set_strength(&mut self, strength: f64) {
        self.strength = ((strength * 256.0) as u16).clamp(0, 0xFF00)
    }
}

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
#[repr(transparent)]
pub struct DropShadowFilter<'gc>(GcCell<'gc, DropShadowFilterData>);

impl<'gc> DropShadowFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let drop_shadow_filter = Self(GcCell::new(
            activation.context.gc_context,
            Default::default(),
        ));
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
        Self(GcCell::new(gc_context, filter.into()))
    }

    pub(crate) fn duplicate(&self, gc_context: &Mutation<'gc>) -> Self {
        Self(GcCell::new(gc_context, self.0.read().clone()))
    }

    fn distance(&self) -> f64 {
        self.0.read().distance
    }

    fn set_distance(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let distance = value.coerce_to_f64(activation)?;
            self.0.write(activation.context.gc_context).distance = distance;
        }
        Ok(())
    }

    fn angle(&self) -> f64 {
        self.0.read().angle.to_degrees()
    }

    fn set_angle(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let angle = (value.coerce_to_f64(activation)? % 360.0).to_radians();
            self.0.write(activation.context.gc_context).angle = angle;
        }
        Ok(())
    }

    fn color(&self) -> i32 {
        self.0.read().color.to_rgb() as i32
    }

    fn set_color(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let value = value.coerce_to_u32(activation)?;
            let mut write = self.0.write(activation.context.gc_context);
            write.color = Color::from_rgb(value, write.color.a);
        }
        Ok(())
    }

    fn alpha(&self) -> f64 {
        f64::from(self.0.read().color.a) / 255.0
    }

    fn set_alpha(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let alpha = (value.coerce_to_f64(activation)? * 255.0) as u8;
            self.0.write(activation.context.gc_context).color.a = alpha;
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

    fn inner(&self) -> bool {
        self.0.read().inner
    }

    fn set_inner(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let inner = value.as_bool(activation.swf_version());
            self.0.write(activation.context.gc_context).inner = inner;
        }
        Ok(())
    }

    fn knockout(&self) -> bool {
        self.0.read().knockout
    }

    fn set_knockout(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let knockout = value.as_bool(activation.swf_version());
            self.0.write(activation.context.gc_context).knockout = knockout;
        }
        Ok(())
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

    fn strength(&self) -> f64 {
        self.0.read().strength()
    }

    fn set_strength(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            self.0
                .write(activation.context.gc_context)
                .set_strength(value.coerce_to_f64(activation)?);
        }
        Ok(())
    }

    fn hide_object(&self) -> bool {
        self.0.read().hide_object
    }

    fn set_hide_object(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let hide_object = value.as_bool(activation.swf_version());
            self.0.write(activation.context.gc_context).hide_object = hide_object;
        }
        Ok(())
    }

    pub fn filter(&self) -> swf::DropShadowFilter {
        self.0.read().deref().into()
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
            activation.context.gc_context,
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
    let drop_shadow_filter_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, drop_shadow_filter_proto, fn_proto);
    drop_shadow_filter_proto.into()
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(drop_shadow_filter_method!(0)),
        constructor_to_fn!(drop_shadow_filter_method!(0)),
        fn_proto,
        proto,
    )
}
