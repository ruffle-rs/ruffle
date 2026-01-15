use gc_arena::Collect;
use ruffle_common::tag_utils::{SwfMovie, SwfSlice};
use std::sync::Arc;

#[derive(Collect, Debug)]
#[collect(require_static)]
pub struct BinaryData(SwfSlice);

impl BinaryData {
    pub fn from_swf_tag(movie: Arc<SwfMovie>, tag: &swf::DefineBinaryData) -> Self {
        Self(SwfSlice::from(movie).to_subslice(tag.data))
    }

    pub fn to_vec(&self) -> Vec<u8> {
        SwfSlice::as_ref(&self.0).to_vec()
    }
}
