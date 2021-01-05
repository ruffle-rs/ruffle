use crate::add_field_accessors;
use crate::avm1::error::Error;
use crate::avm1::{Object, ScriptObject, TObject, Value};
use crate::impl_custom_object_without_set;
use gc_arena::{Collect, GcCell, MutationContext};

use crate::avm1::activation::Activation;
use crate::avm1::object::bevel_filter::BevelFilterType;
use std::fmt;

/// A GradientBevelFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct GradientBevelFilterObject<'gc>(GcCell<'gc, GradientBevelFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct GradientBevelFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    alphas: Vec<f64>,
    angle: f64,
    blur_x: f64,
    blur_y: f64,
    colors: Vec<u32>,
    distance: f64,
    knockout: bool,
    quality: i32,
    ratios: Vec<u8>,
    strength: f64,
    type_: BevelFilterType,
}

impl fmt::Debug for GradientBevelFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("GradientBevelFilter")
            .field("alphas", &this.alphas)
            .field("angle", &this.angle)
            .field("blurX", &this.blur_x)
            .field("blurY", &this.blur_y)
            .field("colors", &this.colors)
            .field("distance", &this.distance)
            .field("knockout", &this.knockout)
            .field("quality", &this.quality)
            .field("ratios", &this.ratios)
            .field("strength", &this.strength)
            .field("type_", &this.type_)
            .finish()
    }
}

impl<'gc> GradientBevelFilterObject<'gc> {
    add_field_accessors!(
        [set_angle, angle, angle, f64],
        [set_blur_x, blur_x, blur_x, f64],
        [set_blur_y, blur_y, blur_y, f64],
        [set_distance, distance, distance, f64],
        [set_knockout, knockout, knockout, bool],
        [set_quality, quality, quality, i32],
        [set_strength, strength, strength, f64],
        [set_type, type_, type_, BevelFilterType],
    );

    //TODO: combine with above
    add_field_accessors!(
        [alphas, Vec<f64>, set => set_alphas],
        [colors, Vec<u32>, set => set_colors],
        [ratios, Vec<u8>, set => set_ratios],
    );

    pub fn alphas(&self) -> Vec<f64> {
        self.0.read().alphas.clone()
    }

    pub fn colors(&self) -> Vec<u32> {
        self.0.read().colors.clone()
    }

    pub fn ratios(&self) -> Vec<u8> {
        self.0.read().ratios.clone()
    }

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        GradientBevelFilterObject(GcCell::allocate(
            gc_context,
            GradientBevelFilterData {
                base: ScriptObject::object(gc_context, proto),
                alphas: vec![],
                angle: 0.0,
                blur_x: 0.0,
                blur_y: 0.0,
                colors: vec![],
                distance: 0.0,
                knockout: false,
                quality: 0,
                ratios: vec![],
                strength: 0.0,
                type_: BevelFilterType::Inner,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for GradientBevelFilterObject<'gc> {
    impl_custom_object_without_set!(base);

    fn set(
        &self,
        name: &str,
        value: Value<'gc>,
        activation: &mut Activation<'_, 'gc, '_>,
    ) -> Result<(), Error<'gc>> {
        let base = self.0.read().base;
        base.internal_set(
            name,
            value,
            activation,
            (*self).into(),
            Some(activation.context.avm1.prototypes.gradient_bevel_filter),
        )
    }

    fn as_gradient_bevel_filter_object(&self) -> Option<GradientBevelFilterObject<'gc>> {
        Some(*self)
    }

    fn create_bare_object(
        &self,
        activation: &mut Activation<'_, 'gc, '_>,
        _this: Object<'gc>,
    ) -> Result<Object<'gc>, Error<'gc>> {
        Ok(GradientBevelFilterObject::empty_object(
            activation.context.gc_context,
            Some(activation.context.avm1.prototypes.gradient_bevel_filter),
        )
        .into())
    }
}
