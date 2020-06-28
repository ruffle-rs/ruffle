//! `Number` class impl

use crate::avm1::error::Error;
use crate::avm1::function::{Executable, FunctionObject};
use crate::avm1::property::Attribute::*;
use crate::avm1::stack_frame::StackFrame;
use crate::avm1::value_object::ValueObject;
use crate::avm1::{Object, TObject, Value};
use crate::context::UpdateContext;
use enumset::EnumSet;
use gc_arena::MutationContext;

/// `Number` constructor/function
pub fn number<'gc>(
    activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
    this: Object<'gc>,
    args: &[Value<'gc>],
) -> Result<Value<'gc>, Error<'gc>> {
    let value = if let Some(val) = args.get(0) {
        val.coerce_to_f64(activation, context)?
    } else {
        0.0
    };

    // If called from a constructor, populate `this`.
    if let Some(mut vbox) = this.as_value_object() {
        vbox.replace_value(context.gc_context, value.into());
    }

    // If Number is called as a function, return the value.
    Ok(value.into())
}

pub fn create_number_object<'gc>(
    gc_context: MutationContext<'gc, '_>,
    number_proto: Option<Object<'gc>>,
    fn_proto: Option<Object<'gc>>,
) -> Object<'gc> {
    let number = FunctionObject::function(
        gc_context,
        Executable::Native(number),
        fn_proto,
        number_proto,
    );
    let object = number.as_script_object().unwrap();

    object.define_value(
        gc_context,
        "MAX_VALUE",
        std::f64::MAX.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    object.define_value(
        gc_context,
        "MIN_VALUE",
        // Note this is actually the smallest positive denormalized f64.
        // Rust doesn't provide a constant for this (`MIN_POSITIVE` is a normal f64).
        Value::Number(f64::from_bits(1)),
        DontDelete | ReadOnly | DontEnum,
    );

    object.define_value(
        gc_context,
        "NaN",
        std::f64::NAN.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    object.define_value(
        gc_context,
        "NEGATIVE_INFINITY",
        std::f64::NEG_INFINITY.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    object.define_value(
        gc_context,
        "POSITIVE_INFINITY",
        std::f64::INFINITY.into(),
        DontDelete | ReadOnly | DontEnum,
    );

    number
}

/// Creates `Number.prototype`.
pub fn create_proto<'gc>(
    gc_context: MutationContext<'gc, '_>,
    proto: Object<'gc>,
    fn_proto: Object<'gc>,
) -> Object<'gc> {
    let number_proto = ValueObject::empty_box(gc_context, Some(proto));
    let mut object = number_proto.as_script_object().unwrap();

    object.force_set_function(
        "toString",
        to_string,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );
    object.force_set_function(
        "valueOf",
        value_of,
        gc_context,
        EnumSet::empty(),
        Some(fn_proto),
    );

    number_proto
}

fn to_string<'gc>(
    activation: &mut StackFrame<'_, 'gc>,
    context: &mut UpdateContext<'_, 'gc, '_>,
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
            .coerce_to_f64(activation, context)?;
        if radix >= 2.0 && radix <= 36.0 {
            radix as u32
        } else {
            10
        }
    };

    if radix == 10 {
        // Output number as floating-point decimal.
        Ok(Value::from(this)
            .coerce_to_string(activation, context)?
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
        Ok(out.into())
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
    _activation: &mut StackFrame<'_, 'gc>,
    _context: &mut UpdateContext<'_, 'gc, '_>,
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
//     "-/--,,..-,-,0,-",
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
