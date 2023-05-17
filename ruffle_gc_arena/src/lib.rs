mod gc_cell;
mod gc_weak_cell;

pub use gc_arena::*;
pub use gc_cell::GcCell;
pub use gc_weak_cell::GcWeakCell;

// TODO: remove usage of these typedefs
pub type MutationContext<'gc, 'a> = &'a Mutation<'gc>;
pub type CollectionContext<'a> = &'a Collection;
