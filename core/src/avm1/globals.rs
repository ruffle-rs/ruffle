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
    globals.force_set(
        "Math",
        Value::Object(math::create(gc_context)),
        EnumSet::empty(),
    );
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
mod tests {
    use super::*;
    use crate::avm1::activation::Activation;
    use crate::avm1::Error;
    use crate::backend::audio::NullAudioBackend;
    use crate::backend::navigator::NullNavigatorBackend;
    use crate::display_object::DisplayObject;
    use crate::movie_clip::MovieClip;
    use gc_arena::rootless_arena;
    use rand::{rngs::SmallRng, SeedableRng};

    macro_rules! test_std {
        ( $test: ident, $fun: expr, $version: expr, $($args: expr => $out: expr),* ) => {
            #[test]
            fn $test() -> Result<(), Error> {
                with_avm($version, |avm, context, this| {

                    $(
                        assert_eq!($fun(avm, context, this, $args), $out);
                    )*

                    Ok(())
                })
            }
        };
    }

    fn with_avm<F, R>(swf_version: u8, test: F) -> R
    where
        F: for<'a, 'gc> FnOnce(
            &mut Avm1<'gc>,
            &mut ActionContext<'a, 'gc, '_>,
            GcCell<'gc, Object<'gc>>,
        ) -> R,
    {
        rootless_arena(|gc_context| {
            let mut avm = Avm1::new(gc_context);
            let movie_clip: Box<dyn DisplayObject> = Box::new(MovieClip::new(gc_context));
            let root = GcCell::allocate(gc_context, movie_clip);
            let mut context = ActionContext {
                gc_context,
                global_time: 0,
                root,
                start_clip: root,
                active_clip: root,
                target_clip: Some(root),
                target_path: Value::Undefined,
                rng: &mut SmallRng::from_seed([0u8; 16]),
                audio: &mut NullAudioBackend::new(),
                navigator: &mut NullNavigatorBackend::new(),
            };

            let globals = avm.global_object_cell();
            avm.insert_stack_frame(Activation::from_nothing(swf_version, globals, gc_context));

            let this = root.read().object().as_object().unwrap().to_owned();

            test(&mut avm, &mut context, this)
        })
    }

    test_std!(boolean_function, boolean, 19,
        &[Value::Bool(true)] => Value::Bool(true),
        &[Value::Bool(false)] => Value::Bool(false),
        &[Value::Number(10.0)] => Value::Bool(true),
        &[Value::Number(-10.0)] => Value::Bool(true),
        &[Value::Number(0.0)] => Value::Bool(false),
        &[Value::Number(std::f64::INFINITY)] => Value::Bool(true),
        &[Value::Number(std::f64::NAN)] => Value::Bool(false),
        &[Value::String("".to_string())] => Value::Bool(false),
        &[Value::String("Hello".to_string())] => Value::Bool(true),
        &[Value::String(" ".to_string())] => Value::Bool(true),
        &[Value::String("0".to_string())] => Value::Bool(true),
        &[Value::String("1".to_string())] => Value::Bool(true),
        &[] => Value::Bool(false)
    );

    test_std!(boolean_function_swf6, boolean, 6,
        &[Value::Bool(true)] => Value::Bool(true),
        &[Value::Bool(false)] => Value::Bool(false),
        &[Value::Number(10.0)] => Value::Bool(true),
        &[Value::Number(-10.0)] => Value::Bool(true),
        &[Value::Number(0.0)] => Value::Bool(false),
        &[Value::Number(std::f64::INFINITY)] => Value::Bool(true),
        &[Value::Number(std::f64::NAN)] => Value::Bool(false),
        &[Value::String("".to_string())] => Value::Bool(false),
        &[Value::String("Hello".to_string())] => Value::Bool(false),
        &[Value::String(" ".to_string())] => Value::Bool(false),
        &[Value::String("0".to_string())] => Value::Bool(false),
        &[Value::String("1".to_string())] => Value::Bool(true),
        &[] => Value::Bool(false)
    );

    test_std!(is_nan_function, is_nan, 19,
        &[Value::Bool(true)] => Value::Bool(false),
        &[Value::Bool(false)] => Value::Bool(false),
        &[Value::Number(10.0)] => Value::Bool(false),
        &[Value::Number(-10.0)] => Value::Bool(false),
        &[Value::Number(0.0)] => Value::Bool(false),
        &[Value::Number(std::f64::INFINITY)] => Value::Bool(false),
        &[Value::Number(std::f64::NAN)] => Value::Bool(true),
        &[Value::String("".to_string())] => Value::Bool(false),
        &[Value::String("Hello".to_string())] => Value::Bool(true),
        &[Value::String(" ".to_string())] => Value::Bool(true),
        &[Value::String("  5  ".to_string())] => Value::Bool(true),
        &[Value::String("0".to_string())] => Value::Bool(false),
        &[Value::String("1".to_string())] => Value::Bool(false),
        &[Value::String("Infinity".to_string())] => Value::Bool(true),
        &[Value::String("100a".to_string())] => Value::Bool(true),
        &[Value::String("0x10".to_string())] => Value::Bool(false),
        &[Value::String("0xhello".to_string())] => Value::Bool(true),
        &[Value::String("123e-1".to_string())] => Value::Bool(false),
        &[] => Value::Bool(true)
    );

    test_std!(number_function, number, 19,
        &[Value::Bool(true)] => Value::Number(1.0),
        &[Value::Bool(false)] => Value::Number(0.0),
        &[Value::Number(10.0)] => Value::Number(10.0),
        &[Value::Number(-10.0)] => Value::Number(-10.0),
        &[Value::Number(0.0)] => Value::Number(0.0),
        &[Value::Number(std::f64::INFINITY)] => Value::Number(std::f64::INFINITY),
        &[Value::Number(std::f64::NAN)] => Value::Number(std::f64::NAN),
        &[Value::String("".to_string())] => Value::Number(0.0),
        &[Value::String("Hello".to_string())] => Value::Number(std::f64::NAN),
        &[Value::String(" ".to_string())] => Value::Number(std::f64::NAN),
        &[Value::String("  5  ".to_string())] => Value::Number(std::f64::NAN),
        &[Value::String("0".to_string())] => Value::Number(0.0),
        &[Value::String("1".to_string())] => Value::Number(1.0),
        &[Value::String("Infinity".to_string())] => Value::Number(std::f64::NAN),
        &[Value::String("100a".to_string())] => Value::Number(std::f64::NAN),
        &[Value::String("0x10".to_string())] => Value::Number(16.0),
        &[Value::String("0xhello".to_string())] => Value::Number(std::f64::NAN),
        &[Value::String("123e-1".to_string())] => Value::Number(12.3),
        &[] => Value::Number(0.0)
    );

}
