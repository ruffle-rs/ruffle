//! `Number` class impl

use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::object::value_object::ValueObject;
use crate::avm1::property_decl::{define_properties_on, Declaration};
use crate::avm1::{Object, TObject, Value};
use crate::string::AvmString;
use gc_arena::MutationContext;

const PROTO_DECLS: &[Declaration] = declare_properties! {
    "toString" => method(to_string);
    "valueOf" => method(value_of);
};

const OBJECT_DECLS: &[Declaration] = declare_properties! {
    "MAX_VALUE" => float(f64::MAX; DONT_ENUM | DONT_DELETE | READ_ONLY);
    // Note this is actually the smallest positive denormalized f64.
    // Rust doesn't provide a constant for this (`MIN_POSITIVE` is a normal f64).
    "MIN_VALUE" => float(5e-324; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "NaN" => float(f64::NAN; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "NEGATIVE_INFINITY" => float(f64::NEG_INFINITY; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "POSITIVE_INFINITY" => float(f64::INFINITY; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

/// `Number` constructor
pub fn number<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = if let Some(val) = args.get(0) {
        val.coerce_to_f64(activation)?
    } else {
        0.0
    };

    // If called from a constructor, populate `this`.
    if let Some(mut vbox) = this.as_value_object() {
        vbox.replace_value(activation.context.gc_context, value.into());
    }

    Ok(this.into())
}

/// `Number` function
pub fn number_function<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
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

pub fn create_number_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    number_proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let number = FunctionObject::constructor(
        gc_context,
        Executable::Native(number),
        Executable::Native(number_function),
        Some(fn_proto),
        number_proto,
    );
    let object = number.as_script_object().unwrap();
    define_properties_on(OBJECT_DECLS, gc_context, object, fn_proto);
    number
}

/// Creates `Number.prototype`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let number_proto = ValueObject::empty_box(gc_context, Some(proto));
    let object = number_proto.as_script_object().unwrap();
    define_properties_on(PROTO_DECLS, gc_context, object, fn_proto);
    number_proto
}

fn to_string<'gc>(
    activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    // Boxed value must be a number. No coercion.
    let this = if let Some(vbox) = this.as_value_object() {
        if let Value::Number(n) = vbox.unbox() {
            n
        } else {
            return Ok(Value::Undefined);
        }
    } else {
        return Ok(Value::Undefined);
    };

    let radix = {
        let radix = args
            .get(0)
            .unwrap_or(&Value::Undefined)
            .coerce_to_f64(activation)?;
        if radix >= 2.0 && radix <= 36.0 {
            radix as u32
        } else {
            10
        }
    };

    if radix == 10 {
        // Output number as floating-point decimal.
        Ok(AvmString::new(
            activation.context.gc_context,
            Value::from(this).coerce_to_string(activation)?.to_string(),
        )
        .into())
    } else if this > -2_147_483_648.0 && this < 2_147_483_648.0 {
        // Output truncated integer in specified base.
        let n = this as i32;
        use std::cmp::Ordering;
        let (mut n, is_negative) = match n.cmp(&0) {
            Ordering::Less => ((-n) as u32, true),
            Ordering::Greater => (n as u32, false),
            Ordering::Equal => {
                // Bail out immediately if we're 0.
                return Ok("0".into());
            }
        };

        // Max 32 digits in base 2 + negative sign.
        let mut digits = ['\0'; 33];
        let mut i = 0;
        while n > 0 {
            let digit = n % radix;
            n /= radix;
            digits[i] = std::char::from_digit(digit, radix).unwrap();
            i += 1;
        }
        if is_negative {
            digits[i] = '-';
            i += 1;
        }
        let out: String = digits[..i].iter().rev().collect();
        Ok(AvmString::new(activation.context.gc_context, out).into())
    } else {
        // NaN or large numbers.
        // Player version specific behavior:
        // NaN.toString(x) will print completely garbage values in Flash Player 7+:
        // for example, NaN.toString(3) gives "-/.//./..././/0.0./0.".
        // Flash Player 6 will print a much more sane value of 0, so let's go with that.
        // TODO: Allow configuration of player version.
        Ok("0".into())
    }
}

fn value_of<'gc>(
    _activation: &mut Activation<'_, 'gc, '_>,
    this: Object<'gc>,
    _args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    if let Some(vbox) = this.as_value_object() {
        if let Value::Number(n) = vbox.unbox() {
            return Ok(n.into());
        }
    }

    Ok(Value::Undefined)
}

// The values returned by `NaN.toString(radix)` in Flash Player v7+
// for each radix from 2 to 36. Currently unused, but leaving it here
// in case we want to emulate this behavior.
// This table was generated in Flash.
// Not sure where the heck these values actually come from...!?
// const TO_STRING_NANS: &[&str] = &[
//     "-/0000000000000000000000000000000",
//     "-/.//./..././/0.0./0.",
//     "-.000000000000000",
//     "-/--,..-,-,0,-",
//     "-++-0-.00++-.",
//     "-/0,/-,.///*.",
//     "-.0000000000",
//     "-+,)())-*).",
//     "NaN",
//     "-&0...0.(.",
//     "-,%%.-0(&(",
//     "-.(.%&,&&%",
//     "-/*+.$&'-.",
//     "-$()\x22**%(",
//     "-(0000000",
//     "-+- )!+,'",
//     "--'.( -\x1F.",
//     "-.)$+)\x1F--",
//     "-/#%/!'.(",
//     "-/,0\x1F.#'.",
//     "-\x1E\x1C!+%!.",
//     "-\x22%\x22\x1B!'*",
//     "-%+  \x22+(",
//     "-(\x1D\x1A#\x19\x1C\x19",
//     "-*\x18\x1D(\x1E\x18\x18",
//     "-+\x22\x1F\x19$\x1C%",
//     "-,$\x1B\x1A'( ",
//     "--\x1F\x1C)'((",
//     "-.\x14%*$\x14(",
//     "-.#0'\x12$.",
//     "-.000000",
//     "-/\x1B\x14\x16\x13\x1B.",
//     "-/#(\x0F\x16\x15\x16",
//     "-/+\x11..\x12\x19",
//     "-\x0D\x1E\x1C0\x0D\x1C",
// ];
//
