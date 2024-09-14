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
    let movie = crate::tag_utils::SwfMovie::empty(swf_version);
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
        let this = root.object().coerce_to_object(&mut activation);
        let result = test(&mut activation, this);
        if let Err(e) = result {
            panic!("Encountered exception during test: {e}");
        }
    })
}

macro_rules! test_method {
    ( $test: ident, $name: expr, $object: expr, $($versions: expr => { $( $(@epsilon($epsilon: expr))? [$($arg: expr),*] => $out: expr),* }),* ) => {
        #[test]
        #[allow(unreachable_code)] // the `assert_eq!` at the end, in expansions without `@epsilon`
        fn $test() {
            use $crate::avm1::test_utils::*;
            $(
                for version in &$versions {
                    with_avm(*version, |activation, _root| -> Result<(), Error> {
                        let name: $crate::string::AvmString<'_> = $name.into();
                        let object = $object(activation);

                        $(
                            let args: Vec<Value> = vec![$($arg.into()),*];
                            let ret = crate::avm1::object::TObject::call_method(&object, name, &args, activation, crate::avm1::function::ExecutionReason::Special)?;

                            // Do a numeric comparison with tolerance if `@epsilon` was given:
                            $(
                                assert!(f64::abs($out as f64 - ret.coerce_to_f64(activation)?) < $epsilon as f64, "@epsilon({:?}) {:?} => {:?} in swf {}", $epsilon, args, $out, version);
                                return Ok(());
                            )?
                            // Else, do a generic equality comparison:
                            assert_eq!(ret, $out.into(), "{:?} => {:?} in swf {}", args, $out, version);
                        )*

                        Ok(())
                    });
                }
            )*
        }
    };
}
