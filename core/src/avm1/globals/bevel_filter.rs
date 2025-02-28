//! flash.filters.BevelFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::string::StringContext;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_macros::istr;
use std::cell::Cell;
use swf::{BevelFilterFlags, Color, Fixed16, Fixed8, GradientFilterFlags};

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum BevelFilterType {
    Inner,
    Outer,
    Full,
}

impl BevelFilterType {
    pub fn as_bevel_flags(self) -> BevelFilterFlags {
        match self {
            BevelFilterType::Inner => BevelFilterFlags::INNER_SHADOW,
            BevelFilterType::Outer => BevelFilterFlags::empty(),
            BevelFilterType::Full => BevelFilterFlags::ON_TOP,
        }
    }

    pub fn as_gradient_flags(self) -> GradientFilterFlags {
        match self {
            BevelFilterType::Inner => GradientFilterFlags::INNER_SHADOW,
            BevelFilterType::Outer => GradientFilterFlags::empty(),
            BevelFilterType::Full => GradientFilterFlags::ON_TOP,
        }
    }
}

impl From<BevelFilterFlags> for BevelFilterType {
    fn from(value: BevelFilterFlags) -> Self {
        if value.contains(BevelFilterFlags::ON_TOP) {
            BevelFilterType::Full
        } else if value.contains(BevelFilterFlags::INNER_SHADOW) {
            BevelFilterType::Inner
        } else {
            BevelFilterType::Outer
        }
    }
}

impl From<GradientFilterFlags> for BevelFilterType {
    fn from(value: GradientFilterFlags) -> Self {
        if value.contains(GradientFilterFlags::ON_TOP) {
            BevelFilterType::Full
        } else if value.contains(GradientFilterFlags::INNER_SHADOW) {
            BevelFilterType::Inner
        } else {
            BevelFilterType::Outer
        }
    }
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct BevelFilterData {
    distance: Cell<f64>,
    // TODO: Introduce `Angle<Radians>` struct.
    angle: Cell<f64>,
    highlight: Cell<Color>,
    shadow: Cell<Color>,
    quality: Cell<i32>,
    // TODO: Introduce unsigned `Fixed8`?
    strength: Cell<u16>,
    knockout: Cell<bool>,
    blur_x: Cell<f64>,
    blur_y: Cell<f64>,
    type_: Cell<BevelFilterType>,
}

impl From<&BevelFilterData> for swf::BevelFilter {
    fn from(filter: &BevelFilterData) -> swf::BevelFilter {
        let mut flags = BevelFilterFlags::COMPOSITE_SOURCE;
        flags |= BevelFilterFlags::from_passes(filter.quality.get() as u8);
        flags |= filter.type_.get().as_bevel_flags();
        flags.set(BevelFilterFlags::KNOCKOUT, filter.knockout.get());
        swf::BevelFilter {
            shadow_color: filter.shadow.get(),
            highlight_color: filter.highlight.get(),
            blur_x: Fixed16::from_f64(filter.blur_x.get()),
            blur_y: Fixed16::from_f64(filter.blur_y.get()),
            angle: Fixed16::from_f64(filter.angle.get()),
            distance: Fixed16::from_f64(filter.distance.get()),
            strength: Fixed8::from_f64(filter.strength()),
            flags,
        }
    }
}

impl From<swf::BevelFilter> for BevelFilterData {
    fn from(filter: swf::BevelFilter) -> BevelFilterData {
        Self {
            distance: Cell::new(filter.distance.into()),
            angle: Cell::new(filter.angle.into()),
            highlight: Cell::new(filter.highlight_color),
            shadow: Cell::new(filter.shadow_color),
            quality: Cell::new(filter.num_passes().into()),
            strength: Cell::new((filter.strength.to_f64() * 256.0) as u16),
            knockout: Cell::new(filter.is_knockout()),
            blur_x: Cell::new(filter.blur_x.into()),
            blur_y: Cell::new(filter.blur_y.into()),
            type_: Cell::new(filter.flags.into()),
        }
    }
}

impl Default for BevelFilterData {
    #[allow(clippy::approx_constant)]
    fn default() -> Self {
        Self {
            distance: Cell::new(4.0),
            angle: Cell::new(0.785398163), // ~45 degrees
            highlight: Cell::new(Color::WHITE),
            shadow: Cell::new(Color::BLACK),
            quality: Cell::new(1),
            strength: Cell::new(1 << 8),
            knockout: Cell::new(false),
            blur_x: Cell::new(4.0),
            blur_y: Cell::new(4.0),
            type_: Cell::new(BevelFilterType::Inner),
        }
    }
}

impl BevelFilterData {
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
pub struct BevelFilter<'gc>(Gc<'gc, BevelFilterData>);

impl<'gc> BevelFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let bevel_filter = Self(Gc::new(activation.gc(), Default::default()));
        bevel_filter.set_distance(activation, args.get(0))?;
        bevel_filter.set_angle(activation, args.get(1))?;
        bevel_filter.set_highlight_color(activation, args.get(2))?;
        bevel_filter.set_highlight_alpha(activation, args.get(3))?;
        bevel_filter.set_shadow_color(activation, args.get(4))?;
        bevel_filter.set_shadow_alpha(activation, args.get(5))?;
        bevel_filter.set_blur_x(activation, args.get(6))?;
        bevel_filter.set_blur_y(activation, args.get(7))?;
        bevel_filter.set_strength(activation, args.get(8))?;
        bevel_filter.set_quality(activation, args.get(9))?;
        bevel_filter.set_type(activation, args.get(10))?;
        bevel_filter.set_knockout(activation, args.get(11))?;
        Ok(bevel_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::BevelFilter) -> Self {
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

    fn highlight_color(self) -> i32 {
        self.0.highlight.get().to_rgb() as i32
    }

    fn set_highlight_color(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let value = value.coerce_to_u32(activation)?;
            let highlight = self.0.highlight.get();
            self.0.highlight.set(Color::from_rgb(value, highlight.a));
        }
        Ok(())
    }

