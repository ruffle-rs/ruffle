//! NetStream implementation

use crate::context::UpdateContext;
use crate::loader::Error;
use gc_arena::{Collect, GcCell, MutationContext};

/// Manager for all media streams.
///
/// This does *not* handle data transport; which is delegated to `LoadManager`.
/// `StreamManager` *only* handles decoding or encoding of relevant media
/// streams.
#[derive(Collect)]
#[collect(no_drop)]
pub struct StreamManager<'gc> {
    /// List of actively playing streams.
    ///
    /// This is not the total list of all created NetStreams; only the ones
    /// that have been configured to play media.
    playing_streams: Vec<GcCell<'gc, NetStream>>,
}

impl<'gc> Default for StreamManager<'gc> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'gc> StreamManager<'gc> {
    pub fn new() -> Self {
        StreamManager {
            playing_streams: Vec::new(),
        }
    }

    /// Process all playing media streams.
    ///
    /// This is an unlocked timestep; the `dt` parameter indicates how many
    /// milliseconds have elapsed since the last tick. This is intended to
    /// support video framerates separate from the Stage frame rate.
    ///
    /// This does not borrow `&mut self` as we need the `UpdateContext`, too.
    pub fn tick(_context: &mut UpdateContext<'gc, '_>, _dt: f64) {}
}

/// A stream representing download of some (audiovisual) data.
///
/// `NetStream` interacts with several different parts of player
/// infrastructure:
///
///  * `LoadManager` fills individual `NetStream` buffers with data (or, in the
///    future, empties them out for media upload)
///  * `StreamManager` processes media data in the `NetStream` buffer (in the
///    future, sending it to the audio backend or `SoundManager`)
///  * `Video` display objects linked to this `NetStream` display the latest
///    decoded frame.
///
/// It corresponds directly to the AVM1 and AVM2 `NetStream` classes; it's API
/// is intended to be a VM-agnostic version of those.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct NetStream {
    /// All data currently loaded in the stream.
    buffer: Vec<u8>,
}

impl NetStream {
    pub fn new<'gc>(gc_context: MutationContext<'gc, '_>) -> GcCell<'gc, Self> {
        GcCell::allocate(gc_context, NetStream { buffer: Vec::new() })
    }

    pub fn load_buffer(&mut self, data: &mut Vec<u8>) {
        self.buffer.append(data);
    }

    pub fn report_error(&mut self, _error: Error) {
        //TODO: Report an `asyncError` to AVM1 or 2.
    }
}
