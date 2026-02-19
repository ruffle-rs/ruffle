use crate::avm1::{
    Object,
    activation::{Activation, ActivationIdentifier},
    error::Error,
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
