use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, ObjectCell, UpdateContext};
use gc_arena::GcCell;
use std::f64::NAN;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Value<'gc> {
    Undefined,
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(ObjectCell<'gc>),
}

impl<'gc> From<String> for Value<'gc> {
    fn from(string: String) -> Self {
        Value::String(string)
    }
}

impl<'gc> From<&str> for Value<'gc> {
    fn from(string: &str) -> Self {
        Value::String(string.to_owned())
    }
}

impl<'gc> From<bool> for Value<'gc> {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

impl<'gc> From<ObjectCell<'gc>> for Value<'gc> {
    fn from(object: ObjectCell<'gc>) -> Self {
        Value::Object(object)
    }
}

impl<'gc> From<f64> for Value<'gc> {
    fn from(value: f64) -> Self {
        Value::Number(value)
    }
}

impl<'gc> From<f32> for Value<'gc> {
    fn from(value: f32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<u8> for Value<'gc> {
    fn from(value: u8) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<i32> for Value<'gc> {
    fn from(value: i32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<u32> for Value<'gc> {
    fn from(value: u32) -> Self {
        Value::Number(f64::from(value))
    }
}

impl<'gc> From<usize> for Value<'gc> {
    fn from(value: usize) -> Self {
        Value::Number(value as f64)
    }
}

unsafe impl<'gc> gc_arena::Collect for Value<'gc> {
    fn trace(&self, cc: gc_arena::CollectionContext) {
        if let Value::Object(object) = self {
            object.trace(cc);
        }
    }
}

impl PartialEq for Value<'_> {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Undefined => match other {
                Value::Undefined => true,
                _ => false,
            },
            Value::Null => match other {
                Value::Null => true,
                _ => false,
            },
            Value::Bool(value) => match other {
                Value::Bool(other_value) => value == other_value,
                _ => false,
            },
            Value::Number(value) => match other {
                Value::Number(other_value) => {
                    (value == other_value) || (value.is_nan() && other_value.is_nan())
                }
                _ => false,
            },
            Value::String(value) => match other {
                Value::String(other_value) => value == other_value,
                _ => false,
            },
            Value::Object(value) => match other {
                Value::Object(other_value) => value.as_ptr() == other_value.as_ptr(),
                _ => false,
            },
        }
    }
}

impl<'gc> Value<'gc> {
    pub fn into_number_v1(self) -> f64 {
        match self {
            Value::Bool(true) => 1.0,
            Value::Number(v) => v,
            Value::String(v) => v.parse().unwrap_or(0.0),
            _ => 0.0,
        }
    }

