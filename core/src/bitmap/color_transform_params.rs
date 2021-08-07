use gc_arena::Collect;

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ColorTransformParams {
    pub red_multiplier: f64,
    pub green_multiplier: f64,
    pub blue_multiplier: f64,
    pub alpha_multiplier: f64,
    pub red_offset: f64,
    pub green_offset: f64,
    pub blue_offset: f64,
    pub alpha_offset: f64,
}
