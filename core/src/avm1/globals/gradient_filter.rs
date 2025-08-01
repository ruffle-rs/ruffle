//! flash.filters.GradientBevelFilter and flash.filters.GradientGlowFilter objects

use crate::avm1::clamp::Clamp;
use crate::avm1::function::FunctionObject;
use crate::avm1::globals::bevel_filter::BevelFilterType;
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, ArrayBuilder, Error, Object, Value};
use crate::string::StringContext;
use gc_arena::{Collect, Gc, Mutation};
use ruffle_macros::istr;
use std::cell::{Cell, RefCell};
use swf::{Color, Fixed16, Fixed8, GradientFilterFlags, GradientRecord};

const MAX_COLORS: usize = 16;

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct GradientFilterData {
    distance: Cell<f64>,
    // TODO: Introduce `Angle<Radians>` struct.
    angle: Cell<f64>,
    colors: RefCell<[GradientRecord; MAX_COLORS]>,
    num_colors: Cell<usize>,
    blur_x: Cell<f64>,
    blur_y: Cell<f64>,
    // TODO: Introduce unsigned `Fixed8`?
    strength: Cell<u16>,
    quality: Cell<i32>,
    type_: Cell<BevelFilterType>,
    knockout: Cell<bool>,
}

impl From<&GradientFilterData> for swf::GradientFilter {
    fn from(filter: &GradientFilterData) -> swf::GradientFilter {
        let mut flags = GradientFilterFlags::COMPOSITE_SOURCE;
        flags |= GradientFilterFlags::from_passes(filter.quality.get() as u8);
        flags |= filter.type_.get().as_gradient_flags();
        flags.set(GradientFilterFlags::KNOCKOUT, filter.knockout.get());
        swf::GradientFilter {
            colors: filter
                .colors
                .borrow()
                .into_iter()
                .take(filter.num_colors.get())
                .collect(),
            blur_x: Fixed16::from_f64(filter.blur_x.get()),
            blur_y: Fixed16::from_f64(filter.blur_y.get()),
            angle: Fixed16::from_f64(filter.angle.get()),
            distance: Fixed16::from_f64(filter.distance.get()),
            strength: Fixed8::from_f64(filter.strength()),
            flags,
        }
    }
}

impl From<swf::GradientFilter> for GradientFilterData {
    fn from(filter: swf::GradientFilter) -> GradientFilterData {
        let mut colors = [GradientRecord::default(); MAX_COLORS];
        let num_colors = filter.colors.len().min(MAX_COLORS);
        for (i, slot) in colors[..num_colors].iter_mut().enumerate() {
            *slot = filter.colors[i];
        }

        let quality = filter.num_passes().into();
        let knockout = filter.is_knockout();
        Self {
            distance: Cell::new(filter.distance.into()),
            angle: Cell::new(filter.angle.into()),
            colors: RefCell::new(colors),
            num_colors: Cell::new(num_colors),
            quality: Cell::new(quality),
            strength: Cell::new((filter.strength.to_f64() * 256.0) as u16),
            knockout: Cell::new(knockout),
            blur_x: Cell::new(filter.blur_x.into()),
            blur_y: Cell::new(filter.blur_y.into()),
            type_: Cell::new(filter.flags.into()),
        }
    }
}

impl Default for GradientFilterData {
    #[expect(clippy::approx_constant)]
    fn default() -> Self {
        Self {
            distance: Cell::new(4.0),
            angle: Cell::new(0.785398163), // ~45 degrees
            colors: Default::default(),
            num_colors: Cell::new(0),
            blur_x: Cell::new(4.0),
            blur_y: Cell::new(4.0),
            strength: Cell::new(1 << 8),
            quality: Cell::new(1),
            type_: Cell::new(BevelFilterType::Inner),
            knockout: Cell::new(false),
        }
    }
}

impl GradientFilterData {
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
pub struct GradientFilter<'gc>(Gc<'gc, GradientFilterData>);

