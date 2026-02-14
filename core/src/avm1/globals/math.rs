use crate::avm1::activation::Activation;
use crate::avm1::error::Error;
use crate::avm1::parameters::ParametersExt as _;
use crate::avm1::property_decl::{DeclContext, StaticDeclarations};
use crate::avm1::{Object, Value};

use std::f64::consts;

const OBJECT_DECLS: StaticDeclarations = declare_static_properties! {
    "SQRT2" => value(consts::SQRT_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "SQRT1_2" => value(consts::FRAC_1_SQRT_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "PI" => value(consts::PI; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LOG2E" => value(consts::LOG2_E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LOG10E" => value(consts::LOG10_E; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LN2" => value(consts::LN_2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "LN10" => value(consts::LN_10; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "E" => value(consts::E; DONT_ENUM | DONT_DELETE | READ_ONLY);

    use fn method;
    "abs" => method(ABS; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "min" => method(MIN; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "max" => method(MAX; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "sin" => method(SIN; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "cos" => method(COS; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "atan2" => method(ATAN2; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "tan" => method(TAN; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "exp" => method(EXP; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "log" => method(LOG; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "sqrt" => method(SQRT; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "round" => method(ROUND; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "random" => method(RANDOM; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "floor" => method(FLOOR; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "ceil" => method(CEIL; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "atan" => method(ATAN; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "asin" => method(ASIN; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "acos" => method(ACOS; DONT_ENUM | DONT_DELETE | READ_ONLY);
    "pow" => method(POW; DONT_ENUM | DONT_DELETE | READ_ONLY);
};

pub fn create<'gc>(context: &mut DeclContext<'_, 'gc>) -> Object<'gc> {
    let math = Object::new(context.strings, Some(context.object_proto));
    context.define_properties_on(math, OBJECT_DECLS(context));
    math
}

pub mod method {
    pub const ABS: u16 = 0;
    pub const MIN: u16 = 1;
    pub const MAX: u16 = 2;
    pub const SIN: u16 = 3;
    pub const COS: u16 = 4;
    pub const ATAN2: u16 = 5;
    pub const TAN: u16 = 6;
    pub const EXP: u16 = 7;
    pub const LOG: u16 = 8;
    pub const SQRT: u16 = 9;
    pub const ROUND: u16 = 10;
    pub const RANDOM: u16 = 11;
    pub const FLOOR: u16 = 12;
    pub const CEIL: u16 = 13;
    pub const ATAN: u16 = 14;
    pub const ASIN: u16 = 15;
    pub const ACOS: u16 = 16;
    pub const POW: u16 = 17;
}

pub fn method<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Object<'gc>,
    args: &[Value<'gc>],
    index: u16,
) -> Result<Value<'gc>, Error<'gc>> {
    use method::*;

    // All Math methods coerce their first two arguments, even when not used.
    let (x, y) = (args.get_f64(activation, 0)?, args.get_f64(activation, 1)?);

    // In SWFv6 and below, method arities are checked.
    if activation.swf_version() <= 6 {
        let valid_arity = match (index, args.len()) {
            (RANDOM, _) => true,
            (MIN | MAX, 0) => true,
            (POW, 1) if x == 1.0 => true,
            (MIN | MAX | POW | ATAN2, 0 | 1) => false,
            (_, 0) => false,
            _ => true,
        };
        if !valid_arity {
            return Ok(f64::NAN.into());
        }
    }

    let result = match index {
        ABS => x.abs(),
        // min/max without arguments returns +/-inf
        MIN if args.is_empty() => f64::INFINITY,
        MAX if args.is_empty() => f64::NEG_INFINITY,
        // Note that Flash Main.min/max propagates NaNs in one argument,
        // unlike Rust's f64::min/max which requires both arguments to be NaNs.
        MIN | MAX if x.is_nan() || y.is_nan() => f64::NAN,
        MIN => x.min(y),
        MAX => x.max(y),
        SIN => x.sin(),
        COS => x.cos(),
        ATAN2 => x.atan2(y),
        TAN => x.tan(),
        EXP => x.exp(),
        LOG => x.ln(),
        SQRT => x.sqrt(),
        // Note that Flash Math.round always rounds toward infinity,
        // unlike Rust's f64::round which rounds away from zero.
        ROUND => (x + 0.5).floor(),
        // See https://github.com/adobe/avmplus/blob/858d034a3bd3a54d9b70909386435cf4aec81d21/core/MathUtils.cpp#L1731C24-L1731C44
        // This generated a restricted set of 'f64' values, which some SWFs implicitly rely on.
        RANDOM => {
            const MAX_VAL: u32 = 0x7FFFFFFF;
            let rand = activation.context.rng.generate_random_number();
            (rand as f64) / (MAX_VAL as f64 + 1f64)
        }
        FLOOR => x.floor(),
        CEIL => x.ceil(),
        ATAN => x.atan(),
        ASIN => x.asin(),
        ACOS => x.acos(),
        POW => x.powf(y),
        // Unlike most ASnative methods, Math returns NaN for unknown indices.
        _ => f64::NAN,
    };

    Ok(result.into())
}
