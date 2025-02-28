//! flash.filters.ConvolutionFilter object

use crate::avm1::clamp::Clamp;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, ArrayObject, Error, Object, ScriptObject, TObject, Value};
use crate::string::StringContext;
use gc_arena::{Collect, Gc, Mutation};
use std::cell::{Cell, RefCell};
use swf::{Color, ConvolutionFilterFlags};

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct ConvolutionFilterData {
    matrix_x: Cell<u8>,
    matrix_y: Cell<u8>,
    matrix: RefCell<Vec<f32>>,
    divisor: Cell<f32>,
    bias: Cell<f32>,
    preserve_alpha: Cell<bool>,
    clamp: Cell<bool>,
    color: Cell<Color>,
}

impl ConvolutionFilterData {
    fn resize_matrix(&self) {
        let new_len = (self.matrix_x.get() * self.matrix_y.get()) as usize;
        let mut matrix = self.matrix.borrow_mut();
        if new_len > matrix.len() {
            matrix.resize(new_len, 0.0);
        }
    }

    fn set_matrix_x(&self, matrix_x: u8) {
        self.matrix_x.set(matrix_x);
        self.resize_matrix();
    }

    fn set_matrix_y(&self, matrix_y: u8) {
        self.matrix_y.set(matrix_y);
        self.resize_matrix();
    }
}

impl From<&ConvolutionFilterData> for swf::ConvolutionFilter {
    fn from(filter: &ConvolutionFilterData) -> swf::ConvolutionFilter {
        let mut flags = ConvolutionFilterFlags::empty();
        flags.set(
            ConvolutionFilterFlags::PRESERVE_ALPHA,
            filter.preserve_alpha.get(),
        );
        flags.set(ConvolutionFilterFlags::CLAMP, filter.clamp.get());
        swf::ConvolutionFilter {
            num_matrix_rows: filter.matrix_y.get(),
            num_matrix_cols: filter.matrix_x.get(),
            matrix: filter.matrix.borrow().clone(),
            divisor: filter.divisor.get(),
            bias: filter.bias.get(),
            default_color: filter.color.get(),
            flags,
        }
    }
}

impl From<swf::ConvolutionFilter> for ConvolutionFilterData {
    fn from(filter: swf::ConvolutionFilter) -> ConvolutionFilterData {
        let preserve_alpha = filter.is_preserve_alpha();
        let clamp = filter.is_clamped();
        Self {
            matrix_x: Cell::new(filter.num_matrix_cols),
            matrix_y: Cell::new(filter.num_matrix_rows),
            matrix: RefCell::new(filter.matrix),
            divisor: Cell::new(filter.divisor),
            bias: Cell::new(filter.bias),
            preserve_alpha: Cell::new(preserve_alpha),
            clamp: Cell::new(clamp),
            color: Cell::new(filter.default_color),
        }
    }
}

impl Default for ConvolutionFilterData {
    fn default() -> Self {
        Self {
            matrix_x: Cell::new(0),
            matrix_y: Cell::new(0),
            matrix: RefCell::new(Vec::new()),
            divisor: Cell::new(1.0),
            bias: Cell::new(0.0),
            preserve_alpha: Cell::new(true),
            clamp: Cell::new(true),
            color: Cell::new(Color::from_rgba(0)),
        }
    }
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
#[repr(transparent)]
pub struct ConvolutionFilter<'gc>(Gc<'gc, ConvolutionFilterData>);

