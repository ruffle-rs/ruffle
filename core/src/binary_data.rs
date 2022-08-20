use crate::tag_utils::{SwfMovie, SwfSlice};
use std::sync::Arc;

pub type BinaryData = SwfSlice;

impl BinaryData {
    pub fn from_swf_tag(movie: Arc<SwfMovie>, tag: &swf::DefineBinaryData) -> Self {
        SwfSlice::from(movie).to_subslice(tag.data)
    }
}
