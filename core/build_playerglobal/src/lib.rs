//! An internal Ruffle utility to build our AVM1 and AVM2 playerglobals

mod avm1;
mod avm2;

pub use avm1::build_avm1_playerglobal;
pub use avm2::build_avm2_playerglobal;
