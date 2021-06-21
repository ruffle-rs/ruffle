//! Variable-length-code tables

/// A single entry in a VLC table.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Entry<T> {
    /// This entry represents a successful VLC parse.
    ///
    /// The value in `End` will be returned when it is reached in the table.
    End(T),

    /// This entry represents a fork in the table.
    ///
    /// Upon encountering a fork, another bit in the bitstream should be read.
    /// The fork provides a table index for the entry to consider when the bit
    /// is zero (left) or one (right).
    Fork(usize, usize),
}

/// A table whose entries yield `T`.
///
/// This is currently a type alias because otherwise it would either have to...
///
/// 1. Be dynamically sized
/// 2. Hold a `Vec`, meaning that we'd be passing around `&Vec` everywhere.
///
/// Tables can be declared as `Vec<Entry<T>>` for now. For example:
///
/// ```const TEST_TABLE: Vec<Entry<Option<u8>>> = vec![Entry::End(None)];```
pub type Table<T> = [Entry<T>];