    fn highlight_alpha(self) -> f64 {
        f64::from(self.0.highlight.get().a) / 255.0
    }

    fn set_highlight_alpha(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let alpha = (value.coerce_to_f64(activation)? * 255.0) as u8;
            let mut highlight = self.0.highlight.get();
            highlight.a = alpha;
            self.0.highlight.set(highlight);
        }
        Ok(())
    }

    fn shadow_color(self) -> i32 {
        self.0.shadow.get().to_rgb() as i32
    }

    fn set_shadow_color(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let value = value.coerce_to_u32(activation)?;
            let shadow = self.0.shadow.get();
            self.0.shadow.set(Color::from_rgb(value, shadow.a));
        }
        Ok(())
    }

    fn shadow_alpha(self) -> f64 {
        f64::from(self.0.shadow.get().a) / 255.0
    }

    fn set_shadow_alpha(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let alpha = (value.coerce_to_f64(activation)? * 255.0) as u8;
            let mut shadow = self.0.shadow.get();
            shadow.a = alpha;
            self.0.shadow.set(shadow);
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

    fn type_(self) -> BevelFilterType {
        self.0.type_.get()
    }

    fn set_type(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let type_ = value.coerce_to_string(activation)?;

            let type_ = if &type_ == b"inner" {
                BevelFilterType::Inner
            } else if &type_ == b"outer" {
                BevelFilterType::Outer
            } else {
                BevelFilterType::Full
            };

            self.0.type_.set(type_);
        }
        Ok(())
    }

    pub fn filter(self) -> swf::BevelFilter {
        self.0.as_ref().into()
    }
}

