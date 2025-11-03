use crate::avm1::{
    activation::{Activation, ActivationIdentifier},
    error::Error,
    Object,
};
use crate::display_object::TDisplayObject;

pub fn with_avm<F>(swf_version: u8, test: F)
where
    F: for<'a, 'gc> FnOnce(&mut Activation<'_, 'gc>, Object<'gc>) -> Result<(), Error<'gc>>,
{
    let movie = crate::tag_utils::SwfMovie::empty(swf_version, None);
    let player = crate::player::PlayerBuilder::new()
        .with_movie(movie)
        .build();
    let mut player = player.lock().unwrap();
    player.mutate_with_update_context(|context| {
        let root = context
            .stage
            .root_clip()
            .expect("Root should exist for freshly made movie");
        let mut activation =
            Activation::from_nothing(context, ActivationIdentifier::root("[Test]"), root);
        let this = root.object1().unwrap();
        let result = test(&mut activation, this);
        if let Err(e) = result {
            panic!("Encountered exception during test: {e}");
        }
    })
}

macro_rules! test_method {
    ( $test: ident, $name: expr, $object: expr, $($versions: expr => { $( $(@epsilon($epsilon: expr))? [$($arg: expr),*] => $out: expr),* }),* ) => {
        #[test]
        fn $test() {
            use $crate::avm1::test_utils::*;
            $(
                for version in &$versions {
                    with_avm(*version, |activation, _root| -> Result<(), Error> {
                        let name = $crate::string::AvmString::new_utf8(activation.gc(), $name);
                        let object = $object(activation);

                        $(
                            let args: &[Value<'_>] = &[$($arg.into()),*];
                            let ret = $crate::avm1::object::Object::call_method(object, name, args, activation, $crate::avm1::function::ExecutionReason::Special)?;
                            test_method!(@__cmp[ $($epsilon)? ] for version in activation, args => ret == $out );
                        )*

                        Ok(())
                    });
                }
            )*
        }
    };

    // Do a numeric comparison with tolerance if `@epsilon` was given:
    ( @__cmp[$epsilon: expr] for $version: ident in $activation: ident, $args: ident => $ret: ident == $out: expr ) => {{
        let epsilon = $epsilon as f64;
        assert!(f64::abs($out as f64 - $ret.coerce_to_f64($activation)?) < epsilon, "@epsilon({:?}) {:?} => {:?} in swf {}", epsilon, $args, $out, $version)
    }};

    // Else, do a generic equality comparison:
    ( @__cmp[] for $version: ident in $activation: ident, $args: ident => $ret: ident == $out: expr ) => {{
        let out = $crate::avm1::Value::from($out);
        assert_eq!($ret, out, "{:?} => {:?} in swf {}", $args, out, $version)
    }}
}
