use crate::debug::movie_clip_debugger::MovieClipInfo;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum DisplayObjectInfo {
    MovieClip(MovieClipInfo),
}
