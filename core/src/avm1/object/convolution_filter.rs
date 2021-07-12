use crate::add_field_accessors;
use crate::avm1::{Object, ScriptObject, TObject};
use crate::impl_custom_object;
use gc_arena::{Collect, GcCell, MutationContext};

use std::fmt;

/// A ConvolutionFilter
#[derive(Clone, Copy, Collect)]
#[collect(no_drop)]
pub struct ConvolutionFilterObject<'gc>(GcCell<'gc, ConvolutionFilterData<'gc>>);

#[derive(Clone, Collect)]
#[collect(no_drop)]
pub struct ConvolutionFilterData<'gc> {
    /// The underlying script object.
    base: ScriptObject<'gc>,

    alpha: f64,
    bias: f64,
    clamp: bool,
    color: u32,
    divisor: f64,
    matrix: Vec<f64>,
    matrix_x: u8,
    matrix_y: u8,
    preserve_alpha: bool,
}

impl fmt::Debug for ConvolutionFilterObject<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let this = self.0.read();
        f.debug_struct("ConvolutionFilter")
            .field("alpha", &this.alpha)
            .field("bias", &this.bias)
            .field("clamp", &this.clamp)
            .field("color", &this.color)
            .field("divisor", &this.divisor)
            .field("matrix", &this.matrix)
            .field("matrixX", &this.matrix_x)
            .field("matrixY", &this.matrix_y)
            .field("preserveAlpha", &this.preserve_alpha)
            .finish()
    }
}

impl<'gc> ConvolutionFilterObject<'gc> {
    add_field_accessors!(
        [alpha, f64, set => set_alpha, get => alpha],
        [bias, f64, set => set_bias, get => bias],
        [clamp, bool, set => set_clamp, get => clamp],
        [color, u32, set => set_color, get => color],
        [divisor, f64, set => set_divisor, get => divisor],
        [matrix_x, u8, get => matrix_x],
        [matrix_y, u8, get => matrix_y],
        [matrix, Vec<f64>, set => set_matrix],
        [preserve_alpha, bool, set => set_preserve_alpha, get => preserve_alpha],
    );

    pub fn matrix(&self) -> Vec<f64> {
        self.0.read().matrix.clone()
    }

    fn update_matrix_length(&self, gc_context: MutationContext<'gc, '_>) {
        let mut matrix = self.matrix();
        while (self.matrix_x() * self.matrix_y()) as usize > matrix.len() {
            matrix.push(0.0);
        }
        self.set_matrix(gc_context, matrix);
    }

    pub fn set_matrix_x(&self, gc_context: MutationContext<'gc, '_>, matrix_x: u8) {
        self.0.write(gc_context).matrix_x = matrix_x;
        self.update_matrix_length(gc_context);
    }

    pub fn set_matrix_y(&self, gc_context: MutationContext<'gc, '_>, matrix_y: u8) {
        self.0.write(gc_context).matrix_y = matrix_y;
        self.update_matrix_length(gc_context);
    }

    pub fn empty_object(gc_context: MutationContext<'gc, '_>, proto: Option<Object<'gc>>) -> Self {
        ConvolutionFilterObject(GcCell::allocate(
            gc_context,
            ConvolutionFilterData {
                base: ScriptObject::object(gc_context, proto),
                alpha: 0.0,
                bias: 0.0,
                clamp: true,
                color: 0,
                divisor: 1.0,
                matrix: vec![],
                matrix_x: 0,
                matrix_y: 0,
                preserve_alpha: true,
            },
        ))
    }
}

impl<'gc> TObject<'gc> for ConvolutionFilterObject<'gc> {
    impl_custom_object!(base {
        bare_object(as_convolution_filter_object -> ConvolutionFilterObject::empty_object);
    });
}
