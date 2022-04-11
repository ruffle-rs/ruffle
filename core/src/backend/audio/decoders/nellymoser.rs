use super::{Decoder, SeekableDecoder};
use std::io::{Cursor, Read};

pub struct NellymoserDecoder<R: Read> {
    decoder: nellymoser_rs::Decoder<R>,
}

impl<R: Read> NellymoserDecoder<R> {
    pub fn new(reader: R, sample_rate: u32) -> Self {
        Self {
            decoder: nellymoser_rs::Decoder::new(reader, sample_rate),
        }
    }
}

impl<R: Read> Iterator for NellymoserDecoder<R> {
    type Item = [i16; 2];

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.decoder.next()? as i16;
        Some([sample, sample])
    }
}

impl<R: Read + Send + Sync> Decoder for NellymoserDecoder<R> {
    #[inline]
    fn num_channels(&self) -> u8 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u16 {
        self.decoder.sample_rate() as u16
    }
}

impl<R: AsRef<[u8]> + Send + Sync> SeekableDecoder for NellymoserDecoder<Cursor<R>> {
    #[inline]
    fn reset(&mut self) {
        self.decoder.reset();
    }
}