    /// ECMA-262 2nd edtion s. 9.3 ToNumber (after calling `to_primitive_num`)
    ///
    /// Flash diverges from spec in a number of ways. These ways are, as far as
    /// we are aware, version-gated:
    ///
    /// * In SWF6 and lower, `undefined` is coerced to `0.0` (like `false`)
    /// rather than `NaN` as required by spec.
    /// * In SWF5 and lower, hexadecimal is unsupported.
    fn primitive_as_number(
        &self,
        avm: &mut Avm1<'gc>,
        _context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> f64 {
        match self {
            Value::Undefined if avm.current_swf_version() < 7 => 0.0,
            Value::Null if avm.current_swf_version() < 7 => 0.0,
            Value::Undefined => NAN,
            Value::Null => NAN,
            Value::Bool(false) => 0.0,
            Value::Bool(true) => 1.0,
            Value::Number(v) => *v,
            Value::String(v) => match v.as_str() {
                v if avm.current_swf_version() >= 6 && v.starts_with("0x") => {
                    let mut n: u32 = 0;
                    for c in v[2..].bytes() {
                        n = n.wrapping_shl(4);
                        n |= match c {
                            b'0' => 0,
                            b'1' => 1,
                            b'2' => 2,
                            b'3' => 3,
                            b'4' => 4,
                            b'5' => 5,
                            b'6' => 6,
                            b'7' => 7,
                            b'8' => 8,
                            b'9' => 9,
                            b'a' | b'A' => 10,
                            b'b' | b'B' => 11,
                            b'c' | b'C' => 12,
                            b'd' | b'D' => 13,
                            b'e' | b'E' => 14,
                            b'f' | b'F' => 15,
                            _ => return NAN,
                        }
                    }
                    f64::from(n as i32)
                }
                "" => NAN,
                _ => v.trim_start().parse().unwrap_or(NAN),
            },
            Value::Object(_) => NAN,
        }
    }

    /// ECMA-262 2nd edition s. 9.3 ToNumber
    pub fn as_number(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<f64, Error> {
        Ok(match self {
            Value::Object(_) => self
                .to_primitive_num(avm, context)?
                .primitive_as_number(avm, context),
            val => val.primitive_as_number(avm, context),
        })
    }

    /// ECMA-262 2nd edition s. 9.1 ToPrimitive (hint: Number)
    ///
    /// Flash diverges from spec in a number of ways. These ways are, as far as
    /// we are aware, version-gated:
    ///
    /// * `toString` is never called when `ToPrimitive` is invoked with number
    ///   hint.
    /// * Objects with uncallable `valueOf` implementations are coerced to
    ///   `undefined`. This is not a special-cased behavior: All values are
    ///   callable in `AVM1`. Values that are not callable objects instead
    ///   return `undefined` rather than yielding a runtime error.
    pub fn to_primitive_num(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        Ok(match self {
            Value::Object(object) => {
                let value_of_impl = object
                    .read()
                    .get("valueOf", avm, context, *object)?
                    .resolve(avm, context)?;

                let fake_args = Vec::new();
                value_of_impl
                    .call(avm, context, *object, &fake_args)?
                    .resolve(avm, context)?
            }
            val => val.to_owned(),
        })
    }

    /// ECMA-262 2nd edition s. 11.8.5 Abstract relational comparison algorithm
    #[allow(clippy::float_cmp)]
    pub fn abstract_lt(
        &self,
        other: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<Value<'gc>, Error> {
        let prim_self = self.to_primitive_num(avm, context)?;
        let prim_other = other.to_primitive_num(avm, context)?;

        if let (Value::String(a), Value::String(b)) = (&prim_self, &prim_other) {
            return Ok(a.to_string().bytes().lt(b.to_string().bytes()).into());
        }

        let num_self = prim_self.primitive_as_number(avm, context);
        let num_other = prim_other.primitive_as_number(avm, context);

        if num_self.is_nan() || num_other.is_nan() {
            return Ok(Value::Undefined);
        }

        if num_self == num_other
            || num_self == 0.0 && num_other == -0.0
            || num_self == -0.0 && num_other == 0.0
            || num_self.is_infinite() && num_self.is_sign_positive()
            || num_other.is_infinite() && num_other.is_sign_negative()
        {
            return Ok(false.into());
        }

        if num_self.is_infinite() && num_self.is_sign_negative()
            || num_other.is_infinite() && num_other.is_sign_positive()
        {
            return Ok(true.into());
        }

        Ok((num_self < num_other).into())
    }

    /// ECMA-262 2nd edition s. 11.9.3 Abstract equality comparison algorithm
    pub fn abstract_eq(
        &self,
        other: Value<'gc>,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        coerced: bool,
    ) -> Result<Value<'gc>, Error> {
        match (self, &other) {
            (Value::Undefined, Value::Undefined) => Ok(true.into()),
            (Value::Null, Value::Null) => Ok(true.into()),
            (Value::Number(a), Value::Number(b)) => {
                if !coerced && a.is_nan() && b.is_nan() {
                    return Ok(true.into());
                }

                if a == b {
                    return Ok(true.into());
                }

                if *a == 0.0 && *b == -0.0 || *a == -0.0 && *b == 0.0 {
                    return Ok(true.into());
                }

                Ok(false.into())
            }
            (Value::String(a), Value::String(b)) => Ok((a == b).into()),
            (Value::Bool(a), Value::Bool(b)) => Ok((a == b).into()),
            (Value::Object(a), Value::Object(b)) => Ok(GcCell::ptr_eq(*a, *b).into()),
            (Value::Object(a), Value::Null) | (Value::Object(a), Value::Undefined) => {
                Ok((a.as_ptr() == avm.global_object_cell().as_ptr()).into())
            }
            (Value::Null, Value::Object(b)) | (Value::Undefined, Value::Object(b)) => {
                Ok((b.as_ptr() == avm.global_object_cell().as_ptr()).into())
            }
            (Value::Undefined, Value::Null) => Ok(true.into()),
            (Value::Null, Value::Undefined) => Ok(true.into()),
            (Value::Number(_), Value::String(_)) => Ok(self.abstract_eq(
                Value::Number(other.as_number(avm, context)?),
                avm,
                context,
                true,
            )?),
            (Value::String(_), Value::Number(_)) => {
                Ok(Value::Number(self.as_number(avm, context)?)
                    .abstract_eq(other, avm, context, true)?)
            }
            (Value::Bool(_), _) => Ok(Value::Number(self.as_number(avm, context)?)
                .abstract_eq(other, avm, context, true)?),
            (_, Value::Bool(_)) => Ok(self.abstract_eq(
                Value::Number(other.as_number(avm, context)?),
                avm,
                context,
                true,
            )?),
            (Value::String(_), Value::Object(_)) => {
                let non_obj_other = other.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_other {
                    return Ok(false.into());
                }

                Ok(self.abstract_eq(non_obj_other, avm, context, true)?)
            }
            (Value::Number(_), Value::Object(_)) => {
                let non_obj_other = other.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_other {
                    return Ok(false.into());
                }

                Ok(self.abstract_eq(non_obj_other, avm, context, true)?)
            }
            (Value::Object(_), Value::String(_)) => {
                let non_obj_self = self.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_self {
                    return Ok(false.into());
                }

                Ok(non_obj_self.abstract_eq(other, avm, context, true)?)
            }
            (Value::Object(_), Value::Number(_)) => {
                let non_obj_self = self.to_primitive_num(avm, context)?;
                if let Value::Object(_) = non_obj_self {
                    return Ok(false.into());
                }

                Ok(non_obj_self.abstract_eq(other, avm, context, true)?)
            }
            _ => Ok(false.into()),
        }
    }

    pub fn from_bool_v1(value: bool, swf_version: u8) -> Value<'gc> {
        // SWF version 4 did not have true bools and will push bools as 0 or 1.
        // e.g. SWF19 p. 72:
        // "If the numbers are equal, true is pushed to the stack for SWF 5 and later. For SWF 4, 1 is pushed to the stack."
        if swf_version >= 5 {
            Value::Bool(value)
        } else {
            Value::Number(if value { 1.0 } else { 0.0 })
        }
    }

