use crate::loader::Error;
use gc_arena::Collect;

/// A stream representing download of some (audiovisual) data.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct NetStream {
    /// All data currently loaded in the stream.
    buffer: Vec<u8>,
}

impl NetStream {
    pub fn load_buffer(&mut self, data: &mut Vec<u8>) {
        self.buffer.append(data);
    }

    pub fn report_error(&mut self, _error: Error) {
        //TODO: Report an `asyncError` to AVM1 or 2.
    }
}
