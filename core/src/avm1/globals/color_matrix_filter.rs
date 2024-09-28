//! flash.filters.ColorMatrixFilter object

use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Activation, ArrayObject, Error, Object, ScriptObject, TObject, Value};
use crate::string::StringContext;
use gc_arena::{Collect, GcCell, Mutation};
use std::ops::Deref;

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct ColorMatrixFilterData {
    matrix: [f32; 4 * 5],
}

impl From<&ColorMatrixFilterData> for swf::ColorMatrixFilter {
    fn from(filter: &ColorMatrixFilterData) -> swf::ColorMatrixFilter {
        swf::ColorMatrixFilter {
            matrix: filter.matrix,
        }
    }
}

impl From<swf::ColorMatrixFilter> for ColorMatrixFilterData {
    fn from(filter: swf::ColorMatrixFilter) -> ColorMatrixFilterData {
        Self {
            matrix: filter.matrix,
        }
    }
}

impl Default for ColorMatrixFilterData {
    fn default() -> Self {
        Self {
            #[rustfmt::skip]
            matrix: [
                1.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0, 0.0,
            ],
        }
    }
}

#[derive(Clone, Debug, Collect)]
#[collect(no_drop)]
#[repr(transparent)]
pub struct ColorMatrixFilter<'gc>(GcCell<'gc, ColorMatrixFilterData>);

impl<'gc> ColorMatrixFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let color_matrix_filter = Self(GcCell::new(
            activation.context.gc_context,
            Default::default(),
        ));
        color_matrix_filter.set_matrix(activation, args.get(0))?;
        Ok(color_matrix_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::ColorMatrixFilter) -> Self {
        Self(GcCell::new(gc_context, filter.into()))
    }

    pub(crate) fn duplicate(&self, gc_context: &Mutation<'gc>) -> Self {
        Self(GcCell::new(gc_context, self.0.read().clone()))
    }

    fn matrix(&self, activation: &mut Activation<'_, 'gc>) -> Value<'gc> {
        ArrayObject::new(
            activation.context.gc_context,
            activation.context.avm1.prototypes().array,
            self.0.read().matrix.iter().map(|&v| v.into()),
        )
        .into()
    }

    fn set_matrix(
        &self,
        activation: &mut Activation<'_, 'gc>,
        value: Option<&Value<'gc>>,
    ) -> Result<(), Error<'gc>> {
        // Note that FP 11 and FP 32 behave differently here:
        // - FP 11 ignores non-object values, whereas FP 32 treat them as an empty array,
        //   except for null and undefined;
        // - FP 11 uses `0` as a default for missing elements, whereas FP 32 uses `NaN`.
        // This implements FP 32 semantics.
        let mut matrix = [f32::NAN; 4 * 5];

        match value {
            None | Some(Value::Null | Value::Undefined) => return Ok(()),
            Some(Value::Object(object)) => {
                for (i, m) in matrix.iter_mut().enumerate() {
                    let i = i as i32;
                    let length = object.length(activation)?;
                    if i < length {
                        *m = object
                            .get_element(activation, i)
                            .coerce_to_f64(activation)? as f32;
                    }
                }
            }
            _ => (),
        }

        self.0.write(activation.context.gc_context).matrix = matrix;
        Ok(())
    }

    pub fn filter(&self) -> swf::ColorMatrixFilter {
        self.0.read().deref().into()
    }
}

macro_rules! color_matrix_filter_method {
    ($index:literal) => {
        |activation, this, args| method(activation, this, args, $index)
    };
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "matrix" => property(color_matrix_filter_method!(1), color_matrix_filter_method!(2); VERSION_8);
};

fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u8,
) -> Result<Value<'gc>, Error<'gc>> {
    const CONSTRUCTOR: u8 = 0;
    const GET_MATRIX: u8 = 1;
    const SET_MATRIX: u8 = 2;

    if index == CONSTRUCTOR {
        let color_matrix_filter = ColorMatrixFilter::new(activation, args)?;
        this.set_native(
            activation.context.gc_context,
            NativeObject::ColorMatrixFilter(color_matrix_filter),
        );
        return Ok(this.into());
    }

    let this = match this.native() {
        NativeObject::ColorMatrixFilter(color_matrix_filter) => color_matrix_filter,
        _ => return Ok(Value::Undefined),
    };

    Ok(match index {
        GET_MATRIX => this.matrix(activation),
        SET_MATRIX => {
            this.set_matrix(activation, args.get(0))?;
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
    let color_matrix_filter_proto = ScriptObject::new(context.gc_context, Some(proto));
    define_properties_on(PROTO_DECLS, context, color_matrix_filter_proto, fn_proto);
    color_matrix_filter_proto.into()
}

pub fn create_constructor<'gc>(
    context: &mut StringContext<'gc>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    FunctionObject::constructor(
        context.gc_context,
        Executable::Native(color_matrix_filter_method!(0)),
        constructor_to_fn!(color_matrix_filter_method!(0)),
        fn_proto,
        proto,
    )
}
