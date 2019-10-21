use crate::avm1::fscommand;
use crate::avm1::{ActionContext, Avm1, Object, Value};
use crate::backend::navigator::NavigationMethod;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};
use rand::Rng;

mod math;

#[allow(non_snake_case, unused_must_use)] //can't use errors yet
pub fn getURL<'a, 'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut ActionContext<'a, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    //TODO: Error behavior if no arguments are present
    if let Some(url_val) = args.get(0) {
        let url = url_val.clone().into_string();
        if let Some(fscommand) = fscommand::parse(&url) {
            fscommand::handle(fscommand, avm, context);
            return Value::Undefined;
        }

        let window = args.get(1).map(|v| v.clone().into_string());
        let method = match args.get(2) {
            Some(Value::String(s)) if s == "GET" => Some(NavigationMethod::GET),
            Some(Value::String(s)) if s == "POST" => Some(NavigationMethod::POST),
            _ => None,
        };
        let vars_method = method.map(|m| (m, avm.locals_into_form_values(context)));

        context.navigator.navigate_to_url(url, window, vars_method);
    }

    Value::Undefined
}

pub fn random<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    match args.get(0) {
        Some(Value::Number(max)) => {
            Value::Number(action_context.rng.gen_range(0.0f64, max).floor())
        }
        _ => Value::Undefined, //TODO: Shouldn't this be an error condition?
    }
}

pub fn boolean<'gc>(
    avm: &mut Avm1<'gc>,
    _action_context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    if let Some(val) = args.get(0) {
        Value::Bool(val.as_bool(avm.current_swf_version()))
    } else {
        Value::Bool(false)
    }
}

pub fn number<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    if let Some(val) = args.get(0) {
        Value::Number(val.as_number())
    } else {
        Value::Number(0.0)
    }
}

pub fn is_nan<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut ActionContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Value<'gc> {
    if let Some(val) = args.get(0) {
        Value::Bool(val.as_number().is_nan())
    } else {
        Value::Bool(true)
    }
}

pub fn create_globals<'gc>(gc_context: MutationContext<'gc, '_>) -> Object<'gc> {
    let mut globals = Object::object(gc_context);

    globals.force_set_function("isNaN", is_nan, gc_context, EnumSet::empty());
    globals.force_set_function("Boolean", boolean, gc_context, EnumSet::empty());
    globals.force_set("Math", math::create(gc_context), EnumSet::empty());
    globals.force_set_function("getURL", getURL, gc_context, EnumSet::empty());
    globals.force_set_function("Number", number, gc_context, EnumSet::empty());
    globals.force_set_function("random", random, gc_context, EnumSet::empty());

    globals.force_set("NaN", Value::Number(std::f64::NAN), EnumSet::empty());
    globals.force_set(
        "Infinity",
        Value::Number(std::f64::INFINITY),
        EnumSet::empty(),
    );

    globals
}

#[cfg(test)]
#[allow(clippy::unreadable_literal)]
mod tests {
    use super::*;
    use crate::avm1::test_utils::with_avm;
    use crate::avm1::Error;

    macro_rules! test_std {
        ( $test: ident, $fun: expr, $version: expr, $([$($arg: expr),*] => $out: expr),* ) => {
            #[test]
            fn $test() -> Result<(), Error> {
                with_avm($version, |avm, context, this| {

                    $(
                        #[allow(unused_mut)]
                        let mut args: Vec<Value> = Vec::new();
                        $(
                            args.push($arg.into());
                        )*
                        assert_eq!($fun(avm, context, this, &args), $out);
                    )*

                    Ok(())
                })
            }
        };
    }

    test_std!(boolean_function, boolean, 19,
        [true] => Value::Bool(true),
        [false] => Value::Bool(false),
        [Value::Number(10.0)] => Value::Bool(true),
        [Value::Number(-10.0)] => Value::Bool(true),
        [Value::Number(0.0)] => Value::Bool(false),
        [Value::Number(std::f64::INFINITY)] => Value::Bool(true),
        [Value::Number(std::f64::NAN)] => Value::Bool(false),
        [""] => Value::Bool(false),
        ["Hello"] => Value::Bool(true),
        [" "] => Value::Bool(true),
        ["0"] => Value::Bool(true),
        ["1"] => Value::Bool(true),
        [] => Value::Bool(false)
    );

    test_std!(boolean_function_swf6, boolean, 6,
        [true] => Value::Bool(true),
        [false] => Value::Bool(false),
        [Value::Number(10.0)] => Value::Bool(true),
        [Value::Number(-10.0)] => Value::Bool(true),
        [Value::Number(0.0)] => Value::Bool(false),
        [Value::Number(std::f64::INFINITY)] => Value::Bool(true),
        [Value::Number(std::f64::NAN)] => Value::Bool(false),
        [""] => Value::Bool(false),
        ["Hello"] => Value::Bool(false),
        [" "] => Value::Bool(false),
        ["0"] => Value::Bool(false),
        ["1"] => Value::Bool(true),
        [] => Value::Bool(false)
    );

    test_std!(is_nan_function, is_nan, 19,
        [true] => Value::Bool(false),
        [false] => Value::Bool(false),
        [Value::Number(10.0)] => Value::Bool(false),
        [Value::Number(-10.0)] => Value::Bool(false),
        [Value::Number(0.0)] => Value::Bool(false),
        [Value::Number(std::f64::INFINITY)] => Value::Bool(false),
        [Value::Number(std::f64::NAN)] => Value::Bool(true),
        [""] => Value::Bool(false),
        ["Hello"] => Value::Bool(true),
        [" "] => Value::Bool(true),
        ["  5  "] => Value::Bool(true),
        ["0"] => Value::Bool(false),
        ["1"] => Value::Bool(false),
        ["Infinity"] => Value::Bool(true),
        ["100a"] => Value::Bool(true),
        ["0x10"] => Value::Bool(false),
        ["0xhello"] => Value::Bool(true),
        ["0x1999999981ffffff"] => Value::Bool(false),
        ["0xUIXUIDFKHJDF012345678"] => Value::Bool(true),
        ["123e-1"] => Value::Bool(false),
        [] => Value::Bool(true)
    );

    test_std!(number_function, number, 19,
        [true] => Value::Number(1.0),
        [false] => Value::Number(0.0),
        [Value::Number(10.0)] => Value::Number(10.0),
        [Value::Number(-10.0)] => Value::Number(-10.0),
        [Value::Number(0.0)] => Value::Number(0.0),
        [Value::Number(std::f64::INFINITY)] => Value::Number(std::f64::INFINITY),
        [Value::Number(std::f64::NAN)] => Value::Number(std::f64::NAN),
        [""] => Value::Number(0.0),
        ["Hello"] => Value::Number(std::f64::NAN),
        [" "] => Value::Number(std::f64::NAN),
        ["  5  "] => Value::Number(std::f64::NAN),
        ["0"] => Value::Number(0.0),
        ["1"] => Value::Number(1.0),
        ["Infinity"] => Value::Number(std::f64::NAN),
        ["100a"] => Value::Number(std::f64::NAN),
        ["0x10"] => Value::Number(16.0),
        ["0xhello"] => Value::Number(std::f64::NAN),
        ["123e-1"] => Value::Number(12.3),
        ["0x1999999981ffffff"] => Value::Number(-2113929217.0),
        ["0xUIXUIDFKHJDF012345678"] => Value::Number(std::f64::NAN),
        [] => Value::Number(0.0)
    );
}
