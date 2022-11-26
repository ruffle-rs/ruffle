//! flash.filters.BevelFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, Error, Object, ScriptObject, TObject, Value};
use crate::string::{AvmString, WStr};
use gc_arena::{Collect, GcCell, MutationContext};
use swf::Color;

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
pub enum BevelFilterType {
    Inner,
    Outer,
    Full,
}

impl From<&WStr> for BevelFilterType {
    fn from(value: &WStr) -> Self {
        if value == b"inner" {
            Self::Inner
        } else if value == b"outer" {
            Self::Outer
        } else {
            Self::Full
        }
    }
}

impl From<BevelFilterType> for &'static WStr {
    fn from(type_: BevelFilterType) -> &'static WStr {
        match type_ {
            BevelFilterType::Inner => WStr::from_units(b"inner"),
            BevelFilterType::Outer => WStr::from_units(b"outer"),
            BevelFilterType::Full => WStr::from_units(b"full"),
        }
    }
}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct BevelFilterObject {
    distance: f64,
    // TODO: Introduce `Angle<Radians>` struct.
    angle: f64,
    highlight: Color,
    shadow: Color,
    quality: i32,
    // TODO: Introduce unsigned `Fixed8`?
    strength: u16,
    knockout: bool,
    blur_x: f64,
    blur_y: f64,
    type_: BevelFilterType,
}

impl BevelFilterObject {
    fn new<'gc>(
        activation: &mut Activation<'_, 'gc, '_>,
        args: &[Value<'gc>],
    ) -> Result<GcCell<'gc, Self>, Error<'gc>> {
        let bevel_filter = GcCell::allocate(activation.context.gc_context, Default::default());
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
}

impl Default for BevelFilterObject {
    #[allow(clippy::approx_constant)]
    fn default() -> Self {
        Self {
            distance: 4.0,
            angle: 0.785398163, // ~45 degrees
            highlight: Color::WHITE,
            shadow: Color::BLACK,
            quality: 1,
            strength: 1 << 8,
            knockout: false,
            blur_x: 4.0,
            blur_y: 4.0,
            type_: BevelFilterType::Inner,
        }
    }
}

trait BevelFilterExt<'gc> {
    fn distance(self) -> f64;

