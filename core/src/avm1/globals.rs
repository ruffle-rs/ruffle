use crate::avm1::fscommand;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Object, UpdateContext, Value};
use crate::backend::navigator::NavigationMethod;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};
use rand::Rng;

mod math;

#[allow(non_snake_case, unused_must_use)] //can't use errors yet
pub fn getURL<'a, 'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'a, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> ReturnValue<'gc> {
    //TODO: Error behavior if no arguments are present
    if let Some(url_val) = args.get(0) {
        let url = url_val.clone().into_string();
        if let Some(fscommand) = fscommand::parse(&url) {
            fscommand::handle(fscommand, avm, context);
            return ReturnValue::Immediate(Value::Undefined);
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

    ReturnValue::Immediate(Value::Undefined)
}

pub fn random<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> ReturnValue<'gc> {
    match args.get(0) {
        Some(Value::Number(max)) => ReturnValue::Immediate(Value::Number(
            action_context.rng.gen_range(0.0f64, max).floor(),
        )),
        _ => ReturnValue::Immediate(Value::Undefined), //TODO: Shouldn't this be an error condition?
    }
}

pub fn boolean<'gc>(
    avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> ReturnValue<'gc> {
    if let Some(val) = args.get(0) {
        ReturnValue::Immediate(Value::Bool(val.as_bool(avm.current_swf_version())))
    } else {
        ReturnValue::Immediate(Value::Bool(false))
    }
}

pub fn number<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> ReturnValue<'gc> {
    if let Some(val) = args.get(0) {
        ReturnValue::Immediate(Value::Number(val.as_number()))
    } else {
        ReturnValue::Immediate(Value::Number(0.0))
    }
}

pub fn is_nan<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> ReturnValue<'gc> {
    if let Some(val) = args.get(0) {
        ReturnValue::Immediate(Value::Bool(val.as_number().is_nan()))
    } else {
        ReturnValue::Immediate(Value::Bool(true))
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
                        assert_eq!($fun(avm, context, this, &args), ReturnValue::Immediate($out.into()));
                    )*

                    Ok(())
                })
            }
        };
    }

    test_std!(boolean_function, boolean, 19,
        [true] => true,
        [false] => false,
        [10.0] => true,
        [-10.0] => true,
        [0.0] => false,
        [std::f64::INFINITY] => true,
        [std::f64::NAN] => false,
        [""] => false,
        ["Hello"] => true,
        [" "] => true,
        ["0"] => true,
        ["1"] => true,
        [] => false
    );

    test_std!(boolean_function_swf6, boolean, 6,
        [true] => true,
        [false] => false,
        [10.0] => true,
        [-10.0] => true,
        [0.0] => false,
        [std::f64::INFINITY] => true,
        [std::f64::NAN] => false,
        [""] => false,
        ["Hello"] => false,
        [" "] => false,
        ["0"] => false,
        ["1"] => true,
        [] => false
    );

    test_std!(is_nan_function, is_nan, 19,
        [true] => false,
        [false] => false,
        [10.0] => false,
        [-10.0] => false,
        [0.0] => false,
        [std::f64::INFINITY] => false,
        [std::f64::NAN] => true,
        [""] => false,
        ["Hello"] => true,
        [" "] => true,
        ["  5  "] => true,
        ["0"] => false,
        ["1"] => false,
        ["Infinity"] => true,
        ["100a"] => true,
        ["0x10"] => false,
        ["0xhello"] => true,
        ["0x1999999981ffffff"] => false,
        ["0xUIXUIDFKHJDF012345678"] => true,
        ["123e-1"] => false,
        [] => true
    );

    test_std!(number_function, number, 19,
        [true] => 1.0,
        [false] => 0.0,
        [10.0] => 10.0,
        [-10.0] => -10.0,
        [0.0] => 0.0,
        [std::f64::INFINITY] => std::f64::INFINITY,
        [std::f64::NAN] => std::f64::NAN,
        [""] => 0.0,
        ["Hello"] => std::f64::NAN,
        [" "] => std::f64::NAN,
        ["  5  "] => std::f64::NAN,
        ["0"] => 0.0,
        ["1"] => 1.0,
        ["Infinity"] => std::f64::NAN,
        ["100a"] => std::f64::NAN,
        ["0x10"] => 16.0,
        ["0xhello"] => std::f64::NAN,
        ["123e-1"] => 12.3,
        ["0x1999999981ffffff"] => -2113929217.0,
        ["0xUIXUIDFKHJDF012345678"] => std::f64::NAN,
        [] => 0.0
    );
}
