use crate::avm1::activation::Activation;
use crate::avm1::{Avm1, Object, UpdateContext};
use crate::backend::audio::NullAudioBackend;
use crate::backend::navigator::NullNavigatorBackend;
use crate::backend::render::NullRenderer;
use crate::context::ActionQueue;
use crate::display_object::{MovieClip, TDisplayObject};
use crate::library::Library;
use crate::prelude::*;
use gc_arena::{rootless_arena, GcCell, MutationContext};
use rand::{rngs::SmallRng, SeedableRng};
use std::sync::Arc;

pub fn with_avm<F, R>(swf_version: u8, test: F) -> R
where
    F: for<'a, 'gc> FnOnce(&mut Avm1<'gc>, &mut UpdateContext<'a, 'gc, '_>, Object<'gc>) -> R,
{
    fn in_the_arena<'gc, F, R>(swf_version: u8, test: F, gc_context: MutationContext<'gc, '_>) -> R
    where
        F: for<'a> FnOnce(&mut Avm1<'gc>, &mut UpdateContext<'a, 'gc, '_>, Object<'gc>) -> R,
    {
        let mut avm = Avm1::new(gc_context, swf_version);
        let mut root: DisplayObject<'_> = MovieClip::new(swf_version, gc_context).into();
        root.post_instantiation(gc_context, root, avm.prototypes().movie_clip);

        let mut context = UpdateContext {
            gc_context,
            global_time: 0,
            player_version: 32,
            swf_version,
            root,
            rng: &mut SmallRng::from_seed([0u8; 16]),
            audio: &mut NullAudioBackend::new(),
            action_queue: &mut ActionQueue::new(),
            background_color: &mut Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            library: &mut Library::new(),
            navigator: &mut NullNavigatorBackend::new(),
            renderer: &mut NullRenderer::new(),
            swf_data: &mut Arc::new(vec![]),
            system_prototypes: avm.prototypes().clone(),
            mouse_hovered_object: None,
            mouse_position: &(Twips::new(0), Twips::new(0)),
            drag_object: &mut None,
            stage_size: (Twips::from_pixels(550.0), Twips::from_pixels(400.0)),
        };

        let globals = avm.global_object_cell();
        avm.insert_stack_frame(GcCell::allocate(
            gc_context,
            Activation::from_nothing(swf_version, globals, gc_context, root),
        ));

        let this = root.object().as_object().unwrap();

        test(&mut avm, &mut context, this)
    }

    rootless_arena(|gc_context| in_the_arena(swf_version, test, gc_context))
}

macro_rules! test_method {
    ( $test: ident, $name: expr, $object: expr, $($versions: expr => { $([$($arg: expr),*] => $out: expr),* }),* ) => {
        #[test]
        fn $test() -> Result<(), Error> {
            use $crate::avm1::test_utils::*;
            $(
                for version in &$versions {
                    let _ = with_avm(*version, |avm, context, _root| -> Result<(), Error> {
                        let object = $object(avm, context);
                        let function = object.get($name, avm, context)?.unwrap_immediate();

                        $(
                            #[allow(unused_mut)]
                            let mut args: Vec<Value> = Vec::new();
                            $(
                                args.push($arg.into());
                            )*
                            assert_eq!(function.call(avm, context, object, &args)?, ReturnValue::Immediate($out.into()), "{:?} => {:?} in swf {}", args, $out, version);
                        )*

                        Ok(())
                    })?;
                }
            )*

            Ok(())
        }
    };
}