    fn set_distance(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn angle(self) -> f64;

    fn set_angle(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn highlight_color(self) -> i32;

    fn set_highlight_color(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn highlight_alpha(self) -> f64;

    fn set_highlight_alpha(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn shadow_color(self) -> i32;

    fn set_shadow_color(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn shadow_alpha(self) -> f64;

    fn set_shadow_alpha(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn quality(self) -> i32;

    fn set_quality(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn strength(self) -> f64;

    fn set_strength(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn knockout(self) -> bool;

    fn set_knockout(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn blur_x(self) -> f64;

    fn set_blur_x(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn blur_y(self) -> f64;

    fn set_blur_y(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;

    fn type_(self) -> BevelFilterType;

    fn set_type(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>>;
}

impl<'gc> BevelFilterExt<'gc> for GcCell<'gc, BevelFilterObject> {
    fn distance(self) -> f64 {
        self.read().distance
    }

    fn set_distance(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let distance = value.coerce_to_f64(activation)?;
            self.write(activation.context.gc_context).distance = distance;
        }
        Ok(())
    }

    fn angle(self) -> f64 {
        self.read().angle.to_degrees()
    }

    fn set_angle(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let angle = (value.coerce_to_f64(activation)? % 360.0).to_radians();
            self.write(activation.context.gc_context).angle = angle;
        }
        Ok(())
    }

    fn highlight_color(self) -> i32 {
        self.read().highlight.to_rgb() as i32
    }

    fn set_highlight_color(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let color = Color::from_rgb(value.coerce_to_u32(activation)?, 0);
            self.write(activation.context.gc_context).highlight = color;
        }
        Ok(())
    }

    fn highlight_alpha(self) -> f64 {
        f64::from(self.read().highlight.a) / 255.0
    }

    fn set_highlight_alpha(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let alpha = (value.coerce_to_f64(activation)? * 255.0) as u8;
            self.write(activation.context.gc_context).highlight.a = alpha;
        }
        Ok(())
    }

    fn shadow_color(self) -> i32 {
        self.read().shadow.to_rgb() as i32
    }

    fn set_shadow_color(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let color = Color::from_rgb(value.coerce_to_u32(activation)?, 0);
            self.write(activation.context.gc_context).shadow = color;
        }
        Ok(())
    }

    fn shadow_alpha(self) -> f64 {
        f64::from(self.read().shadow.a) / 255.0
    }

    fn set_shadow_alpha(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let alpha = (value.coerce_to_f64(activation)? * 255.0) as u8;
            self.write(activation.context.gc_context).shadow.a = alpha;
        }
        Ok(())
    }

    fn quality(self) -> i32 {
        self.read().quality
    }

    fn set_quality(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let quality = value.coerce_to_i32(activation)?.clamp(0, 15);
            self.write(activation.context.gc_context).quality = quality;
        }
        Ok(())
    }

    fn strength(self) -> f64 {
        f64::from(self.read().strength) / 256.0
    }

    fn set_strength(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let strength = ((value.coerce_to_f64(activation)? * 256.0) as u16).clamp(0, 0xFF00);
            self.write(activation.context.gc_context).strength = strength;
        }
        Ok(())
    }

    fn knockout(self) -> bool {
        self.read().knockout
    }

    fn set_knockout(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let knockout = value.as_bool(activation.swf_version());
            self.write(activation.context.gc_context).knockout = knockout;
        }
        Ok(())
    }

    fn blur_x(self) -> f64 {
        self.read().blur_x
    }

    fn set_blur_x(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_x = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.write(activation.context.gc_context).blur_x = blur_x;
        }
        Ok(())
    }

    fn blur_y(self) -> f64 {
        self.read().blur_y
    }

    fn set_blur_y(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let blur_y = value.coerce_to_f64(activation)?.clamp(0.0, 255.0);
            self.write(activation.context.gc_context).blur_y = blur_y;
        }
        Ok(())
    }

    fn type_(self) -> BevelFilterType {
        self.read().type_
    }

    fn set_type(
        self,
        activation: &mut Activation<'_, 'gc, '_>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let type_ = value.coerce_to_string(activation)?.as_wstr().into();
            self.write(activation.context.gc_context).type_ = type_;
        }
        Ok(())
    }
}

macro_rules! bevel_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "distance" => property(bevel_filter_method!(1), bevel_filter_method!(2));
    "angle" => property(bevel_filter_method!(3), bevel_filter_method!(4));
    "highlightColor" => property(bevel_filter_method!(5), bevel_filter_method!(6));
    "highlightAlpha" => property(bevel_filter_method!(7), bevel_filter_method!(8));
    "shadowColor" => property(bevel_filter_method!(9), bevel_filter_method!(10));
    "shadowAlpha" => property(bevel_filter_method!(11), bevel_filter_method!(12));
    "quality" => property(bevel_filter_method!(13), bevel_filter_method!(14));
    "strength" => property(bevel_filter_method!(15), bevel_filter_method!(16));
    "knockout" => property(bevel_filter_method!(17), bevel_filter_method!(18));
    "blurX" => property(bevel_filter_method!(19), bevel_filter_method!(20));
    "blurY" => property(bevel_filter_method!(21), bevel_filter_method!(22));
    "type" => property(bevel_filter_method!(23), bevel_filter_method!(24));
};

fn method<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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
        let bevel_filter = BevelFilterObject::new(activation, args)?;
        this.set_native(
            activation.context.gc_context,
            NativeObject::BevelFilter(bevel_filter),
        );
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
            let type_: &WStr = this.type_().into();
            AvmString::from(type_).into()
        }
        SET_TYPE => {
            this.set_type(activation, args.get(0))?;
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
    let bevel_filter_proto = ScriptObject::new(gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, gc_context, bevel_filter_proto, fn_proto);
    FunctionObject::constructor(
        gc_context,
        Executable::Native(bevel_filter_method!(0)),
        constructor_to_fn!(bevel_filter_method!(0)),
        fn_proto,
        bevel_filter_proto.into(),
    )
}
