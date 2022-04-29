use crate::avm1::{
    activation::{Activation, ActivationIdentifier},
    error::Error,
    Object,
};
use crate::display_object::TDisplayObject;

pub fn with_avm<F>(swf_version: u8, test: F)
where
    F: for<'a, 'gc> FnOnce(&mut Activation<'_, 'gc, '_>, Object<'gc>) -> Result<(), Error<'gc>>,
{
    let player = crate::player::PlayerBuilder::new().build();
    let mut player = player.lock().unwrap();
    player.mutate_with_update_context(|context| {
        let context = context.reborrow();
        let globals = context.avm1.globals;
        let root = context.stage.root_clip();
        let mut activation = Activation::from_nothing(
            context,
            ActivationIdentifier::root("[Test]"),
            swf_version,
            globals,
            root,
        );
        let this = root.object().coerce_to_object(&mut activation);
        let result = test(&mut activation, this);
        if let Err(e) = result {
            panic!("Encountered exception during test: {}", e);
        }
    })
}

macro_rules! test_method {
    ( $test: ident, $name: expr, $object: expr, $($versions: expr => { $([$($arg: expr),*] => $out: expr),* }),* ) => {
        #[test]
        fn $test() {
            use $crate::avm1::test_utils::*;
            $(
                for version in &$versions {
                    with_avm(*version, |activation, _root| -> Result<(), Error> {
                        let name: $crate::avm1::AvmString<'_> = $name.into();
                        let object = $object(activation);

                        $(
                            let args: Vec<Value> = vec![$($arg.into()),*];
                            let ret = crate::avm1::object::TObject::call_method(&object, name, &args, activation)?;
                            assert_eq!(ret, $out.into(), "{:?} => {:?} in swf {}", args, $out, version);
                        )*

                        Ok(())
                    });
                }
            )*
        }
    };
}