    /// Coerce a value to a string without calling object methods.
    pub fn into_string(self) -> String {
        match self {
            Value::Undefined => "undefined".to_string(),
            Value::Null => "null".to_string(),
            Value::Bool(v) => v.to_string(),
            Value::Number(v) => v.to_string(), // TODO(Herschel): Rounding for int?
            Value::String(v) => v,
            Value::Object(object) => object.read().as_string(),
        }
    }

    /// Coerce a value to a string.
    pub fn coerce_to_string(
        self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Result<String, Error> {
        Ok(match self {
            Value::Object(object) => {
                let to_string_impl = object
                    .read()
                    .get("toString", avm, context, object)?
                    .resolve(avm, context)?;
                let fake_args = Vec::new();
                match to_string_impl
                    .call(avm, context, object, &fake_args)?
                    .resolve(avm, context)?
                {
                    Value::String(s) => s,
                    _ => "[type Object]".to_string(),
                }
            }
            _ => self.into_string(),
        })
    }

    pub fn as_bool(&self, swf_version: u8) -> bool {
        match self {
            Value::Bool(v) => *v,
            Value::Number(v) => !v.is_nan() && *v != 0.0,
            Value::String(v) => {
                if swf_version >= 7 {
                    !v.is_empty()
                } else {
                    let num = v.parse().unwrap_or(0.0);
                    num != 0.0
                }
            }
            Value::Object(_) => true,
            _ => false,
        }
    }

    pub fn type_of(&self) -> Value<'gc> {
        Value::String(
            match self {
                Value::Undefined => "undefined",
                Value::Null => "null",
                Value::Number(_) => "number",
                Value::Bool(_) => "boolean",
                Value::String(_) => "string",
                Value::Object(object) => object.read().type_of(),
            }
            .to_string(),
        )
    }

