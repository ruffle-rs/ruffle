//! Decoded picture type

use crate::types::{Picture, SourceFormat};

/// A decoded picture.
pub struct DecodedPicture {
    /// The header of the picture that was decoded.
    picture_header: Picture,

    /// The source format in force for this picture.
    format: SourceFormat,

    /// The luma data of the decoded picture.
    luma: Vec<u8>,

    /// The u-component chroma data of the decoded picture.
    chroma_b: Vec<u8>,

    /// The v-component chroma data of the decoded picture.
    chroma_r: Vec<u8>,

    /// The number of chroma samples per row of data.
    ///
    /// May be rounded up to the nearest pixel.
    chroma_samples_per_row: usize,
}

impl DecodedPicture {
    /// Construct a new `DecodedPicture` for a given picture with a particular
    /// format.
    ///
    /// The decoded picture will be created with luma and chroma buffers large
    /// enough to hold the dimensions indicated by `format`. These can be any
    /// size; and should not be rounded up to the next macroblock.
    ///
    /// Invalid source formats will fail to generate a decoded picture.
    pub fn new(picture_header: Picture, format: SourceFormat) -> Option<Self> {
        let (w, h) = format.into_width_and_height()?;
        let luma_samples = w as usize * h as usize;
        let mut luma = Vec::new();
        luma.resize(luma_samples, 0);

        let chroma_w = (w as f32 / 2.0).ceil() as usize;
        let chroma_h = (h as f32 / 2.0).ceil() as usize;
        let chroma_samples = chroma_w * chroma_h;
        let mut chroma_b = Vec::new();
        chroma_b.resize(chroma_samples, 0);
        let mut chroma_r = Vec::new();
        chroma_r.resize(chroma_samples, 0);

        Some(Self {
            picture_header,
            format,
            luma,
            chroma_b,
            chroma_r,
            chroma_samples_per_row: chroma_w,
        })
    }

    /// Get the header this picture was decoded with.
    pub fn as_header(&self) -> &Picture {
        &self.picture_header
    }

    /// Get the source format.
    pub fn format(&self) -> SourceFormat {
        self.format
    }

    /// Get the luma data for this picture.
    ///
    /// Raw luma data is stored in row-major (x + y*samples_per_row) order with
    /// 8 bits per pixel. The width of the array is given with
    /// `luma_samples_per_row` and you index the array with the above formula.
    pub fn as_luma(&self) -> &[u8] {
        &self.luma
    }

    /// Mutably borrow the luma data for this picture.
    ///
    /// Raw luma data is stored in row-major (x + y*samples_per_row) order with
    /// 8 bits per pixel. The width of the array is given with
    /// `luma_samples_per_row` and you index the array with the above formula.
    pub fn as_luma_mut(&mut self) -> &mut [u8] {
        &mut self.luma
    }

    /// Get how many luma samples exist per row.
    pub fn luma_samples_per_row(&self) -> usize {
        let (w, _h) = self.format().into_width_and_height().unwrap();
        w as usize
    }

    /// Get how many chroma samples exist per row.
    pub fn chroma_samples_per_row(&self) -> usize {
        self.chroma_samples_per_row
    }

    /// Get the chroma-B data for this picture.
    ///
    /// Raw chroma data is stored in row-major (x + y*samples_per_row) order
    /// with 8 bits per pixel. The width of the array is given with
    /// `chroma_samples_per_row` and you index the array with the above
    /// formula. Each chroma pixel corresponds to four luma pixels.
    pub fn as_chroma_b(&self) -> &[u8] {
        &self.chroma_b
    }

    /// Mutably borrow the chroma-B data for this picture.
    ///
    /// Raw chroma data is stored in row-major (x + y*samples_per_row) order
    /// with 8 bits per pixel. The width of the array is given with
    /// `chroma_samples_per_row` and you index the array with the above
    /// formula. Each chroma pixel corresponds to four luma pixels.
    pub fn as_chroma_b_mut(&mut self) -> &mut [u8] {
        &mut self.chroma_b
    }

    /// Get the chroma-R data for this picture.
    ///
    /// Raw chroma data is stored in row-major (x + y*samples_per_row) order
    /// with 8 bits per pixel. The width of the array is given with
    /// `chroma_samples_per_row` and you index the array with the above
    /// formula. Each chroma pixel corresponds to four luma pixels.
    pub fn as_chroma_r(&self) -> &[u8] {
        &self.chroma_r
    }

    /// Mutably borrow the chroma-R data for this picture.
    ///
    /// Raw chroma data is stored in row-major (x + y*samples_per_row) order
    /// with 8 bits per pixel. The width of the array is given with
    /// `chroma_samples_per_row` and you index the array with the above
    /// formula. Each chroma pixel corresponds to four luma pixels.
    pub fn as_chroma_r_mut(&mut self) -> &mut [u8] {
        &mut self.chroma_r
    }

    /// Borrow the YUV data in this picture.
    pub fn as_yuv(&self) -> (&[u8], &[u8], &[u8]) {
        (&self.luma, &self.chroma_b, &self.chroma_r)
    }
}
