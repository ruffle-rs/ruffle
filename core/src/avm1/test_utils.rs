use crate::avm1::activation::Activation;
use crate::avm1::{Avm1, Object, UpdateContext, Value};
use crate::backend::audio::NullAudioBackend;
use crate::backend::navigator::NullNavigatorBackend;
use crate::backend::render::NullRenderer;
use crate::context::ActionQueue;
use crate::display_object::{DisplayObject, MovieClip};
use crate::library::Library;
use crate::prelude::*;
use gc_arena::{rootless_arena, GcCell};
use rand::{rngs::SmallRng, SeedableRng};
use std::sync::Arc;

pub fn with_avm<F, R>(swf_version: u8, test: F) -> R
where
    F: for<'a, 'gc> FnOnce(
        &mut Avm1<'gc>,
        &mut UpdateContext<'a, 'gc, '_>,
        GcCell<'gc, Object<'gc>>,
    ) -> R,
{
    rootless_arena(|gc_context| {
        let mut avm = Avm1::new(gc_context, swf_version);
        let movie_clip: Box<dyn DisplayObject> = Box::new(MovieClip::new(swf_version, gc_context));
        let root = GcCell::allocate(gc_context, movie_clip);
        let mut context = UpdateContext {
            gc_context,
            global_time: 0,
            player_version: 32,
            swf_version,
            root,
            start_clip: root,
            active_clip: root,
            target_clip: Some(root),
            target_path: Value::Undefined,
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
        };

        let globals = avm.global_object_cell();
        avm.insert_stack_frame(GcCell::allocate(
            gc_context,
            Activation::from_nothing(swf_version, globals, gc_context),
        ));

        let this = root.read().object().as_object().unwrap().to_owned();

        test(&mut avm, &mut context, this)
    })
}
