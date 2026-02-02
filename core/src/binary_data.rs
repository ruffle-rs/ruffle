use gc_arena::{Collect, Gc};
use ruffle_common::tag_utils::{SwfMovie, SwfSlice};

#[derive(Collect, Debug, Clone, Copy)]
#[collect(no_drop)]
pub struct BinaryData<'gc>(SwfSlice<'gc>);

impl<'gc> BinaryData<'gc> {
    pub fn from_swf_tag(movie: Gc<'gc, SwfMovie>, tag: &swf::DefineBinaryData) -> Self {
        Self(SwfSlice::from(movie).to_subslice(tag.data))
    }

    pub fn to_vec(self) -> Vec<u8> {
        SwfSlice::as_ref(&self.0).to_vec()
    }
}
