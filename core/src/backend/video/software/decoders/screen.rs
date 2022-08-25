// This module is heavily based on flashsv.rs from NihAV,
// written by Kostya Shishkov, with permission.

use crate::backend::video::software::VideoDecoder;
use crate::backend::video::{DecodedFrame, EncodedFrame, Error, FrameDependency};

use flate2::Decompress;

#[derive(thiserror::Error, Debug)]
pub enum ScreenError {
    #[error("Unexpected end of file")]
    UnexpectedEOF,

    #[error("Decompression error")]
    DecompressionError(#[from] flate2::DecompressError),

    #[error("Invalid frame type: {0}")]
    InvalidFrameType(u8),

    #[error("Missing reference frame")]
    MissingReferenceFrame,

    #[error("Not all blocks were updated by a supposed keyframe")]
    KeyframeInvalid,
}

impl From<ScreenError> for Error {
    fn from(error: ScreenError) -> Self {
        Error::DecoderError(Box::new(error))
    }
}

/// Screen Video (V1 only) decoder.
pub struct ScreenVideoDecoder {
    w: usize,
    h: usize,
    block_w: usize,
    block_h: usize,

    tile: Vec<u8>, // acts as a scratch buffer

    last_frame: Option<Vec<u8>>,
}

struct ByteReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> ByteReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_byte(&mut self) -> Result<u8, ScreenError> {
        if self.pos >= self.data.len() {
            return Err(ScreenError::UnexpectedEOF);
        }
        let byte = self.data[self.pos];
        self.pos += 1;
        Ok(byte)
    }

    fn read_u16be(&mut self) -> Result<u16, ScreenError> {
        let byte1 = self.read_byte()?;
        let byte2 = self.read_byte()?;
        Ok((byte1 as u16) << 8 | (byte2 as u16))
    }

    fn read_buf_ref(&mut self, length: usize) -> Result<&[u8], ScreenError> {
        if self.pos + length > self.data.len() {
            return Err(ScreenError::UnexpectedEOF);
        }
        let result = &self.data[self.pos..self.pos + length];
        self.pos += length;
        Ok(result)
    }
}

impl ScreenVideoDecoder {
    pub fn new() -> Self {
        Self {
            w: 0,
            h: 0,
            block_w: 0,
            block_h: 0,
            tile: vec![],
            last_frame: None,
        }
    }

    fn decode_v1(
        &mut self,
        src: &mut ByteReader,
        data: &mut [u8],
        stride: usize,
    ) -> Result<bool, Error> {
        let mut is_intra = true;
        for (yy, row) in data.chunks_mut(stride * self.block_h).enumerate() {
            let cur_h = (self.h - yy * self.block_h).min(self.block_h);
            for x in (0..self.w).step_by(self.block_w) {
                let cur_w = (self.w - x).min(self.block_w);

                let data_size = src.read_u16be()? as usize;
                if data_size > 0 {
                    Decompress::new(true)
                        .decompress(
                            src.read_buf_ref(data_size)?,
                            &mut self.tile[..cur_w * cur_h * 3],
                            flate2::FlushDecompress::Finish,
                        )
                        .map_err(ScreenError::DecompressionError)?;

                    for (dst, src) in row[x * 3..]
                        .chunks_mut(stride)
                        .zip(self.tile.chunks(cur_w * 3))
                    {
                        dst[..cur_w * 3].copy_from_slice(src);
                    }
                } else {
                    is_intra = false;
                }
            }
        }
        Ok(is_intra)
    }

    fn flush(&mut self) {
        self.last_frame = None;
    }
}

impl VideoDecoder for ScreenVideoDecoder {
    fn preload_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<FrameDependency, Error> {
        // There's this extra, undocumented byte between the VideoFrame tag headers and the actual
        // SCREENVIDEOPACKET contents, which is the FrameType + CodecID fields of the VIDEODATA tags
        // in FLV. This is super helpful, because it encodes whether the frame is a keyframe or not.

        // Just a quick sanity check for codec IDs...
        debug_assert!(encoded_frame.data[0] & 0xF == 3);

        match encoded_frame.data[0] >> 4 {
            1 => Ok(FrameDependency::None),
            2 => Ok(FrameDependency::Past),
            x => Err(ScreenError::InvalidFrameType(x).into()),
        }
    }

    fn decode_frame(&mut self, encoded_frame: EncodedFrame<'_>) -> Result<DecodedFrame, Error> {
        let is_keyframe = encoded_frame.data[0] >> 4 == 1;

        if !is_keyframe && self.last_frame.is_none() {
            return Err(ScreenError::MissingReferenceFrame.into());
        }

        // Need to drop the extra preceding byte
        let mut br = ByteReader::new(&encoded_frame.data[1..]);

        let hdr0 = br.read_u16be()? as usize;
        let blk_w = (hdr0 >> 12) * 16 + 16;
        let w = hdr0 & 0xFFF;

        let hdr1 = br.read_u16be()? as usize;
        let blk_h = (hdr1 >> 12) * 16 + 16;
        let h = hdr1 & 0xFFF;

        debug_assert!(w != 0 && h != 0 && blk_w != 0 && blk_h != 0);

        if self.w != w || self.h != h || self.block_w != blk_w || self.block_h != blk_h {
            self.flush();
            self.tile.resize(blk_w * blk_h * 3, 0);
            self.w = w;
            self.h = h;
            self.block_w = blk_w;
            self.block_h = blk_h;
        }

        let mut data = self
            .last_frame
            .clone()
            .unwrap_or_else(|| vec![0; w * h * 3]);

        let stride = w * 3;

        let is_intra = self.decode_v1(&mut br, data.as_mut_slice(), stride)?;

        if is_intra != is_keyframe {
            return Err(ScreenError::KeyframeInvalid.into());
        }

        let mut rgba = vec![0u8; w * h * 4];

        // convert from BGR to RGBA and flip Y
        for y in 0..h {
            let data_row = &data[y * w * 3..(y + 1) * w * 3];
            let rgba_row = &mut rgba[(h - y - 1) * w * 4..(h - y) * w * 4];

            for (bgr, rgba) in data_row.chunks(3).zip(rgba_row.chunks_mut(4)) {
                rgba.copy_from_slice(&[bgr[2], bgr[1], bgr[0], 255]);
            }
        }

        self.last_frame = Some(data);

        Ok(DecodedFrame {
            width: w as u16,
            height: h as u16,
            rgba,
        })
    }
}

impl Default for ScreenVideoDecoder {
    fn default() -> Self {
        Self::new()
    }
}