impl<'gc> ConvolutionFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let convolution_filter = Self(Gc::new(activation.gc(), Default::default()));
        convolution_filter.set_matrix_x(activation, args.get(0))?;
        convolution_filter.set_matrix_y(activation, args.get(1))?;
        convolution_filter.set_matrix(activation, args.get(2))?;
        if let Some(value) = args.get(3) {
            convolution_filter.set_divisor(activation, Some(value))?;
        } else if !args.is_empty() {
            let divisor = convolution_filter.0.matrix.borrow().iter().sum();
            convolution_filter.0.divisor.set(divisor);
        }
        convolution_filter.set_bias(activation, args.get(4))?;
        convolution_filter.set_preserve_alpha(activation, args.get(5))?;
        convolution_filter.set_clamp(activation, args.get(6))?;
        if let Some(value) = args.get(7) {
            convolution_filter.set_color(activation, Some(value))?;

            // If a substitute color is specified in the constructor in AVM1,
            // the substitute alpha is set to 1, despite the documentation claiming otherwise.
            // This does not happen in AVM2.
            let mut color = convolution_filter.0.color.get();
            color.a = 255;
            convolution_filter.0.color.set(color);
        }
        convolution_filter.set_alpha(activation, args.get(8))?;
        Ok(convolution_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::ConvolutionFilter) -> Self {
        Self(Gc::new(gc_context, filter.into()))
    }

    pub(crate) fn duplicate(self, gc_context: &Mutation<'gc>) -> Self {
        Self(Gc::new(gc_context, self.0.as_ref().clone()))
    }

    fn matrix_x(self) -> u8 {
        self.0.matrix_x.get()
    }

    fn set_matrix_x(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let matrix_x = value.coerce_to_i32(activation)?.clamp(0, 15) as u8;
            self.0.set_matrix_x(matrix_x);
        }
        Ok(())
    }

    fn matrix_y(self) -> u8 {
        self.0.matrix_y.get()
    }

    fn set_matrix_y(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let matrix_y = value.coerce_to_i32(activation)?.clamp(0, 15) as u8;
            self.0.set_matrix_y(matrix_y);
        }
        Ok(())
    }

    fn matrix(self, activation: &Activation<'_, 'gc>) -> ArrayObject<'gc> {
        ArrayObject::builder(activation).with(self.0.matrix.borrow().iter().map(|&x| x.into()))
    }

    fn set_matrix(
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

        *self.0.matrix.borrow_mut() = vec![0.0; length];
        for i in 0..length {
            let elem = object
                .get_element(activation, i as i32)
                .coerce_to_f64(activation)? as f32;
            self.0.matrix.borrow_mut()[i] = elem;
        }
        self.0.resize_matrix();
        Ok(())
    }

    fn divisor(self) -> f32 {
        self.0.divisor.get()
    }

    fn set_divisor(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let divisor = value.coerce_to_f64(activation)? as f32;
            self.0.divisor.set(divisor);
        }
        Ok(())
    }

    fn bias(self) -> f32 {
        self.0.bias.get()
    }

    fn set_bias(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let bias = value.coerce_to_f64(activation)? as f32;
            self.0.bias.set(bias);
        }
        Ok(())
    }

    fn preserve_alpha(self) -> bool {
        self.0.preserve_alpha.get()
    }

    fn set_preserve_alpha(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let preserve_alpha = value.as_bool(activation.swf_version());
            self.0.preserve_alpha.set(preserve_alpha);
        }
        Ok(())
    }

    fn clamp(self) -> bool {
        self.0.clamp.get()
    }

    fn set_clamp(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let clamp = value.as_bool(activation.swf_version());
            self.0.clamp.set(clamp);
        }
        Ok(())
    }

    fn color(self) -> Color {
        self.0.color.get()
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

    fn set_alpha(
        self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        if let Some(value) = value {
            let alpha = value.coerce_to_f64(activation)?.clamp_also_nan(0.0, 1.0);
            let mut color = self.0.color.get();
            color.a = (alpha * 255.0) as u8;
            self.0.color.set(color);
        }
        Ok(())
    }

    pub fn filter(self) -> swf::ConvolutionFilter {
        self.0.as_ref().into()
    }
}