    pub fn as_i32(&self) -> Result<i32, Error> {
        self.as_f64().map(|n| n as i32)
    }

    pub fn as_u32(&self) -> Result<u32, Error> {
        self.as_f64().map(|n| n as u32)
    }

    pub fn as_i64(&self) -> Result<i64, Error> {
        self.as_f64().map(|n| n as i64)
    }

    pub fn as_usize(&self) -> Result<usize, Error> {
        self.as_f64().map(|n| n as usize)
    }

    pub fn as_f64(&self) -> Result<f64, Error> {
        match *self {
            Value::Number(v) => Ok(v),
            _ => Err(format!("Expected Number, found {:?}", self).into()),
        }
    }

    pub fn as_string(&self) -> Result<&String, Error> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(format!("Expected String, found {:?}", self).into()),
        }
    }

    pub fn as_object(&self) -> Result<ObjectCell<'gc>, Error> {
        if let Value::Object(object) = self {
            Ok(object.to_owned())
        } else {
            Err(format!("Expected Object, found {:?}", self).into())
        }
    }

    pub fn call(
        &self,
        avm: &mut Avm1<'gc>,
        context: &mut UpdateContext<'_, 'gc, '_>,
        this: ObjectCell<'gc>,
        args: &[Value<'gc>],
    ) -> Result<ReturnValue<'gc>, Error> {
        if let Value::Object(object) = self {
            object.read().call(avm, context, this, args)
        } else {
            Ok(Value::Undefined.into())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::avm1::function::Executable;
    use crate::avm1::globals::create_globals;
    use crate::avm1::object::ObjectCell;
    use crate::avm1::return_value::ReturnValue;
    use crate::avm1::script_object::ScriptObject;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::{Avm1, Error, Value};
    use crate::context::UpdateContext;
    use enumset::EnumSet;
    use gc_arena::GcCell;
    use std::f64::{INFINITY, NAN, NEG_INFINITY};

    #[test]
    fn to_primitive_num() {
        with_avm(6, |avm, context, _this| {
            let t = Value::Bool(true);
            let u = Value::Undefined;
            let f = Value::Bool(false);
            let n = Value::Null;

            assert_eq!(t.to_primitive_num(avm, context).unwrap(), t);
            assert_eq!(u.to_primitive_num(avm, context).unwrap(), u);
            assert_eq!(f.to_primitive_num(avm, context).unwrap(), f);
            assert_eq!(n.to_primitive_num(avm, context).unwrap(), n);

            let (protos, global) = create_globals(context.gc_context);
            let vglobal = Value::Object(global);

            assert_eq!(vglobal.to_primitive_num(avm, context).unwrap(), u);

            fn value_of_impl<'gc>(
                _: &mut Avm1<'gc>,
                _: &mut UpdateContext<'_, 'gc, '_>,
                _: ObjectCell<'gc>,
                _: &[Value<'gc>],
            ) -> Result<ReturnValue<'gc>, Error> {
                Ok(5.0.into())
            }

            let valueof = ScriptObject::function(
                context.gc_context,
                Executable::Native(value_of_impl),
                Some(protos.function),
                None,
            );

            let o = ScriptObject::object_cell(context.gc_context, Some(protos.object));
            o.write(context.gc_context)
                .define_value("valueOf", valueof.into(), EnumSet::empty());

            assert_eq!(
                Value::Object(o).to_primitive_num(avm, context).unwrap(),
                Value::Number(5.0)
            );
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn to_number_swf7() {
        with_avm(7, |avm, context, _this| {
            let t = Value::Bool(true);
            let u = Value::Undefined;
            let f = Value::Bool(false);
            let n = Value::Null;

            assert_eq!(t.as_number(avm, context).unwrap(), 1.0);
            assert!(u.as_number(avm, context).unwrap().is_nan());
            assert_eq!(f.as_number(avm, context).unwrap(), 0.0);
            assert!(n.as_number(avm, context).unwrap().is_nan());

            let bo = Value::Object(GcCell::allocate(
                context.gc_context,
                Box::new(ScriptObject::bare_object()),
            ));

            assert!(bo.as_number(avm, context).unwrap().is_nan());
        });
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn to_number_swf6() {
        with_avm(6, |avm, context, _this| {
            let t = Value::Bool(true);
            let u = Value::Undefined;
            let f = Value::Bool(false);
            let n = Value::Null;

            assert_eq!(t.as_number(avm, context).unwrap(), 1.0);
            assert_eq!(u.as_number(avm, context).unwrap(), 0.0);
            assert_eq!(f.as_number(avm, context).unwrap(), 0.0);
            assert_eq!(n.as_number(avm, context).unwrap(), 0.0);

            let bo = Value::Object(GcCell::allocate(
                context.gc_context,
                Box::new(ScriptObject::bare_object()),
            ));

            assert_eq!(bo.as_number(avm, context).unwrap(), 0.0);
        });
    }

    #[test]
    fn abstract_lt_num() {
        with_avm(8, |avm, context, _this| {
            let a = Value::Number(1.0);
            let b = Value::Number(2.0);

            assert_eq!(a.abstract_lt(b, avm, context).unwrap(), Value::Bool(true));

            let nan = Value::Number(NAN);
            assert_eq!(a.abstract_lt(nan, avm, context).unwrap(), Value::Undefined);

            let inf = Value::Number(INFINITY);
            assert_eq!(a.abstract_lt(inf, avm, context).unwrap(), Value::Bool(true));

            let neg_inf = Value::Number(NEG_INFINITY);
            assert_eq!(
                a.abstract_lt(neg_inf, avm, context).unwrap(),
                Value::Bool(false)
            );

            let zero = Value::Number(0.0);
            assert_eq!(
                a.abstract_lt(zero, avm, context).unwrap(),
                Value::Bool(false)
            );
        });
    }

    #[test]
    fn abstract_gt_num() {
        with_avm(8, |avm, context, _this| {
            let a = Value::Number(1.0);
            let b = Value::Number(2.0);

            assert_eq!(
                b.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Bool(false)
            );

            let nan = Value::Number(NAN);
            assert_eq!(
                nan.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Undefined
            );

            let inf = Value::Number(INFINITY);
            assert_eq!(
                inf.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Bool(false)
            );

            let neg_inf = Value::Number(NEG_INFINITY);
            assert_eq!(
                neg_inf.abstract_lt(a.clone(), avm, context).unwrap(),
                Value::Bool(true)
            );

            let zero = Value::Number(0.0);
            assert_eq!(
                zero.abstract_lt(a, avm, context).unwrap(),
                Value::Bool(true)
            );
        });
    }

    #[test]
    fn abstract_lt_str() {
        with_avm(8, |avm, context, _this| {
            let a = Value::String("a".to_owned());
            let b = Value::String("b".to_owned());

            assert_eq!(a.abstract_lt(b, avm, context).unwrap(), Value::Bool(true))
        })
    }

    #[test]
    fn abstract_gt_str() {
        with_avm(8, |avm, context, _this| {
            let a = Value::String("a".to_owned());
            let b = Value::String("b".to_owned());

            assert_eq!(b.abstract_lt(a, avm, context).unwrap(), Value::Bool(false))
        })
    }
}