macro_rules! bevel_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "distance" => property(bevel_filter_method!(1), bevel_filter_method!(2); VERSION_8);
    "angle" => property(bevel_filter_method!(3), bevel_filter_method!(4); VERSION_8);
    "highlightColor" => property(bevel_filter_method!(5), bevel_filter_method!(6); VERSION_8);
    "highlightAlpha" => property(bevel_filter_method!(7), bevel_filter_method!(8); VERSION_8);
    "shadowColor" => property(bevel_filter_method!(9), bevel_filter_method!(10); VERSION_8);
    "shadowAlpha" => property(bevel_filter_method!(11), bevel_filter_method!(12); VERSION_8);
    "quality" => property(bevel_filter_method!(13), bevel_filter_method!(14); VERSION_8);
    "strength" => property(bevel_filter_method!(15), bevel_filter_method!(16); VERSION_8);
    "knockout" => property(bevel_filter_method!(17), bevel_filter_method!(18); VERSION_8);
    "blurX" => property(bevel_filter_method!(19), bevel_filter_method!(20); VERSION_8);
    "blurY" => property(bevel_filter_method!(21), bevel_filter_method!(22); VERSION_8);
    "type" => property(bevel_filter_method!(23), bevel_filter_method!(24); VERSION_8);
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
    const GET_HIGHLIGHT_COLOR: u8 = 5;
    const SET_HIGHLIGHT_COLOR: u8 = 6;
    const GET_HIGHLIGHT_ALPHA: u8 = 7;
    const SET_HIGHLIGHT_ALPHA: u8 = 8;
    const GET_SHADOW_COLOR: u8 = 9;
    const SET_SHADOW_COLOR: u8 = 10;
    const GET_SHADOW_ALPHA: u8 = 11;
    const SET_SHADOW_ALPHA: u8 = 12;
    const GET_QUALITY: u8 = 13;
    const SET_QUALITY: u8 = 14;
    const GET_STRENGTH: u8 = 15;
    const SET_STRENGTH: u8 = 16;
    const GET_KNOCKOUT: u8 = 17;
    const SET_KNOCKOUT: u8 = 18;
    const GET_BLUR_X: u8 = 19;
    const SET_BLUR_X: u8 = 20;
    const GET_BLUR_Y: u8 = 21;
    const SET_BLUR_Y: u8 = 22;
    const GET_TYPE: u8 = 23;
    const SET_TYPE: u8 = 24;

    if index == CONSTRUCTOR {
        let bevel_filter = BevelFilter::new(activation, args)?;
        this.set_native(activation.gc(), NativeObject::BevelFilter(bevel_filter));
        return Ok(this.into());
    }

    let this = match this.native() {
        NativeObject::BevelFilter(bevel_filter) => bevel_filter,
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
        GET_HIGHLIGHT_COLOR => this.highlight_color().into(),
        SET_HIGHLIGHT_COLOR => {
            this.set_highlight_color(activation, args.get(0))?;
            Value::Undefined
        }
        GET_HIGHLIGHT_ALPHA => this.highlight_alpha().into(),
        SET_HIGHLIGHT_ALPHA => {
            this.set_highlight_alpha(activation, args.get(0))?;
            Value::Undefined
        }
        GET_SHADOW_COLOR => this.shadow_color().into(),
        SET_SHADOW_COLOR => {
            this.set_shadow_color(activation, args.get(0))?;
            Value::Undefined
        }
        GET_SHADOW_ALPHA => this.shadow_alpha().into(),
        SET_SHADOW_ALPHA => {
            this.set_shadow_alpha(activation, args.get(0))?;
            Value::Undefined
        }
        GET_QUALITY => this.quality().into(),
        SET_QUALITY => {
            this.set_quality(activation, args.get(0))?;
            Value::Undefined
        }
        GET_STRENGTH => this.strength().into(),
        SET_STRENGTH => {
            this.set_strength(activation, args.get(0))?;
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
        GET_TYPE => {
            let type_ = match this.type_() {
                BevelFilterType::Inner => istr!("inner"),
                BevelFilterType::Outer => istr!("outer"),
                BevelFilterType::Full => istr!("full"),
            };

            type_.into()
        }
        SET_TYPE => {
            this.set_type(activation, args.get(0))?;
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
    let bevel_filter_proto = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, bevel_filter_proto, fn_proto);
    bevel_filter_proto.into()
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context,
        Executable::Native(bevel_filter_method!(0)),
        constructor_to_fn!(bevel_filter_method!(0)),
        fn_proto,
        proto,
    )
}
