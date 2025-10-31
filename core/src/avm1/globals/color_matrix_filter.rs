use crate::avm1::object::NativeObject;
use crate::avm1::property_decl::{DeclContext, Declaration, SystemClass};
use crate::avm1::{Activation, ArrayBuilder, Error, Object, Value};
use gc_arena::{Collect, Gc, Mutation};
use std::cell::Cell;

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
struct ColorMatrixFilterData {
    matrix: Cell<[f32; 4 * 5]>,
}

impl From<&ColorMatrixFilterData> for swf::ColorMatrixFilter {
    fn from(filter: &ColorMatrixFilterData) -> swf::ColorMatrixFilter {
        swf::ColorMatrixFilter {
            matrix: filter.matrix.get(),
        }
    }
}

impl From<swf::ColorMatrixFilter> for ColorMatrixFilterData {
    fn from(filter: swf::ColorMatrixFilter) -> ColorMatrixFilterData {
        Self {
            matrix: Cell::new(filter.matrix),
        }
    }
}

impl Default for ColorMatrixFilterData {
    fn default() -> Self {
        Self {
            #[rustfmt::skip]
            matrix: Cell::new([
                1.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 0.0, 1.0, 0.0,
            ]),
        }
    }
}

#[derive(Copy, Clone, Debug, Collect)]
#[collect(no_drop)]
#[repr(transparent)]
pub struct ColorMatrixFilter<'gc>(Gc<'gc, ColorMatrixFilterData>);

impl<'gc> ColorMatrixFilter<'gc> {
    fn new(activation: &mut Activation<'_, 'gc>, args: &[Value<'gc>]) -> Result<Self, Error<'gc>> {
        let color_matrix_filter = Self(Gc::new(activation.gc(), Default::default()));
        color_matrix_filter.set_matrix(activation, args.get(0))?;
        Ok(color_matrix_filter)
    }

    pub fn from_filter(gc_context: &Mutation<'gc>, filter: swf::ColorMatrixFilter) -> Self {
        Self(Gc::new(gc_context, filter.into()))
    }

    pub(crate) fn duplicate(self, gc_context: &Mutation<'gc>) -> Self {
        Self(Gc::new(gc_context, self.0.as_ref().clone()))
    }

    fn matrix(self, activation: &mut Activation<'_, 'gc>) -> Value<'gc> {
        // Use `.as_slice_of_cells()` to avoid a copy out of the `Cell`.
        // FIXME: use `.as_array_of_cells()` once stabilized.
        let matrix: &Cell<[f32]> = &self.0.matrix;
        let matrix = matrix.as_slice_of_cells();
        ArrayBuilder::new(activation)
            .with(matrix.iter().map(|v| v.get().into()))
            .into()
    }

    fn set_matrix(
        self,
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

        self.0.matrix.set(matrix);
        Ok(())
    }

    pub fn filter(self) -> swf::ColorMatrixFilter {
        self.0.as_ref().into()
    }
}

const PROTO_DECLS: &[Declaration] = declare_properties! {
    use fn method;
    "matrix" => property(GET_MATRIX, SET_MATRIX; VERSION_8);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(table_constructor!(method), None, super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS);
    class
}

pub mod method {
    pub const CONSTRUCTOR: u16 = 0;
    pub const GET_MATRIX: u16 = 1;
    pub const SET_MATRIX: u16 = 2;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;

    if index == CONSTRUCTOR {
        let color_matrix_filter = ColorMatrixFilter::new(activation, args)?;
        this.set_native(
            activation.gc(),
            NativeObject::ColorMatrixFilter(color_matrix_filter),
        );
        return Ok(this.into());
    }

    let NativeObject::ColorMatrixFilter(this) = this.native() else {
        return Ok(Value::Undefined);
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
