use gc_arena::Mutation;

use crate::string::AvmStringInterner;

/// Context for managing `AvmString`s: allocating them, interning them, etc...
pub struct StringContext<'gc> {
    /// The mutation context to allocate and mutate `Gc` pointers.
    pub gc_context: &'gc Mutation<'gc>,

    /// The global string interner.
    pub interner: &'gc mut AvmStringInterner<'gc>,
}

impl<'gc> StringContext<'gc> {
    #[inline(always)]
    pub fn gc(&self) -> &'gc Mutation<'gc> {
        self.gc_context
    }
}
