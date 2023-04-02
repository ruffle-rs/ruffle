//! NetStream implementation

use crate::backend::navigator::Request;
use crate::context::UpdateContext;
use crate::loader::Error;
use crate::string::AvmString;
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
    playing_streams: Vec<NetStream<'gc>>,
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

    pub fn ensure_playing(context: &mut UpdateContext<'_, 'gc>, stream: NetStream<'gc>) {
        if !context.stream_manager.playing_streams.contains(&stream) {
            context.stream_manager.playing_streams.push(stream);
        }
    }

    pub fn ensure_paused(context: &mut UpdateContext<'_, 'gc>, stream: NetStream<'gc>) {
        let index = context
            .stream_manager
            .playing_streams
            .iter()
            .position(|x| *x == stream);
        if let Some(index) = index {
            context.stream_manager.playing_streams.remove(index);
        }
    }

    pub fn toggle_paused(context: &mut UpdateContext<'_, 'gc>, stream: NetStream<'gc>) {
        let index = context
            .stream_manager
            .playing_streams
            .iter()
            .position(|x| *x == stream);
        if let Some(index) = index {
            context.stream_manager.playing_streams.remove(index);
        } else {
            context.stream_manager.playing_streams.push(stream);
        }
    }

    /// Process all playing media streams.
    ///
    /// This is an unlocked timestep; the `dt` parameter indicates how many
    /// milliseconds have elapsed since the last tick. This is intended to
    /// support video framerates separate from the Stage frame rate.
    ///
    /// This does not borrow `&mut self` as we need the `UpdateContext`, too.
    pub fn tick(_context: &mut UpdateContext<'_, 'gc>, _dt: f64) {}
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
#[derive(Clone, Debug, Collect, Copy)]
#[collect(no_drop)]
pub struct NetStream<'gc>(GcCell<'gc, NetStreamData>);

impl<'gc> PartialEq for NetStream<'gc> {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl<'gc> Eq for NetStream<'gc> {}

#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct NetStreamData {
    /// All data currently loaded in the stream.
    buffer: Vec<u8>,
}

impl<'gc> NetStream<'gc> {
    pub fn new(gc_context: MutationContext<'gc, '_>) -> Self {
        Self(GcCell::allocate(
            gc_context,
            NetStreamData { buffer: Vec::new() },
        ))
    }

    pub fn load_buffer(self, gc_context: MutationContext<'gc, '_>, data: &mut Vec<u8>) {
        self.0.write(gc_context).buffer.append(data);
    }

    pub fn report_error(self, _error: Error) {
        //TODO: Report an `asyncError` to AVM1 or 2.
    }

    pub fn bytes_loaded(self) -> usize {
        self.0.read().buffer.len()
    }

    pub fn bytes_total(self) -> usize {
        self.0.read().buffer.len()
    }

    /// Start playing media from this NetStream.
    ///
    /// If `name` is specified, this will also trigger streaming download of
    /// the given resource. Otherwise, the stream will play whatever data is
    /// available in the buffer.
    pub fn play(self, context: &mut UpdateContext<'_, 'gc>, name: Option<AvmString<'gc>>) {
        if let Some(name) = name {
            let request = Request::get(name.to_string());
            context
                .load_manager
                .load_netstream(context.player.clone(), self, request);
        }

        StreamManager::ensure_playing(context, self);
    }

    /// Pause stream playback.
    pub fn pause(self, context: &mut UpdateContext<'_, 'gc>) {
        StreamManager::ensure_paused(context, self);
    }

    /// Resume stream playback.
    pub fn resume(self, context: &mut UpdateContext<'_, 'gc>) {
        StreamManager::ensure_playing(context, self);
    }

    /// Resume stream playback if paused, pause otherwise.
    pub fn toggle_paused(self, context: &mut UpdateContext<'_, 'gc>) {
        StreamManager::toggle_paused(context, self);
    }
}
