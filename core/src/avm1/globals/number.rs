//! `Number` class impl

use ruffle_macros::istr;

use crate::avm1::activation::Activation;
use crate::avm1::clamp::Clamp;
use crate::avm1::error::Error;
use crate::avm1::object::BoxedF64;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations, SystemClass};
use crate::avm1::{NativeObject, Object, Value};
use crate::string::AvmString;

const PROTO_DECLS: StaticDeclarations = declare_static_properties! {
    "valueOf" => method(value_of; DONT_ENUM | DONT_DELETE);
    "toString" => method(to_string; DONT_ENUM | DONT_DELETE);
};

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "MAX_VALUE" => float(f64::MAX; DONT_ENUM | DONT_DELETE | READ_ONLY);
    // Note this is actually the smallest positive denormalized f64.
    // Rust doesn't provide a constant for this (`MIN_POSITIVE` is a normal f64).
    "MIN_VALUE" => float(5e-324; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "NaN" => float(f64::NAN; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "NEGATIVE_INFINITY" => float(f64::NEG_INFINITY; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "POSITIVE_INFINITY" => float(f64::INFINITY; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn create_class<'gc>(
    context: &mut DeclContext<'_, 'gc>,
    super_proto: Object<'gc>,
) -> SystemClass<'gc> {
    let class = context.native_class(constructor, Some(function), super_proto);
    context.define_properties_on(class.proto, PROTO_DECLS(context));
    context.define_properties_on(class.constr, OBJECT_DECLS(context));
    class
}

/// `Number` constructor
pub fn constructor<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = if let Some(val) = args.get(0) {
        val.coerce_to_f64(activation)?
    } else {
        0.0
    };

    // Called from a constructor, populate `this`.
    let value = BoxedF64::new(activation.gc(), value);
    this.set_native(activation.gc(), NativeObject::Number(value));

    Ok(this.into())
}

/// `Number` function
fn function<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = if let Some(val) = args.get(0) {
        val.coerce_to_f64(activation)?
    } else {
        0.0
    };

    // If Number is called as a function, return the value.
    Ok(value.into())
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Boxed value must be a number. No coercion.
    let NativeObject::Number(number) = this.native() else {
        return Ok(Value::Undefined);
    };
    let number = number.value();

    let radix = match args {
        [] => 10,
        [radix, ..] => {
            let radix = radix.coerce_to_f64(activation)? as i32;
            if (2..=36).contains(&radix) {
                radix
            } else {
                10
            }
        }
    };

    if radix == 10 {
        // Output number as floating-point decimal.
        Ok(Value::from(number).coerce_to_string(activation)?.into())
    } else {
        // Player version specific behavior:
        // `NaN.toString(x)` returns completely garbage values in Flash Player 7+:
        // For example, `NaN.toString(3)` gives "-/.//./..././/0.0./0.".
        // Flash Player 6 returns a much more sane value of 0.
        // TODO: Allow configuration of player version.

        let number = number.clamp_to_i32();

        use std::cmp::Ordering;
        let (mut number, is_negative) = match number.cmp(&0) {
            Ordering::Less => (number.wrapping_neg(), true),
            Ordering::Greater => (number, false),
            Ordering::Equal => {
                // Bail out immediately if we're 0.
                return Ok(istr!("0").into());
            }
        };

        // Max 32 digits in base 2 + negative sign.
        let mut digits = [0; 33];
        let mut i = digits.len();
        while number != 0 {
            let digit = number % radix;
            number /= radix;

            i -= 1;
            digits[i] = if digit < 10 {
                i32::from(b'0') + digit
            } else {
                i32::from(b'a') + digit - 10
            } as u8;
        }
        if is_negative {
            i -= 1;
            digits[i] = b'-';
        }
        Ok(AvmString::new_utf8_bytes(activation.gc(), &digits[i..]).into())
    }
}

fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Boxed value must be a number. No coercion.
    if let NativeObject::Number(number) = this.native() {
        return Ok(number.value().into());
    }

    Ok(Value::Undefined)
}
