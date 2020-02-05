//! ActionScript Virtual Machine 2 (AS3) support

use crate::avm2::value::Value;
use crate::context::UpdateContext;
use crate::tag_utils::SwfSlice;
use gc_arena::Collect;
use swf::avm2::read::Reader;

mod names;
mod object;
mod script_object;
mod value;

/// Boxed error alias.
///
/// As AVM2 is a far stricter VM than AVM1, this may eventually be replaced
/// with a proper Avm2Error enum.
type Error = Box<dyn std::error::Error>;

/// The state of an AVM2 interpreter.
#[derive(Collect)]
#[collect(no_drop)]
pub struct Avm2<'gc> {
    /// Values currently present on the operand stack.
    stack: Vec<Value<'gc>>,
}

impl<'gc> Avm2<'gc> {
    /// Construct a new AVM interpreter.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Load an ABC file embedded in a `SwfSlice`.
    ///
    /// The `SwfSlice` must resolve to the contents of an ABC file.
    ///
    /// The `preload` flag indicates if the file is being encountered as part
    /// of a preloading operation. If false, then this file has actually been
    /// encountered as part of normal movie playback and it's final script
    /// should be executed.
    pub fn load_abc(
        &mut self,
        abc: SwfSlice,
        context: &mut UpdateContext<'_, 'gc, '_>,
        preload: bool,
    ) -> Result<(), Error> {
        let mut read = Reader::new(abc.as_ref());

        let _abc_file = read.read()?;

        Ok(())
    }
}