macro_rules! convolution_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "matrixX" => property(convolution_filter_method!(1), convolution_filter_method!(2); VERSION_8);
    "matrixY" => property(convolution_filter_method!(3), convolution_filter_method!(4); VERSION_8);
    "matrix" => property(convolution_filter_method!(5), convolution_filter_method!(6); VERSION_8);
    "divisor" => property(convolution_filter_method!(7), convolution_filter_method!(8); VERSION_8);
    "bias" => property(convolution_filter_method!(9), convolution_filter_method!(10); VERSION_8);
    "preserveAlpha" => property(convolution_filter_method!(11), convolution_filter_method!(12); VERSION_8);
    "clamp" => property(convolution_filter_method!(13), convolution_filter_method!(14); VERSION_8);
    "color" => property(convolution_filter_method!(15), convolution_filter_method!(16); VERSION_8);
    "alpha" => property(convolution_filter_method!(17), convolution_filter_method!(18); VERSION_8);
};

fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u8,
) -> Result<Value<'gc>, Error<'gc>> {
    const CONSTRUCTOR: u8 = 0;
    const GET_MATRIX_X: u8 = 1;
    const SET_MATRIX_X: u8 = 2;
    const GET_MATRIX_Y: u8 = 3;
    const SET_MATRIX_Y: u8 = 4;
    const GET_MATRIX: u8 = 5;
    const SET_MATRIX: u8 = 6;
    const GET_DIVISOR: u8 = 7;
    const SET_DIVISOR: u8 = 8;
    const GET_BIAS: u8 = 9;
    const SET_BIAS: u8 = 10;
    const GET_PRESERVE_ALPHA: u8 = 11;
    const SET_PRESERVE_ALPHA: u8 = 12;
    const GET_CLAMP: u8 = 13;
    const SET_CLAMP: u8 = 14;
    const GET_COLOR: u8 = 15;
    const SET_COLOR: u8 = 16;
    const GET_ALPHA: u8 = 17;
    const SET_ALPHA: u8 = 18;

    if index == CONSTRUCTOR {
        let convolution_filter = ConvolutionFilter::new(activation, args)?;
        this.set_native(
            activation.gc(),
            NativeObject::ConvolutionFilter(convolution_filter),
        );
        return Ok(this.into());
    }

    let this = match this.native() {
        NativeObject::ConvolutionFilter(convolution_filter) => convolution_filter,
        _ => return Ok(Value::Undefined),
    };

    Ok(match index {
        GET_MATRIX_X => this.matrix_x().into(),
        SET_MATRIX_X => {
            this.set_matrix_x(activation, args.get(0))?;
            Value::Undefined
        }
        GET_MATRIX_Y => this.matrix_y().into(),
        SET_MATRIX_Y => {
            this.set_matrix_y(activation, args.get(0))?;
            Value::Undefined
        }
        GET_MATRIX => this.matrix(activation).into(),
        SET_MATRIX => {
            this.set_matrix(activation, args.get(0))?;
            Value::Undefined
        }
        GET_DIVISOR => this.divisor().into(),
        SET_DIVISOR => {
            this.set_divisor(activation, args.get(0))?;
            Value::Undefined
        }
        GET_BIAS => this.bias().into(),
        SET_BIAS => {
            this.set_bias(activation, args.get(0))?;
            Value::Undefined
        }
        GET_PRESERVE_ALPHA => this.preserve_alpha().into(),
        SET_PRESERVE_ALPHA => {
            this.set_preserve_alpha(activation, args.get(0))?;
            Value::Undefined
        }
        GET_CLAMP => this.clamp().into(),
        SET_CLAMP => {
            this.set_clamp(activation, args.get(0))?;
            Value::Undefined
        }
        GET_COLOR => this.color().to_rgb().into(),
        SET_COLOR => {
            this.set_color(activation, args.get(0))?;
            Value::Undefined
        }
        GET_ALPHA => (this.color().a as f64 / 255.0).into(),
        SET_ALPHA => {
            this.set_alpha(activation, args.get(0))?;
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
    let convolution_filter_proto = ScriptObject::new(context, Some(proto));
    define_properties_on(PROTO_DECLS, context, convolution_filter_proto, fn_proto);
    convolution_filter_proto.into()
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context,
        Executable::Native(convolution_filter_method!(0)),
        constructor_to_fn!(convolution_filter_method!(0)),
        fn_proto,
        proto,
    )
}
