//! Group-of-blocks

use crate::decoder::DecoderOption;
use crate::error::{Error, Result};
use crate::parser::reader::H263Reader;
use crate::types::GroupOfBlocks;
use std::io::Read;

/// Attempts to read a GOB record from an H.263 bitstream.
///
/// If no valid start code could be found in the bitstream, this function will
/// raise an error. If it is currently at the start of a picture instead of a
/// GOB, then it will yield `None`, signalling that the current data should
/// be parsed as a picture.
///
/// The set of `DecoderOptions` allows configuring certain information about
/// the decoding process that cannot be determined by decoding the bitstream
/// itself.
///
/// TODO: GOB decoding is a stub (and likely will always be so)
pub fn decode_gob<R>(
    reader: &mut H263Reader<R>,
    _decoder_options: DecoderOption,
) -> Result<Option<GroupOfBlocks>>
where
    R: Read,
{
    reader.with_transaction_union(|reader| {
        let skipped_bits = reader
            .recognize_start_code(false)?
            .ok_or(Error::InvalidGobHeader)?;

        reader.skip_bits(17 + skipped_bits)?;

        let gob_id = reader.read_bits::<u8>(5)?;
        if gob_id == 0 || gob_id == 15 {
            return Ok(None);
        }

        Err(Error::UnimplementedDecoding)
    })
}
