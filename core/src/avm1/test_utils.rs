use crate::avm1::activation::Activation;
use crate::avm1::{ActionContext, Avm1, Object, Value};
use crate::backend::audio::NullAudioBackend;
use crate::backend::navigator::NullNavigatorBackend;
use crate::display_object::DisplayObject;
use crate::movie_clip::MovieClip;
use gc_arena::{rootless_arena, GcCell};
use rand::{rngs::SmallRng, SeedableRng};

pub fn with_avm<F, R>(swf_version: u8, test: F) -> R
where
    F: for<'a, 'gc> FnOnce(
        &mut Avm1<'gc>,
        &mut ActionContext<'a, 'gc, '_>,
        GcCell<'gc, Object<'gc>>,
    ) -> R,
{
    rootless_arena(|gc_context| {
        let mut avm = Avm1::new(gc_context, swf_version);
        let movie_clip: Box<dyn DisplayObject> = Box::new(MovieClip::new(swf_version, gc_context));
        let root = GcCell::allocate(gc_context, movie_clip);
        let mut context = ActionContext {
            gc_context,
            global_time: 0,
            player_version: 32,
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
        avm.insert_stack_frame(
            Activation::from_nothing(swf_version, globals, gc_context),
            &mut context,
        );

        let this = root.read().object().as_object().unwrap().to_owned();

        test(&mut avm, &mut context, this)
    })
}
