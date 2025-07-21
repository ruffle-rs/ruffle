#[cfg(test)]
mod tests;

#[cfg(feature = "assembly")]
pub mod assembly;
pub mod disassembly;
mod parser;

pub use parser::*;