impl<'gc> GradientFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let gradient_bevel_filter = Self(Gc::new(activation.gc(), Default::default()));
        gradient_bevel_filter.set_distance(activation, args.get(0))?;
        gradient_bevel_filter.set_angle(activation, args.get(1))?;
        gradient_bevel_filter.set_colors(activation, args.get(2))?;
        gradient_bevel_filter.set_alphas(activation, args.get(3))?;
        gradient_bevel_filter.set_ratios(activation, args.get(4))?;
        gradient_bevel_filter.set_blur_x(activation, args.get(5))?;
        gradient_bevel_filter.set_blur_y(activation, args.get(6))?;
        gradient_bevel_filter.set_strength(activation, args.get(7))?;
        gradient_bevel_filter.set_quality(activation, args.get(8))?;
        gradient_bevel_filter.set_type(activation, args.get(9))?;
        gradient_bevel_filter.set_knockout(activation, args.get(10))?;
        Ok(gradient_bevel_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::GradientFilter) -> Self {
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

    fn colors(self, activation: &Activation<'_, 'gc>) -> Object<'gc> {
        let num_colors = self.0.num_colors.get();
        ArrayBuilder::new(activation).with(
            self.0.colors.borrow()[..num_colors]
                .iter()
                .map(|r| r.color.to_rgb().into()),
        )
    }

    fn set_colors(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        let Some(value) = value else { return Ok(()) };

        // FP 11 and FP 32 behave differently here: in FP 11, only "true" objects resize
        // the matrix, but in FP 32 strings will too (and so fill the matrix with `NaN`
        // values, as they have a `length` but no actual elements).
        let object = value.coerce_to_object(activation);
        let length = usize::try_from(object.length(activation)?).unwrap_or_default();
        let num_colors = length.min(MAX_COLORS);

        self.0.num_colors.set(num_colors);

        let mut colors = self.0.colors.borrow_mut();
        for i in 0..num_colors {
            let rgb = object
                .get_element(activation, i as i32)
                .coerce_to_i32(activation)? as u32;
            let alpha = colors[i].color.a;
            colors[i].color = Color::from_rgb(rgb, alpha);
        }
        Ok(())
    }

    fn alphas(self, activation: &Activation<'_, 'gc>) -> Object<'gc> {
        let num_colors = self.0.num_colors.get();
        ArrayBuilder::new(activation).with(
            self.0.colors.borrow()[..num_colors]
                .iter()
                .map(|r| (f64::from(r.color.a) / 255.0).into()),
        )
    }

    fn set_alphas(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(Value::Object(object)) = value {
            // Note that unlike `colors` and `ratios`, setting `alphas` doesn't change
            // the number of colors in the gradient.
            let num_colors = self.0.num_colors.get();
            let length = usize::try_from(object.length(activation)?).unwrap_or_default();
            let mut colors = self.0.colors.borrow_mut();
            for i in 0..num_colors {
                let alpha = if i < length {
                    let alpha = object
                        .get_element(activation, i as i32)
                        .coerce_to_f64(activation)?;
                    if alpha.is_finite() {
                        (255.0 * alpha + 0.5) as u8
                    } else {
                        u8::MAX
                    }
                } else {
                    u8::MAX
                };
                colors[i].color.a = alpha;
            }
        }
        Ok(())
    }

    fn ratios(self, activation: &Activation<'_, 'gc>) -> Object<'gc> {
        let num_colors = self.0.num_colors.get();
        ArrayBuilder::new(activation).with(
            self.0.colors.borrow()[..num_colors]
                .iter()
                .map(|r| r.ratio.into()),
        )
    }

    fn set_ratios(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(Value::Object(object)) = value {
            let num_colors = usize::try_from(object.length(activation)?).unwrap_or_default();

            // Modifying `ratios` can only reduce the number of colors, never increase it.
            let num_colors = num_colors.min(self.0.num_colors.get());
            self.0.num_colors.set(num_colors);

            let mut colors = self.0.colors.borrow_mut();
            for i in 0..num_colors {
                let ratio = object
                    .get_element(activation, i as i32)
                    .coerce_to_i32(activation)?
                    .clamp(0, u8::MAX.into()) as u8;
                colors[i].ratio = ratio;
            }
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
            let blur_x = value.coerce_to_f64(activation)?.clamp_also_nan(0.0, 255.0);
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
            let blur_y = value.coerce_to_f64(activation)?.clamp_also_nan(0.0, 255.0);
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

    pub fn filter(self) -> swf::GradientFilter {
        self.0.as_ref().into()
    }
}

macro_rules! gradient_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "distance" => property(gradient_filter_method!(1), gradient_filter_method!(2); VERSION_8);
    "angle" => property(gradient_filter_method!(3), gradient_filter_method!(4); VERSION_8);
    "colors" => property(gradient_filter_method!(5), gradient_filter_method!(6); VERSION_8);
    "alphas" => property(gradient_filter_method!(7), gradient_filter_method!(8); VERSION_8);
    "ratios" => property(gradient_filter_method!(9), gradient_filter_method!(10); VERSION_8);
    "blurX" => property(gradient_filter_method!(11), gradient_filter_method!(12); VERSION_8);
    "blurY" => property(gradient_filter_method!(13), gradient_filter_method!(14); VERSION_8);
    "quality" => property(gradient_filter_method!(15), gradient_filter_method!(16); VERSION_8);
    "strength" => property(gradient_filter_method!(17), gradient_filter_method!(18); VERSION_8);
    "knockout" => property(gradient_filter_method!(19), gradient_filter_method!(20); VERSION_8);
    "type" => property(gradient_filter_method!(21), gradient_filter_method!(22); VERSION_8);
};

fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    const GLOW_CONSTRUCTOR: u16 = 0;
    const GET_DISTANCE: u16 = 1;
    const SET_DISTANCE: u16 = 2;
    const GET_ANGLE: u16 = 3;
    const SET_ANGLE: u16 = 4;
    const GET_COLORS: u16 = 5;
    const SET_COLORS: u16 = 6;
    const GET_ALPHAS: u16 = 7;
    const SET_ALPHAS: u16 = 8;
    const GET_RATIOS: u16 = 9;
    const SET_RATIOS: u16 = 10;
    const GET_BLUR_X: u16 = 11;
    const SET_BLUR_X: u16 = 12;
    const GET_BLUR_Y: u16 = 13;
    const SET_BLUR_Y: u16 = 14;
    const GET_QUALITY: u16 = 15;
    const SET_QUALITY: u16 = 16;
    const GET_STRENGTH: u16 = 17;
    const SET_STRENGTH: u16 = 18;
    const GET_KNOCKOUT: u16 = 19;
    const SET_KNOCKOUT: u16 = 20;
    const GET_TYPE: u16 = 21;
    const SET_TYPE: u16 = 22;
    const BEVEL_CONSTRUCTOR: u16 = 1000;

    if index == BEVEL_CONSTRUCTOR {
        let gradient_bevel_filter = GradientFilter::new(activation, args)?;
        this.set_native(
            activation.gc(),
            NativeObject::GradientBevelFilter(gradient_bevel_filter),
        );
        return Ok(this.into());
    }
    if index == GLOW_CONSTRUCTOR {
        let gradient_glow_filter = GradientFilter::new(activation, args)?;
        this.set_native(
            activation.gc(),
            NativeObject::GradientGlowFilter(gradient_glow_filter),
        );
        return Ok(this.into());
    }

    let this = match this.native() {
        NativeObject::GradientBevelFilter(gradient_bevel_filter) => gradient_bevel_filter,
        NativeObject::GradientGlowFilter(gradient_glow_filter) => gradient_glow_filter,
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
        GET_COLORS => this.colors(activation).into(),
        SET_COLORS => {
            this.set_colors(activation, args.get(0))?;
            Value::Undefined
        }
        GET_ALPHAS => this.alphas(activation).into(),
        SET_ALPHAS => {
            this.set_alphas(activation, args.get(0))?;
            Value::Undefined
        }
        GET_RATIOS => this.ratios(activation).into(),
        SET_RATIOS => {
            this.set_ratios(activation, args.get(0))?;
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

pub fn create_bevel_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let gradient_bevel_filter_proto = Object::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, gradient_bevel_filter_proto, fn_proto);
    gradient_bevel_filter_proto
}

pub fn create_bevel_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context,
        gradient_filter_method!(1000),
        None,
        fn_proto,
        proto,
    )
}

pub fn create_glow_proto<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let gradient_bevel_filter_proto = Object::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, gradient_bevel_filter_proto, fn_proto);
    gradient_bevel_filter_proto
}

pub fn create_glow_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(context, gradient_filter_method!(0), None, fn_proto, proto)
}
