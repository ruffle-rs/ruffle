use crate::avm1::fscommand;
use crate::avm1::return_value::ReturnValue;
use crate::avm1::{Avm1, Error, Object, UpdateContext, Value};
use crate::backend::navigator::NavigationMethod;
use enumset::EnumSet;
use gc_arena::{GcCell, MutationContext};
use rand::Rng;

mod function;
mod math;
mod movie_clip;
mod object;

#[allow(non_snake_case, unused_must_use)] //can't use errors yet
pub fn getURL<'a, 'gc>(
    avm: &mut Avm1<'gc>,
    context: &mut UpdateContext<'a, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    //TODO: Error behavior if no arguments are present
    if let Some(url_val) = args.get(0) {
        let url = url_val.clone().into_string();
        if let Some(fscommand) = fscommand::parse(&url) {
            fscommand::handle(fscommand, avm, context);
            return Ok(Value::Undefined.into());
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

    Ok(Value::Undefined.into())
}

pub fn random<'gc>(
    _avm: &mut Avm1<'gc>,
    action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    match args.get(0) {
        Some(Value::Number(max)) => Ok(action_context.rng.gen_range(0.0f64, max).floor().into()),
        _ => Ok(Value::Undefined.into()), //TODO: Shouldn't this be an error condition?
    }
}

pub fn boolean<'gc>(
    avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(val) = args.get(0) {
        Ok(val.as_bool(avm.current_swf_version()).into())
    } else {
        Ok(false.into())
    }
}

pub fn number<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(val) = args.get(0) {
        Ok(val.as_number().into())
    } else {
        Ok(0.0.into())
    }
}

pub fn is_nan<'gc>(
    _avm: &mut Avm1<'gc>,
    _action_context: &mut UpdateContext<'_, 'gc, '_>,
    _this: GcCell<'gc, Object<'gc>>,
    args: &[Value<'gc>],
) -> Result<ReturnValue<'gc>, Error> {
    if let Some(val) = args.get(0) {
        Ok(val.as_number().is_nan().into())
    } else {
        Ok(true.into())
    }
}

/// This structure represents all system builtins that are used regardless of
/// whatever the hell happens to `_global`. These are, of course,
/// user-modifiable.
#[derive(Clone)]
pub struct SystemPrototypes<'gc> {
    pub object: GcCell<'gc, Object<'gc>>,
    pub function: GcCell<'gc, Object<'gc>>,
    pub movie_clip: GcCell<'gc, Object<'gc>>,
}

unsafe impl<'gc> gc_arena::Collect for SystemPrototypes<'gc> {
    #[inline]
    fn trace(&self, cc: gc_arena::CollectionContext) {
        self.object.trace(cc);
        self.function.trace(cc);
        self.movie_clip.trace(cc);
    }
}

/// Initialize default global scope and builtins for an AVM1 instance.
pub fn create_globals<'gc>(
    gc_context: MutationContext<'gc, '_>,
) -> (SystemPrototypes<'gc>, Object<'gc>) {
    let object_proto = GcCell::allocate(gc_context, Object::bare_object());
    let function_proto = function::create_proto(gc_context, object_proto);

    object::fill_proto(gc_context, object_proto, function_proto);

    let movie_clip_proto = movie_clip::create_proto(gc_context, object_proto, function_proto);

    let object = GcCell::allocate(
        gc_context,
        Object::native_function(
            |_, _, _, _| Ok(Value::Undefined.into()),
            Some(function_proto),
        ),
    );
    let function = GcCell::allocate(
        gc_context,
        Object::native_function(
            |_, _, _, _| Ok(Value::Undefined.into()),
            Some(function_proto),
        ),
    );
    let movie_clip = GcCell::allocate(
        gc_context,
        Object::native_function(
            |_, _, _, _| Ok(Value::Undefined.into()),
            Some(movie_clip_proto),
        ),
    );

    let mut globals = Object::object(gc_context, Some(object_proto));
    globals.force_set("Object", Value::Object(object), EnumSet::empty());
    globals.force_set("Function", Value::Object(function), EnumSet::empty());
    globals.force_set("MovieClip", Value::Object(movie_clip), EnumSet::empty());
    globals.force_set_function(
        "Number",
        number,
        gc_context,
        EnumSet::empty(),
        Some(function_proto),
    );
    globals.force_set_function(
        "Boolean",
        boolean,
        gc_context,
        EnumSet::empty(),
        Some(function_proto),
    );
    globals.force_set(
        "Math",
        Value::Object(math::create(
            gc_context,
            Some(object_proto),
            Some(function_proto),
        )),
        EnumSet::empty(),
    );
    globals.force_set_function(
        "isNaN",
        is_nan,
        gc_context,
        EnumSet::empty(),
        Some(function_proto),
    );
    globals.force_set_function(
        "getURL",
        getURL,
        gc_context,
        EnumSet::empty(),
        Some(function_proto),
    );
    globals.force_set_function(
        "random",
        random,
        gc_context,
        EnumSet::empty(),
        Some(function_proto),
    );
    globals.force_set("NaN", Value::Number(std::f64::NAN), EnumSet::empty());
    globals.force_set(
        "Infinity",
        Value::Number(std::f64::INFINITY),
        EnumSet::empty(),
    );

    (
        SystemPrototypes {
            object: object_proto,
            function: function_proto,
            movie_clip: movie_clip_proto,
        },
        globals,
    )
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
                        assert_eq!($fun(avm, context, this, &args).unwrap(), ReturnValue::Immediate($out.into()));
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
