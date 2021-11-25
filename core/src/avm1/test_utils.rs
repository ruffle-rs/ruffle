use crate::avm1::activation::{Activation, ActivationIdentifier};
use crate::avm1::error::Error;
use crate::avm1::globals::system::SystemProperties;
use crate::avm1::{Avm1, Object, Timers, UpdateContext};
use crate::avm2::Avm2;
use crate::backend::audio::{AudioManager, NullAudioBackend};
use crate::backend::locale::NullLocaleBackend;
use crate::backend::log::NullLogBackend;
use crate::backend::navigator::NullNavigatorBackend;
use crate::backend::render::NullRenderer;
use crate::backend::storage::MemoryStorageBackend;
use crate::backend::ui::NullUiBackend;
use crate::backend::video::NullVideoBackend;
use crate::context::ActionQueue;
use crate::display_object::{MovieClip, Stage, TDisplayObject};
use crate::focus_tracker::FocusTracker;
use crate::library::Library;
use crate::loader::LoadManager;
use crate::prelude::*;
use crate::tag_utils::SwfMovie;
use crate::vminterface::Instantiator;
use gc_arena::{rootless_arena, MutationContext};
use instant::Instant;
use rand::{rngs::SmallRng, SeedableRng};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

pub fn with_avm<F>(swf_version: u8, test: F)
where
    F: for<'a, 'gc> FnOnce(&mut Activation<'_, 'gc, '_>, Object<'gc>) -> Result<(), Error<'gc>>,
{
    fn in_the_arena<'a, 'gc: 'a, F>(swf_version: u8, test: F, gc_context: MutationContext<'gc, '_>)
    where
        F: FnOnce(&mut Activation<'_, 'gc, '_>, Object<'gc>) -> Result<(), Error<'gc>>,
    {
        let mut avm1 = Avm1::new(gc_context, swf_version);
        let mut avm2 = Avm2::new(gc_context);
        let swf = Arc::new(SwfMovie::empty(swf_version));
        let root: DisplayObject<'gc> = MovieClip::new(swf.clone(), gc_context).into();
        root.set_depth(gc_context, 0);
        let stage = Stage::empty(gc_context, 550, 400);
        let mut frame_rate = 12.0;
        let globals = avm1.global_object_cell();

        let mut context = UpdateContext {
            gc_context,
            player_version: 32,
            swf: &swf,
            stage,
            rng: &mut SmallRng::from_seed([0u8; 32]),
            audio: &mut NullAudioBackend::new(),
            ui: &mut NullUiBackend::new(),
            action_queue: &mut ActionQueue::new(),
            library: &mut Library::empty(gc_context),
            navigator: &mut NullNavigatorBackend::new(),
            renderer: &mut NullRenderer::new(),
            locale: &mut NullLocaleBackend::new(),
            log: &mut NullLogBackend::new(),
            video: &mut NullVideoBackend::new(),
            mouse_over_object: None,
            mouse_down_object: None,
            input: &Default::default(),
            mouse_position: &(Twips::ZERO, Twips::ZERO),
            drag_object: &mut None,
            player: None,
            load_manager: &mut LoadManager::new(),
            system: &mut SystemProperties::default(),
            instance_counter: &mut 0,
            storage: &mut MemoryStorageBackend::default(),
            shared_objects: &mut HashMap::new(),
            unbound_text_fields: &mut Vec::new(),
            timers: &mut Timers::new(),
            current_context_menu: &mut None,
            needs_render: &mut false,
            avm1: &mut avm1,
            avm2: &mut avm2,
            external_interface: &mut Default::default(),
            update_start: Instant::now(),
            max_execution_duration: Duration::from_secs(15),
            focus_tracker: FocusTracker::new(gc_context),
            times_get_time_called: 0,
            time_offset: &mut 0,
            audio_manager: &mut AudioManager::new(),
            frame_rate: &mut frame_rate,
        };
        context.stage.replace_at_depth(&mut context, root, 0);

        root.post_instantiation(&mut context, root, None, Instantiator::Movie, false);
        root.set_name(context.gc_context, "".into());

        fn run_test<'a, 'gc: 'a, F>(
            activation: &mut Activation<'_, 'gc, '_>,
            root: DisplayObject<'gc>,
            test: F,
        ) where
            F: FnOnce(&mut Activation<'_, 'gc, '_>, Object<'gc>) -> Result<(), Error<'gc>>,
        {
            let this = root.object().coerce_to_object(activation);
            let result = test(activation, this);
            if let Err(e) = result {
                panic!("Encountered exception during test: {}", e);
            }
        }

        let swf_version = context.swf.version();
        let mut activation = Activation::from_nothing(
            context,
            ActivationIdentifier::root("[Test]"),
            swf_version,
            globals,
            root,
        );

        run_test(&mut activation, root, test)
    }

    rootless_arena(|gc_context| in_the_arena(swf_version, test, gc_context))
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
