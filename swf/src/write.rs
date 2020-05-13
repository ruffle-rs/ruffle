#![allow(
    clippy::cognitive_complexity,
    clippy::float_cmp,
    clippy::inconsistent_digit_grouping,
    clippy::unreadable_literal
)]

use crate::error::{Error, Result};
use crate::tag_code::TagCode;
use crate::types::*;
use byteorder::{LittleEndian, WriteBytesExt};
use enumset::EnumSet;
use std::cmp::max;
use std::io::{self, Write};

/// Writes an SWF file to an output stream.
/// # Example
/// ```
/// use swf::*;
///
/// let swf = Swf {
///     header: Header {
///         version: 6,
///         compression: Compression::Zlib,
///         stage_size: Rectangle { x_min: Twips::from_pixels(0.0), x_max: Twips::from_pixels(400.0), y_min: Twips::from_pixels(0.0), y_max: Twips::from_pixels(400.0) },
///         frame_rate: 60.0,
///         num_frames: 1,
///     },
///     tags: vec![
///         Tag::SetBackgroundColor(Color { r: 255, g: 0, b: 0, a: 255 }),
///         Tag::ShowFrame
///     ]
/// };
/// let output = Vec::new();
/// swf::write_swf(&swf, output).unwrap();
/// ```
pub fn write_swf<W: Write>(swf: &Swf, mut output: W) -> Result<()> {
    let signature = match swf.header.compression {
        Compression::None => b"FWS",
        Compression::Zlib => b"CWS",
        Compression::Lzma => b"ZWS",
    };
    output.write_all(&signature[..])?;
    output.write_u8(swf.header.version)?;

    // Write SWF body.
    let mut swf_body = Vec::new();
    {
        let mut writer = Writer::new(&mut swf_body, swf.header.version);

        writer.write_rectangle(&swf.header.stage_size)?;
        writer.write_fixed8(swf.header.frame_rate)?;
        writer.write_u16(swf.header.num_frames)?;

        // Write main timeline tag list.
        writer.write_tag_list(&swf.tags)?;
    }

    // Write SWF header.
    // Uncompressed SWF length.
    output.write_u32::<LittleEndian>(swf_body.len() as u32 + 8)?;

    // Compress SWF body.
    match swf.header.compression {
        Compression::None => {
            output.write_all(&swf_body)?;
        }

        Compression::Zlib => write_zlib_swf(&mut output, &swf_body)?,

        // LZMA header.
        // SWF format has a mangled LZMA header, so we have to do some magic to conver the
        // standard LZMA header to SWF format.
        // https://adobe.ly/2s8oYzn
        Compression::Lzma => write_lzma_swf(&mut output, &swf_body)?,
    };

    Ok(())
}

#[cfg(feature = "flate2")]
fn write_zlib_swf<W: Write>(mut output: W, swf_body: &[u8]) -> Result<()> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    let mut encoder = ZlibEncoder::new(&mut output, Compression::best());
    encoder.write_all(&swf_body)?;
    encoder.finish()?;
    Ok(())
}

#[cfg(all(feature = "libflate", not(feature = "flate2")))]
fn write_zlib_swf<W: Write>(mut output: W, swf_body: &[u8]) -> Result<()> {
    use libflate::zlib::Encoder;
    let mut encoder = Encoder::new(&mut output)?;
    encoder.write_all(&swf_body)?;
    encoder.finish().into_result()?;
    Ok(())
}

#[cfg(not(any(feature = "flate2", feature = "libflate")))]
fn write_zlib_swf<W: Write>(_output: W, _swf_body: &[u8]) -> Result<()> {
    Err(Error::unsupported(
        "Support for Zlib compressed SWFs is not enabled.",
    ))
}

#[cfg(feature = "lzma")]
fn write_lzma_swf<W: Write>(mut output: W, swf_body: &[u8]) -> Result<()> {
    use xz2::{
        stream::{Action, LzmaOptions, Stream},
        write::XzEncoder,
    };
    let mut stream = Stream::new_lzma_encoder(&LzmaOptions::new_preset(9).unwrap()).unwrap();
    let mut lzma_header = [0; 13];
    stream.process(&[], &mut lzma_header, Action::Run).unwrap();
    // Compressed length. We just write out a dummy value.
    output.write_u32::<LittleEndian>(0xffffffff)?;
    output.write_all(&lzma_header[0..5])?; // LZMA property bytes.
    let mut encoder = XzEncoder::new_stream(&mut output, stream);
    encoder.write_all(&swf_body)?;
    Ok(())
}

#[cfg(not(feature = "lzma"))]
fn write_lzma_swf<W: Write>(_output: W, _swf_body: &[u8]) -> Result<()> {
    Err(Error::unsupported(
        "Support for LZMA compressed SWFs is not enabled.",
    ))
}

pub trait SwfWrite<W: Write> {
    fn get_inner(&mut self) -> &mut W;

    fn write_u8(&mut self, n: u8) -> io::Result<()> {
        self.get_inner().write_u8(n)
    }

    fn write_u16(&mut self, n: u16) -> io::Result<()> {
        self.get_inner().write_u16::<LittleEndian>(n)
    }

    fn write_u32(&mut self, n: u32) -> io::Result<()> {
        self.get_inner().write_u32::<LittleEndian>(n)
    }

    fn write_u64(&mut self, n: u64) -> io::Result<()> {
        self.get_inner().write_u64::<LittleEndian>(n)
    }

    fn write_i8(&mut self, n: i8) -> io::Result<()> {
        self.get_inner().write_i8(n)
    }

    fn write_i16(&mut self, n: i16) -> io::Result<()> {
        self.get_inner().write_i16::<LittleEndian>(n)
    }

    fn write_i32(&mut self, n: i32) -> io::Result<()> {
        self.get_inner().write_i32::<LittleEndian>(n)
    }

    fn write_fixed8(&mut self, n: f32) -> io::Result<()> {
        self.write_i16((n * 256f32) as i16)
    }

    fn write_fixed16(&mut self, n: f64) -> io::Result<()> {
        self.write_i32((n * 65536f64) as i32)
    }

    fn write_f32(&mut self, n: f32) -> io::Result<()> {
        self.get_inner().write_f32::<LittleEndian>(n)
    }

    fn write_f64(&mut self, n: f64) -> io::Result<()> {
        // Flash weirdly stores f64 as two LE 32-bit chunks.
        // First word is the hi-word, second word is the lo-word.
        let mut num = [0u8; 8];
        (&mut num[..]).write_f64::<LittleEndian>(n)?;
        num.swap(0, 4);
        num.swap(1, 5);
        num.swap(2, 6);
        num.swap(3, 7);
        self.get_inner().write_all(&num)
    }

    fn write_c_string(&mut self, s: &str) -> io::Result<()> {
        self.get_inner().write_all(s.as_bytes())?;
        self.write_u8(0)
    }
}

struct Writer<W: Write> {
    pub output: W,
    pub version: u8,
    pub byte: u8,
    pub bit_index: u8,
    pub num_fill_bits: u8,
    pub num_line_bits: u8,
}

impl<W: Write> SwfWrite<W> for Writer<W> {
    fn get_inner(&mut self) -> &mut W {
        &mut self.output
    }

    fn write_u8(&mut self, n: u8) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_u8(n)
    }

    fn write_u16(&mut self, n: u16) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_u16::<LittleEndian>(n)
    }

    fn write_u32(&mut self, n: u32) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_u32::<LittleEndian>(n)
    }

    fn write_i8(&mut self, n: i8) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_i8(n)
    }

    fn write_i16(&mut self, n: i16) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_i16::<LittleEndian>(n)
    }

    fn write_i32(&mut self, n: i32) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_i32::<LittleEndian>(n)
    }

    fn write_f32(&mut self, n: f32) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_f32::<LittleEndian>(n)
    }

    fn write_f64(&mut self, n: f64) -> io::Result<()> {
        self.flush_bits()?;
        self.output.write_f64::<LittleEndian>(n)
    }

    fn write_c_string(&mut self, s: &str) -> io::Result<()> {
        self.flush_bits()?;
        self.get_inner().write_all(s.as_bytes())?;
        self.write_u8(0)
    }
}

impl<W: Write> Writer<W> {
    fn new(output: W, version: u8) -> Writer<W> {
        Writer {
            output,
            version,
            byte: 0,
            bit_index: 8,
            num_fill_bits: 0,
            num_line_bits: 0,
        }
    }

    #[allow(dead_code)]
    fn into_inner(self) -> W {
        self.output
    }

    fn write_bit(&mut self, set: bool) -> Result<()> {
        self.bit_index -= 1;
        if set {
            self.byte |= 1 << self.bit_index;
        }
        if self.bit_index == 0 {
            self.flush_bits()?;
        }
        Ok(())
    }

    fn flush_bits(&mut self) -> io::Result<()> {
        if self.bit_index != 8 {
            self.output.write_u8(self.byte)?;
            self.bit_index = 8;
            self.byte = 0;
        }
        Ok(())
    }

    fn write_ubits(&mut self, num_bits: u8, n: u32) -> Result<()> {
        for i in 0..num_bits {
            self.write_bit(n & (1 << u32::from(num_bits - i - 1)) != 0)?;
        }
        Ok(())
    }

    fn write_sbits(&mut self, num_bits: u8, n: i32) -> Result<()> {
        self.write_ubits(num_bits, n as u32)
    }

    fn write_sbits_twips(&mut self, num_bits: u8, twips: Twips) -> Result<()> {
        self.write_sbits(num_bits, twips.get())
    }

    fn write_fbits(&mut self, num_bits: u8, n: f32) -> Result<()> {
        self.write_sbits(num_bits, (n * 65536f32) as i32)
    }

    fn write_encoded_u32(&mut self, mut n: u32) -> Result<()> {
        loop {
            let mut byte = (n & 0b01111111) as u8;
            n >>= 7;
            if n != 0 {
                byte |= 0b10000000;
            }
            self.write_u8(byte)?;
            if n == 0 {
                break;
            }
        }
        Ok(())
    }

    fn write_rectangle(&mut self, rectangle: &Rectangle) -> Result<()> {
        self.flush_bits()?;
        let num_bits: u8 = [
            rectangle.x_min,
            rectangle.x_max,
            rectangle.y_min,
            rectangle.y_max,
        ]
        .iter()
        .map(|x| count_sbits_twips(*x))
        .max()
        .unwrap();
        self.write_ubits(5, num_bits.into())?;
        self.write_sbits_twips(num_bits, rectangle.x_min)?;
        self.write_sbits_twips(num_bits, rectangle.x_max)?;
        self.write_sbits_twips(num_bits, rectangle.y_min)?;
        self.write_sbits_twips(num_bits, rectangle.y_max)?;
        Ok(())
    }

    fn write_character_id(&mut self, id: CharacterId) -> Result<()> {
        self.write_u16(id)?;
        Ok(())
    }

    fn write_rgb(&mut self, color: &Color) -> Result<()> {
        self.write_u8(color.r)?;
        self.write_u8(color.g)?;
        self.write_u8(color.b)?;
        Ok(())
    }

    fn write_rgba(&mut self, color: &Color) -> Result<()> {
        self.write_u8(color.r)?;
        self.write_u8(color.g)?;
        self.write_u8(color.b)?;
        self.write_u8(color.a)?;
        Ok(())
    }

    fn write_color_transform_no_alpha(&mut self, color_transform: &ColorTransform) -> Result<()> {
        // TODO: Assert that alpha is 1.0?
        self.flush_bits()?;
        let has_mult = color_transform.r_multiply != 1f32
            || color_transform.g_multiply != 1f32
            || color_transform.b_multiply != 1f32;
        let has_add =
            color_transform.r_add != 0 || color_transform.g_add != 0 || color_transform.b_add != 0;
        let multiply = [
            color_transform.r_multiply,
            color_transform.g_multiply,
            color_transform.b_multiply,
        ];
        let add = [
            color_transform.a_add,
            color_transform.g_add,
            color_transform.b_add,
            color_transform.a_add,
        ];
        self.write_bit(has_mult)?;
        self.write_bit(has_add)?;
        let mut num_bits = if has_mult {
            multiply
                .iter()
                .map(|n| count_sbits((*n * 256f32) as i32))
                .max()
                .unwrap()
        } else {
            0u8
        };
        if has_add {
            num_bits = max(
                num_bits,
                add.iter()
                    .map(|n| count_sbits(i32::from(*n)))
                    .max()
                    .unwrap(),
            );
        }
        self.write_ubits(4, num_bits.into())?;
        if has_mult {
            self.write_sbits(num_bits, (color_transform.r_multiply * 256f32) as i32)?;
            self.write_sbits(num_bits, (color_transform.g_multiply * 256f32) as i32)?;
            self.write_sbits(num_bits, (color_transform.b_multiply * 256f32) as i32)?;
        }
        if has_add {
            self.write_sbits(num_bits, color_transform.r_add.into())?;
            self.write_sbits(num_bits, color_transform.g_add.into())?;
            self.write_sbits(num_bits, color_transform.b_add.into())?;
        }
        Ok(())
    }

    fn write_color_transform(&mut self, color_transform: &ColorTransform) -> Result<()> {
        self.flush_bits()?;
        let has_mult = color_transform.r_multiply != 1f32
            || color_transform.g_multiply != 1f32
            || color_transform.b_multiply != 1f32
            || color_transform.a_multiply != 1f32;
        let has_add = color_transform.r_add != 0
            || color_transform.g_add != 0
            || color_transform.b_add != 0
            || color_transform.a_add != 0;
        let multiply = [
            color_transform.r_multiply,
            color_transform.g_multiply,
            color_transform.b_multiply,
            color_transform.a_multiply,
        ];
        let add = [
            color_transform.r_add,
            color_transform.g_add,
            color_transform.b_add,
            color_transform.a_add,
        ];
        self.write_bit(has_add)?;
        self.write_bit(has_mult)?;
        let mut num_bits = if has_mult {
            multiply
                .iter()
                .map(|n| count_sbits((*n * 256f32) as i32))
                .max()
                .unwrap()
        } else {
            0u8
        };
        if has_add {
            num_bits = max(
                num_bits,
                add.iter()
                    .map(|n| count_sbits(i32::from(*n)))
                    .max()
                    .unwrap(),
            );
        }
        self.write_ubits(4, num_bits.into())?;
        if has_mult {
            self.write_sbits(num_bits, (color_transform.r_multiply * 256f32) as i32)?;
            self.write_sbits(num_bits, (color_transform.g_multiply * 256f32) as i32)?;
            self.write_sbits(num_bits, (color_transform.b_multiply * 256f32) as i32)?;
            self.write_sbits(num_bits, (color_transform.a_multiply * 256f32) as i32)?;
        }
        if has_add {
            self.write_sbits(num_bits, color_transform.r_add.into())?;
            self.write_sbits(num_bits, color_transform.g_add.into())?;
            self.write_sbits(num_bits, color_transform.b_add.into())?;
            self.write_sbits(num_bits, color_transform.a_add.into())?;
        }
        Ok(())
    }

    fn write_matrix(&mut self, m: &Matrix) -> Result<()> {
        self.flush_bits()?;
        // Scale
        let has_scale = m.scale_x != 1f32 || m.scale_y != 1f32;
        self.write_bit(has_scale)?;
        if has_scale {
            let num_bits = max(count_fbits(m.scale_x), count_fbits(m.scale_y));
            self.write_ubits(5, num_bits.into())?;
            self.write_fbits(num_bits, m.scale_x)?;
            self.write_fbits(num_bits, m.scale_y)?;
        }
        // Rotate/Skew
        let has_rotate_skew = m.rotate_skew_0 != 0f32 || m.rotate_skew_1 != 0f32;
        self.write_bit(has_rotate_skew)?;
        if has_rotate_skew {
            let num_bits = max(count_fbits(m.rotate_skew_0), count_fbits(m.rotate_skew_1));
            self.write_ubits(5, num_bits.into())?;
            self.write_fbits(num_bits, m.rotate_skew_0)?;
            self.write_fbits(num_bits, m.rotate_skew_1)?;
        }
        // Translate (always written)
        let num_bits = max(
            count_sbits_twips(m.translate_x),
            count_sbits_twips(m.translate_y),
        );
        self.write_ubits(5, num_bits.into())?;
        self.write_sbits_twips(num_bits, m.translate_x)?;
        self.write_sbits_twips(num_bits, m.translate_y)?;
        Ok(())
    }

    fn write_language(&mut self, language: Language) -> Result<()> {
        self.write_u8(match language {
            Language::Unknown => 0,
            Language::Latin => 1,
            Language::Japanese => 2,
            Language::Korean => 3,
            Language::SimplifiedChinese => 4,
            Language::TraditionalChinese => 5,
        })?;
        Ok(())
    }

    fn write_tag(&mut self, tag: &Tag) -> Result<()> {
        match *tag {
            Tag::ShowFrame => self.write_tag_header(TagCode::ShowFrame, 0)?,

            Tag::ExportAssets(ref exports) => self.write_export_assets(&exports[..])?,

            Tag::Protect(ref password) => {
                if let Some(ref password_md5) = *password {
                    self.write_tag_header(TagCode::Protect, password_md5.len() as u32 + 3)?;
                    self.write_u16(0)?; // Two null bytes? Not specified in SWF19.
                    self.write_c_string(password_md5)?;
                } else {
                    self.write_tag_header(TagCode::Protect, 0)?;
                }
            }

            Tag::CsmTextSettings(ref settings) => {
                self.write_tag_header(TagCode::CsmTextSettings, 12)?;
                self.write_character_id(settings.id)?;
                self.write_u8(
                    if settings.use_advanced_rendering {
                        0b01_000000
                    } else {
                        0
                    } | match settings.grid_fit {
                        TextGridFit::None => 0,
                        TextGridFit::Pixel => 0b01_000,
                        TextGridFit::SubPixel => 0b10_000,
                    },
                )?;
                self.write_f32(settings.thickness)?;
                self.write_f32(settings.sharpness)?;
                self.write_u8(0)?; // Reserved (0).
            }

            Tag::DefineBinaryData { id, ref data } => {
                self.write_tag_header(TagCode::DefineBinaryData, data.len() as u32 + 6)?;
                self.write_u16(id)?;
                self.write_u32(0)?; // Reserved
                self.output.write_all(data)?;
            }

            Tag::DefineBits { id, ref jpeg_data } => {
                self.write_tag_header(TagCode::DefineBits, jpeg_data.len() as u32 + 2)?;
                self.write_u16(id)?;
                self.output.write_all(jpeg_data)?;
            }

            Tag::DefineBitsJpeg2 { id, ref jpeg_data } => {
                self.write_tag_header(TagCode::DefineBitsJpeg2, jpeg_data.len() as u32 + 2)?;
                self.write_u16(id)?;
                self.output.write_all(jpeg_data)?;
            }

            Tag::DefineBitsJpeg3(ref jpeg) => {
                self.write_tag_header(
                    TagCode::DefineBitsJpeg3,
                    (jpeg.data.len() + jpeg.alpha_data.len() + 6) as u32,
                )?;
                self.write_u16(jpeg.id)?;
                if jpeg.version >= 4 {
                    self.write_fixed8(jpeg.deblocking)?;
                }
                // TODO(Herschel): Verify deblocking parameter is zero in version 3.
                self.write_u32(jpeg.data.len() as u32)?;
                self.output.write_all(&jpeg.data)?;
                self.output.write_all(&jpeg.alpha_data)?;
            }

            Tag::DefineBitsLossless(ref tag) => {
                let mut length = 7 + tag.data.len();
                if tag.format == BitmapFormat::ColorMap8 {
                    length += 1;
                }
                // TODO(Herschel): Throw error if RGB15 in tag version 2.
                let tag_code = if tag.version == 1 {
                    TagCode::DefineBitsLossless
                } else {
                    TagCode::DefineBitsLossless2
                };
                self.write_tag_header(tag_code, length as u32)?;
                self.write_character_id(tag.id)?;
                let format_id = match tag.format {
                    BitmapFormat::ColorMap8 => 3,
                    BitmapFormat::Rgb15 => 4,
                    BitmapFormat::Rgb32 => 5,
                };
                self.write_u8(format_id)?;
                self.write_u16(tag.width)?;
                self.write_u16(tag.height)?;
                if tag.format == BitmapFormat::ColorMap8 {
                    self.write_u8(tag.num_colors)?;
                }
                self.output.write_all(&tag.data)?;
            }

            Tag::DefineButton(ref button) => self.write_define_button(button)?,

            Tag::DefineButton2(ref button) => self.write_define_button_2(button)?,

            Tag::DefineButtonColorTransform(ref button_color) => {
                let mut buf = Vec::new();
                {
                    let mut writer = Writer::new(&mut buf, self.version);
                    writer.write_character_id(button_color.id)?;
                    for color_transform in &button_color.color_transforms {
                        writer.write_color_transform_no_alpha(color_transform)?;
                        writer.flush_bits()?;
                    }
                }
                self.write_tag_header(TagCode::DefineButtonCxform, buf.len() as u32)?;
                self.output.write_all(&buf)?;
            }

            Tag::DefineButtonSound(ref button_sounds) => {
                let mut buf = Vec::new();
                {
                    let mut writer = Writer::new(&mut buf, self.version);
                    writer.write_u16(button_sounds.id)?;
                    if let Some(ref sound) = button_sounds.over_to_up_sound {
                        writer.write_u16(sound.0)?;
                        writer.write_sound_info(&sound.1)?;
                    } else {
                        writer.write_u16(0)?
                    };
                    if let Some(ref sound) = button_sounds.up_to_over_sound {
                        writer.write_u16(sound.0)?;
                        writer.write_sound_info(&sound.1)?;
                    } else {
                        writer.write_u16(0)?
                    };
                    if let Some(ref sound) = button_sounds.over_to_down_sound {
                        writer.write_u16(sound.0)?;
                        writer.write_sound_info(&sound.1)?;
                    } else {
                        writer.write_u16(0)?
                    };
                    if let Some(ref sound) = button_sounds.down_to_over_sound {
                        writer.write_u16(sound.0)?;
                        writer.write_sound_info(&sound.1)?;
                    } else {
                        writer.write_u16(0)?
                    };
                }
                self.write_tag_header(TagCode::DefineButtonSound, buf.len() as u32)?;
                self.output.write_all(&buf)?;
            }

            Tag::DefineEditText(ref edit_text) => self.write_define_edit_text(edit_text)?,

            Tag::DefineFont(ref font) => {
                let num_glyphs = font.glyphs.len();
                let mut offsets = vec![];
                let mut buf = vec![];
                {
                    let mut writer = Writer::new(&mut buf, self.version);
                    for glyph in &font.glyphs {
                        let offset = num_glyphs * 2 + writer.output.len();
                        offsets.push(offset as u16);

                        // Bit length for fill and line indices.
                        // TODO: This theoretically could be >1?
                        writer.num_fill_bits = 1;
                        writer.num_line_bits = 0;
                        writer.write_ubits(4, 1)?;
                        writer.write_ubits(4, 0)?;

                        for shape_record in glyph {
                            writer.write_shape_record(shape_record, 1)?;
                        }
                        // End shape record.
                        writer.write_ubits(6, 0)?;
                        writer.flush_bits()?;
                    }
                }

                let tag_len = (2 + 2 * font.glyphs.len() + buf.len()) as u32;
                self.write_tag_header(TagCode::DefineFont, tag_len)?;
                self.write_u16(font.id)?;
                for offset in offsets {
                    self.write_u16(offset)?;
                }
                self.output.write_all(&buf)?;
            }

            Tag::DefineFont2(ref font) => self.write_define_font_2(font)?,
            Tag::DefineFont4(ref font) => self.write_define_font_4(font)?,

            Tag::DefineFontAlignZones {
                id,
                thickness,
                ref zones,
            } => {
                self.write_tag_header(TagCode::DefineFontAlignZones, 3 + 10 * zones.len() as u32)?;
                self.write_character_id(id)?;
                self.write_u8(match thickness {
                    FontThickness::Thin => 0b00_000000,
                    FontThickness::Medium => 0b01_000000,
                    FontThickness::Thick => 0b10_000000,
                })?;
                for zone in zones {
                    self.write_u8(2)?; // Always 2 dimensions.
                    self.write_i16(zone.left)?;
                    self.write_i16(zone.width)?;
                    self.write_i16(zone.bottom)?;
                    self.write_i16(zone.height)?;
                    self.write_u8(0b000000_11)?; // Always 2 dimensions.
                }
            }

            Tag::DefineFontInfo(ref font_info) => {
                let use_wide_codes = self.version >= 6 || font_info.version >= 2;

                let len = font_info.name.len()
                    + if use_wide_codes { 2 } else { 1 } * font_info.code_table.len()
                    + if font_info.version >= 2 { 1 } else { 0 }
                    + 4;

                let tag_id = if font_info.version == 1 {
                    TagCode::DefineFontInfo
                } else {
                    TagCode::DefineFontInfo2
                };
                self.write_tag_header(tag_id, len as u32)?;
                self.write_u16(font_info.id)?;

                // SWF19 has ANSI and Shift-JIS backwards?
                self.write_u8(font_info.name.len() as u8)?;
                self.output.write_all(font_info.name.as_bytes())?;
                self.write_u8(
                    if font_info.is_small_text { 0b100000 } else { 0 }
                        | if font_info.is_ansi { 0b10000 } else { 0 }
                        | if font_info.is_shift_jis { 0b1000 } else { 0 }
                        | if font_info.is_italic { 0b100 } else { 0 }
                        | if font_info.is_bold { 0b10 } else { 0 }
                        | if use_wide_codes { 0b1 } else { 0 },
                )?;
                // TODO(Herschel): Assert language is unknown for v1.
                if font_info.version >= 2 {
                    self.write_language(font_info.language)?;
                }
                for &code in &font_info.code_table {
                    if use_wide_codes {
                        self.write_u16(code)?;
                    } else {
                        self.write_u8(code as u8)?;
                    }
                }
            }

            Tag::DefineFontName {
                id,
                ref name,
                ref copyright_info,
            } => {
                let len = name.len() + copyright_info.len() + 4;
                self.write_tag_header(TagCode::DefineFontName, len as u32)?;
                self.write_character_id(id)?;
                self.write_c_string(name)?;
                self.write_c_string(copyright_info)?;
            }

            Tag::DefineMorphShape(ref define_morph_shape) => {
                self.write_define_morph_shape(define_morph_shape)?
            }

            Tag::DefineScalingGrid {
                id,
                ref splitter_rect,
            } => {
                let mut buf = Vec::new();
                {
                    let mut writer = Writer::new(&mut buf, self.version);
                    writer.write_u16(id)?;
                    writer.write_rectangle(splitter_rect)?;
                    writer.flush_bits()?;
                }
                self.write_tag_header(TagCode::DefineScalingGrid, buf.len() as u32)?;
                self.output.write_all(&buf)?;
            }

            Tag::DefineShape(ref shape) => self.write_define_shape(shape)?,
            Tag::DefineSound(ref sound) => self.write_define_sound(sound)?,
            Tag::DefineSprite(ref sprite) => self.write_define_sprite(sprite)?,
            Tag::DefineText(ref text) => self.write_define_text(text)?,
            Tag::DefineVideoStream(ref video) => self.write_define_video_stream(video)?,
            Tag::DoAbc(ref do_abc) => {
                let len = do_abc.data.len() + do_abc.name.len() + 5;
                self.write_tag_header(TagCode::DoAbc, len as u32)?;
                self.write_u32(if do_abc.is_lazy_initialize { 1 } else { 0 })?;
                self.write_c_string(&do_abc.name)?;
                self.output.write_all(&do_abc.data)?;
            }
            Tag::DoAction(ref action_data) => {
                self.write_tag_header(TagCode::DoAction, action_data.len() as u32)?;
                self.output.write_all(action_data)?;
            }
            Tag::DoInitAction {
                id,
                ref action_data,
            } => {
                self.write_tag_header(TagCode::DoInitAction, action_data.len() as u32 + 2)?;
                self.write_u16(id)?;
                self.output.write_all(action_data)?;
            }

            Tag::EnableDebugger(ref password_md5) => {
                let len = password_md5.len() as u32 + 1;
                if self.version >= 6 {
                    // SWF v6+ uses EnableDebugger2 tag.
                    self.write_tag_header(TagCode::EnableDebugger2, len + 2)?;
                    self.write_u16(0)?; // Reserved
                } else {
                    self.write_tag_header(TagCode::EnableDebugger, len)?;
                }

                self.write_c_string(password_md5)?;
            }

            Tag::EnableTelemetry { ref password_hash } => {
                if !password_hash.is_empty() {
                    self.write_tag_header(TagCode::EnableTelemetry, 34)?;
                    self.write_u16(0)?;
                    self.output.write_all(&password_hash[0..32])?;
                } else {
                    self.write_tag_header(TagCode::EnableTelemetry, 2)?;
                    self.write_u16(0)?;
                }
            }

            Tag::End => self.write_tag_header(TagCode::End, 0)?,

            Tag::ImportAssets {
                ref url,
                ref imports,
            } => {
                let len = imports.iter().map(|e| e.name.len() as u32 + 3).sum::<u32>()
                    + url.len() as u32
                    + 1
                    + 2;
                // SWF v8 and later use ImportAssets2 tag.
                if self.version >= 8 {
                    self.write_tag_header(TagCode::ImportAssets2, len + 2)?;
                    self.write_c_string(url)?;
                    self.write_u8(1)?;
                    self.write_u8(0)?;
                } else {
                    self.write_tag_header(TagCode::ImportAssets, len)?;
                    self.write_c_string(url)?;
                }
                self.write_u16(imports.len() as u16)?;
                for &ExportedAsset { id, ref name } in imports {
                    self.write_u16(id)?;
                    self.write_c_string(name)?;
                }
            }

            Tag::JpegTables(ref data) => {
                self.write_tag_header(TagCode::JpegTables, data.len() as u32)?;
                self.output.write_all(data)?;
            }

            Tag::Metadata(ref metadata) => {
                self.write_tag_header(TagCode::Metadata, metadata.len() as u32 + 1)?;
                self.write_c_string(metadata)?;
            }

            // TODO: Allow clone of color.
            Tag::SetBackgroundColor(ref color) => {
                self.write_tag_header(TagCode::SetBackgroundColor, 3)?;
                self.write_rgb(&color)?;
            }

            Tag::ScriptLimits {
                max_recursion_depth,
                timeout_in_seconds,
            } => {
                self.write_tag_header(TagCode::ScriptLimits, 4)?;
                self.write_u16(max_recursion_depth)?;
                self.write_u16(timeout_in_seconds)?;
            }

            Tag::SetTabIndex { depth, tab_index } => {
                self.write_tag_header(TagCode::SetTabIndex, 4)?;
                self.write_u16(depth)?;
                self.write_u16(tab_index)?;
            }

            Tag::PlaceObject(ref place_object) => match (*place_object).version {
                1 => self.write_place_object(place_object)?,
                2 => self.write_place_object_2_or_3(place_object, 2)?,
                3 => self.write_place_object_2_or_3(place_object, 3)?,
                4 => self.write_place_object_2_or_3(place_object, 4)?,
                _ => return Err(Error::invalid_data("Invalid PlaceObject version.")),
            },

            Tag::RemoveObject(ref remove_object) => {
                if let Some(id) = remove_object.character_id {
                    self.write_tag_header(TagCode::RemoveObject, 4)?;
                    self.write_u16(id)?;
                } else {
                    self.write_tag_header(TagCode::RemoveObject2, 2)?;
                }
                self.write_u16(remove_object.depth)?;
            }

            Tag::SoundStreamBlock(ref data) => {
                self.write_tag_header(TagCode::SoundStreamBlock, data.len() as u32)?;
                self.output.write_all(data)?;
            }

            Tag::SoundStreamHead(ref sound_stream_head) => {
                self.write_sound_stream_head(sound_stream_head, 1)?;
            }

            Tag::SoundStreamHead2(ref sound_stream_head) => {
                self.write_sound_stream_head(sound_stream_head, 2)?;
            }

            Tag::StartSound(ref start_sound) => {
                let sound_info = &start_sound.sound_info;
                let length = 3
                    + if sound_info.in_sample.is_some() { 4 } else { 0 }
                    + if sound_info.out_sample.is_some() {
                        4
                    } else {
                        0
                    }
                    + if sound_info.num_loops > 1 { 2 } else { 0 }
                    + if let Some(ref e) = sound_info.envelope {
                        e.len() as u32 * 8 + 1
                    } else {
                        0
                    };
                self.write_tag_header(TagCode::StartSound, length)?;
                self.write_u16(start_sound.id)?;
                self.write_sound_info(sound_info)?;
            }

            Tag::StartSound2 {
                ref class_name,
                ref sound_info,
            } => {
                let length = class_name.len() as u32
                    + 2
                    + if sound_info.in_sample.is_some() { 4 } else { 0 }
                    + if sound_info.out_sample.is_some() {
                        4
                    } else {
                        0
                    }
                    + if sound_info.num_loops > 1 { 2 } else { 0 }
                    + if let Some(ref e) = sound_info.envelope {
                        e.len() as u32 * 8 + 1
                    } else {
                        0
                    };
                self.write_tag_header(TagCode::StartSound2, length)?;
                self.write_c_string(class_name)?;
                self.write_sound_info(sound_info)?;
            }

            Tag::SymbolClass(ref symbols) => {
                let len = symbols
                    .iter()
                    .map(|e| e.class_name.len() as u32 + 3)
                    .sum::<u32>()
                    + 2;
                self.write_tag_header(TagCode::SymbolClass, len)?;
                self.write_u16(symbols.len() as u16)?;
                for &SymbolClassLink { id, ref class_name } in symbols {
                    self.write_u16(id)?;
                    self.write_c_string(class_name)?;
                }
            }

            Tag::VideoFrame(ref frame) => {
                self.write_tag_header(TagCode::VideoFrame, 4 + frame.data.len() as u32)?;
                self.write_character_id(frame.stream_id)?;
                self.write_u16(frame.frame_num)?;
                self.output.write_all(&frame.data)?;
            }

            Tag::FileAttributes(ref attributes) => {
                self.write_tag_header(TagCode::FileAttributes, 4)?;
                let mut flags = 0u32;
                if attributes.use_direct_blit {
                    flags |= 0b01000000;
                }
                if attributes.use_gpu {
                    flags |= 0b00100000;
                }
                if attributes.has_metadata {
                    flags |= 0b00010000;
                }
                if attributes.is_action_script_3 {
                    flags |= 0b00001000;
                }
                if attributes.use_network_sandbox {
                    flags |= 0b00000001;
                }
                self.write_u32(flags)?;
            }

            Tag::FrameLabel(FrameLabel {
                ref label,
                is_anchor,
            }) => {
                // TODO: Assert proper version
                let is_anchor = is_anchor && self.version >= 6;
                let length = label.len() as u32 + if is_anchor { 2 } else { 1 };
                self.write_tag_header(TagCode::FrameLabel, length)?;
                self.write_c_string(label)?;
                if is_anchor {
                    self.write_u8(1)?;
                }
            }

            Tag::DefineSceneAndFrameLabelData(ref data) => {
                self.write_define_scene_and_frame_label_data(data)?
            }
            Tag::ProductInfo(ref product_info) => self.write_product_info(product_info)?,
            Tag::DebugId(ref debug_id) => self.write_debug_id(debug_id)?,

            Tag::Unknown { tag_code, ref data } => {
                self.write_tag_code_and_length(tag_code, data.len() as u32)?;
                self.output.write_all(data)?;
            }
        }
        Ok(())
    }

    fn write_define_button(&mut self, button: &Button) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(button.id)?;
            for record in &button.records {
                writer.write_button_record(record, 1)?;
            }
            writer.write_u8(0)?; // End button records
                                 // TODO: Assert we have some action.
            writer.output.write_all(&button.actions[0].action_data)?;
        }
        self.write_tag_header(TagCode::DefineButton, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_button_2(&mut self, button: &Button) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(button.id)?;
            let flags = if button.is_track_as_menu { 1 } else { 0 };
            writer.write_u8(flags)?;

            let mut record_data = Vec::new();
            {
                let mut writer_2 = Writer::new(&mut record_data, self.version);
                for record in &button.records {
                    writer_2.write_button_record(record, 2)?;
                }
                writer_2.write_u8(0)?; // End button records
            }
            writer.write_u16(record_data.len() as u16 + 2)?;
            writer.output.write_all(&record_data)?;

            let mut iter = button.actions.iter().peekable();
            while let Some(action) = iter.next() {
                if iter.peek().is_some() {
                    let length = action.action_data.len() as u16 + 4;
                    writer.write_u16(length)?;
                } else {
                    writer.write_u16(0)?;
                }
                writer.write_u8(
                    if action
                        .conditions
                        .contains(&ButtonActionCondition::IdleToOverDown)
                    {
                        0b1000_0000
                    } else {
                        0
                    } | if action
                        .conditions
                        .contains(&ButtonActionCondition::OutDownToIdle)
                    {
                        0b100_0000
                    } else {
                        0
                    } | if action
                        .conditions
                        .contains(&ButtonActionCondition::OutDownToOverDown)
                    {
                        0b10_0000
                    } else {
                        0
                    } | if action
                        .conditions
                        .contains(&ButtonActionCondition::OverDownToOutDown)
                    {
                        0b1_0000
                    } else {
                        0
                    } | if action
                        .conditions
                        .contains(&ButtonActionCondition::OverDownToOverUp)
                    {
                        0b1000
                    } else {
                        0
                    } | if action
                        .conditions
                        .contains(&ButtonActionCondition::OverUpToOverDown)
                    {
                        0b100
                    } else {
                        0
                    } | if action
                        .conditions
                        .contains(&ButtonActionCondition::OverUpToIdle)
                    {
                        0b10
                    } else {
                        0
                    } | if action
                        .conditions
                        .contains(&ButtonActionCondition::IdleToOverUp)
                    {
                        0b1
                    } else {
                        0
                    },
                )?;
                let mut flags = if action
                    .conditions
                    .contains(&ButtonActionCondition::OverDownToIdle)
                {
                    0b1
                } else {
                    0
                };
                if action.conditions.contains(&ButtonActionCondition::KeyPress) {
                    if let Some(key_code) = action.key_code {
                        flags |= key_code << 1;
                    }
                }
                writer.write_u8(flags)?;
                writer.output.write_all(&action.action_data)?;
            }
        }
        self.write_tag_header(TagCode::DefineButton2, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_morph_shape(&mut self, data: &DefineMorphShape) -> Result<()> {
        if data.start.fill_styles.len() != data.end.fill_styles.len()
            || data.start.line_styles.len() != data.end.line_styles.len()
        {
            return Err(Error::invalid_data(
                "Start and end state of a morph shape must have the same number of styles.",
            ));
        }

        let num_fill_styles = data.start.fill_styles.len();
        let num_line_styles = data.start.line_styles.len();
        let num_fill_bits = count_ubits(num_fill_styles as u32);
        let num_line_bits = count_ubits(num_line_styles as u32);

        // Need to write styles first, to calculate offset to EndEdges.
        let mut start_buf = Vec::new();
        {
            let mut writer = Writer::new(&mut start_buf, self.version);

            // Styles
            // TODO(Herschel): Make fn write_style_len. Check version.
            if num_fill_styles >= 0xff {
                writer.write_u8(0xff)?;
                writer.write_u16(num_fill_styles as u16)?;
            } else {
                writer.write_u8(num_fill_styles as u8)?;
            }
            for (start, end) in data
                .start
                .fill_styles
                .iter()
                .zip(data.end.fill_styles.iter())
            {
                writer.write_morph_fill_style(start, end, data.version)?;
            }

            if num_line_styles >= 0xff {
                writer.write_u8(0xff)?;
                writer.write_u16(num_line_styles as u16)?;
            } else {
                writer.write_u8(num_line_styles as u8)?;
            }
            for (start, end) in data
                .start
                .line_styles
                .iter()
                .zip(data.end.line_styles.iter())
            {
                writer.write_morph_line_style(start, end, data.version)?;
            }

            // TODO(Herschel): Make fn write_shape.
            writer.write_ubits(4, num_fill_bits.into())?;
            writer.write_ubits(4, num_line_bits.into())?;
            writer.num_fill_bits = num_fill_bits;
            writer.num_line_bits = num_line_bits;
            for shape_record in &data.start.shape {
                writer.write_shape_record(shape_record, 1)?;
            }
            // End shape record.
            writer.write_ubits(6, 0)?;
            writer.flush_bits()?;
        }

        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(data.id)?;
            writer.write_rectangle(&data.start.shape_bounds)?;
            writer.write_rectangle(&data.end.shape_bounds)?;
            if data.version >= 2 {
                writer.write_rectangle(&data.start.edge_bounds)?;
                writer.write_rectangle(&data.end.edge_bounds)?;
                writer.write_u8(
                    if data.has_non_scaling_strokes {
                        0b10
                    } else {
                        0
                    } | if data.has_scaling_strokes { 0b1 } else { 0 },
                )?;
            }

            // Offset to EndEdges.
            writer.write_u32(start_buf.len() as u32)?;

            writer.output.write_all(&start_buf)?;

            // EndEdges.
            writer.write_u8(0)?; // NumFillBits and NumLineBits are written as 0 for the end shape.
            writer.num_fill_bits = num_fill_bits;
            writer.num_line_bits = num_line_bits;
            for shape_record in &data.end.shape {
                writer.write_shape_record(shape_record, 1)?;
            }
            // End shape record.
            writer.write_ubits(6, 0)?;
            writer.flush_bits()?;
        }

        let tag_code = if data.version == 1 {
            TagCode::DefineMorphShape
        } else {
            TagCode::DefineMorphShape2
        };
        self.write_tag_header(tag_code, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_morph_fill_style(
        &mut self,
        start: &FillStyle,
        end: &FillStyle,
        shape_version: u8,
    ) -> Result<()> {
        match (start, end) {
            (&FillStyle::Color(ref start_color), &FillStyle::Color(ref end_color)) => {
                self.write_u8(0x00)?; // Solid color.
                self.write_rgba(start_color)?;
                self.write_rgba(end_color)?;
            }

            (
                &FillStyle::LinearGradient(ref start_gradient),
                &FillStyle::LinearGradient(ref end_gradient),
            ) => {
                self.write_u8(0x10)?; // Linear gradient.
                self.write_morph_gradient(start_gradient, end_gradient)?;
            }

            (
                &FillStyle::RadialGradient(ref start_gradient),
                &FillStyle::RadialGradient(ref end_gradient),
            ) => {
                self.write_u8(0x12)?; // Linear gradient.
                self.write_morph_gradient(start_gradient, end_gradient)?;
            }

            (
                &FillStyle::FocalGradient {
                    gradient: ref start_gradient,
                    focal_point: start_focal_point,
                },
                &FillStyle::FocalGradient {
                    gradient: ref end_gradient,
                    focal_point: end_focal_point,
                },
            ) => {
                if self.version < 8 || shape_version < 2 {
                    return Err(Error::invalid_data(
                        "Focal gradients are only support in SWF version 8 \
                         and higher.",
                    ));
                }

                self.write_u8(0x13)?; // Focal gradient.
                self.write_morph_gradient(start_gradient, end_gradient)?;
                self.write_fixed8(start_focal_point)?;
                self.write_fixed8(end_focal_point)?;
            }

            (
                &FillStyle::Bitmap {
                    id,
                    matrix: ref start_matrix,
                    is_smoothed,
                    is_repeating,
                },
                &FillStyle::Bitmap {
                    id: end_id,
                    matrix: ref end_matrix,
                    is_smoothed: end_is_smoothed,
                    is_repeating: end_is_repeating,
                },
            ) if id == end_id && is_smoothed == end_is_smoothed
                || is_repeating == end_is_repeating =>
            {
                let fill_style_type = match (is_smoothed, is_repeating) {
                    (true, true) => 0x40,
                    (true, false) => 0x41,
                    (false, true) => 0x42,
                    (false, false) => 0x43,
                };
                self.write_u8(fill_style_type)?;
                self.write_u16(id)?;
                self.write_matrix(start_matrix)?;
                self.write_matrix(end_matrix)?;
            }

            _ => {
                return Err(Error::invalid_data(
                    "Morph start and end fill styles must be the same variant.",
                ))
            }
        }
        Ok(())
    }

    fn write_morph_gradient(&mut self, start: &Gradient, end: &Gradient) -> Result<()> {
        self.write_matrix(&start.matrix)?;
        self.write_matrix(&end.matrix)?;
        if start.records.len() != end.records.len() {
            return Err(Error::invalid_data(
                "Morph start and end gradient must have the same amount of records.",
            ));
        }
        self.write_gradient_flags(start)?;
        for (start_record, end_record) in start.records.iter().zip(end.records.iter()) {
            self.write_u8(start_record.ratio)?;
            self.write_rgba(&start_record.color)?;
            self.write_u8(end_record.ratio)?;
            self.write_rgba(&end_record.color)?;
        }
        Ok(())
    }

    fn write_morph_line_style(
        &mut self,
        start: &LineStyle,
        end: &LineStyle,
        shape_version: u8,
    ) -> Result<()> {
        if shape_version < 2 {
            // TODO(Herschel): Handle overflow.
            self.write_u16(start.width.get() as u16)?;
            self.write_u16(end.width.get() as u16)?;
            self.write_rgba(&start.color)?;
            self.write_rgba(&end.color)?;
        } else {
            if start.start_cap != end.start_cap
                || start.join_style != end.join_style
                || start.allow_scale_x != end.allow_scale_x
                || start.allow_scale_y != end.allow_scale_y
                || start.is_pixel_hinted != end.is_pixel_hinted
                || start.allow_close != end.allow_close
                || start.end_cap != end.end_cap
            {
                return Err(Error::invalid_data(
                    "Morph start and end line styles must have the same join parameters.",
                ));
            }

            // TODO(Herschel): Handle overflow.
            self.write_u16(start.width.get() as u16)?;
            self.write_u16(end.width.get() as u16)?;

            // MorphLineStyle2
            self.write_ubits(
                2,
                match start.start_cap {
                    LineCapStyle::Round => 0,
                    LineCapStyle::None => 1,
                    LineCapStyle::Square => 2,
                },
            )?;
            self.write_ubits(
                2,
                match start.join_style {
                    LineJoinStyle::Round => 0,
                    LineJoinStyle::Bevel => 1,
                    LineJoinStyle::Miter(_) => 2,
                },
            )?;
            self.write_bit(start.fill_style.is_some())?;
            self.write_bit(!start.allow_scale_x)?;
            self.write_bit(!start.allow_scale_y)?;
            self.write_bit(start.is_pixel_hinted)?;
            self.write_ubits(5, 0)?;
            self.write_bit(!start.allow_close)?;
            self.write_ubits(
                2,
                match start.end_cap {
                    LineCapStyle::Round => 0,
                    LineCapStyle::None => 1,
                    LineCapStyle::Square => 2,
                },
            )?;
            if let LineJoinStyle::Miter(miter_factor) = start.join_style {
                self.write_fixed8(miter_factor)?;
            }
            match (&start.fill_style, &end.fill_style) {
                (&None, &None) => {
                    self.write_rgba(&start.color)?;
                    self.write_rgba(&end.color)?;
                }

                (&Some(ref start_fill), &Some(ref end_fill)) => {
                    self.write_morph_fill_style(start_fill, end_fill, shape_version)?
                }

                _ => {
                    return Err(Error::invalid_data(
                        "Morph start and end line styles must both have fill styles.",
                    ))
                }
            }
        }
        Ok(())
    }

    fn write_define_scene_and_frame_label_data(
        &mut self,
        data: &DefineSceneAndFrameLabelData,
    ) -> Result<()> {
        let mut buf = Vec::with_capacity((data.scenes.len() + data.frame_labels.len()) * 4);
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_encoded_u32(data.scenes.len() as u32)?;
            for scene in &data.scenes {
                writer.write_encoded_u32(scene.frame_num)?;
                writer.write_c_string(&scene.label)?;
            }
            writer.write_encoded_u32(data.frame_labels.len() as u32)?;
            for frame_label in &data.frame_labels {
                writer.write_encoded_u32(frame_label.frame_num)?;
                writer.write_c_string(&frame_label.label)?;
            }
        }
        self.write_tag_header(TagCode::DefineSceneAndFrameLabelData, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_shape(&mut self, shape: &Shape) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(shape.id)?;
            writer.write_rectangle(&shape.shape_bounds)?;
            if shape.version >= 4 {
                writer.write_rectangle(&shape.edge_bounds)?;
                writer.flush_bits()?;
                writer.write_u8(
                    if shape.has_fill_winding_rule {
                        0b100
                    } else {
                        0
                    } | if shape.has_non_scaling_strokes {
                        0b10
                    } else {
                        0
                    } | if shape.has_scaling_strokes { 0b1 } else { 0 },
                )?;
            }

            writer.write_shape_styles(&shape.styles, shape.version)?;

            for shape_record in &shape.shape {
                writer.write_shape_record(shape_record, shape.version)?;
            }
            // End shape record.
            writer.write_ubits(6, 0)?;
            writer.flush_bits()?;
        }

        let tag_code = match shape.version {
            1 => TagCode::DefineShape,
            2 => TagCode::DefineShape2,
            3 => TagCode::DefineShape3,
            4 => TagCode::DefineShape4,
            _ => return Err(Error::invalid_data("Invalid DefineShape version.")),
        };
        self.write_tag_header(tag_code, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_sound(&mut self, sound: &Sound) -> Result<()> {
        self.write_tag_header(TagCode::DefineSound, 7 + sound.data.len() as u32)?;
        self.write_u16(sound.id)?;
        self.write_sound_format(&sound.format)?;
        self.write_u32(sound.num_samples)?;
        self.output.write_all(&sound.data)?;
        Ok(())
    }

    fn write_define_sprite(&mut self, sprite: &Sprite) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(sprite.id)?;
            writer.write_u16(sprite.num_frames)?;
            writer.write_tag_list(&sprite.tags)?;
        };
        self.write_tag_header(TagCode::DefineSprite, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_export_assets(&mut self, exports: &[ExportedAsset]) -> Result<()> {
        let len = exports.iter().map(|e| e.name.len() as u32 + 1).sum::<u32>()
            + exports.len() as u32 * 2
            + 2;
        self.write_tag_header(TagCode::ExportAssets, len)?;
        self.write_u16(exports.len() as u16)?;
        for &ExportedAsset { id, ref name } in exports {
            self.write_u16(id)?;
            self.write_c_string(name)?;
        }
        Ok(())
    }

    fn write_button_record(&mut self, record: &ButtonRecord, tag_version: u8) -> Result<()> {
        // TODO: Validate version
        let flags = if record.blend_mode != BlendMode::Normal {
            0b10_0000
        } else {
            0
        } | if !record.filters.is_empty() {
            0b1_0000
        } else {
            0
        } | if record.states.contains(&ButtonState::HitTest) {
            0b1000
        } else {
            0
        } | if record.states.contains(&ButtonState::Down) {
            0b100
        } else {
            0
        } | if record.states.contains(&ButtonState::Over) {
            0b10
        } else {
            0
        } | if record.states.contains(&ButtonState::Up) {
            0b1
        } else {
            0
        };
        self.write_u8(flags)?;
        self.write_u16(record.id)?;
        self.write_u16(record.depth)?;
        self.write_matrix(&record.matrix)?;
        if tag_version >= 2 {
            self.write_color_transform(&record.color_transform)?;
            if !record.filters.is_empty() {
                self.write_u8(record.filters.len() as u8)?;
                for filter in &record.filters {
                    self.write_filter(filter)?;
                }
            }
            if record.blend_mode != BlendMode::Normal {
                self.write_blend_mode(record.blend_mode)?;
            }
        }
        Ok(())
    }

    fn write_blend_mode(&mut self, blend_mode: BlendMode) -> Result<()> {
        self.write_u8(match blend_mode {
            BlendMode::Normal => 0,
            BlendMode::Layer => 2,
            BlendMode::Multiply => 3,
            BlendMode::Screen => 4,
            BlendMode::Lighten => 5,
            BlendMode::Darken => 6,
            BlendMode::Difference => 7,
            BlendMode::Add => 8,
            BlendMode::Subtract => 9,
            BlendMode::Invert => 10,
            BlendMode::Alpha => 11,
            BlendMode::Erase => 12,
            BlendMode::Overlay => 13,
            BlendMode::HardLight => 14,
        })?;
        Ok(())
    }

    fn write_shape_styles(&mut self, styles: &ShapeStyles, shape_version: u8) -> Result<()> {
        // TODO: Check shape_version.
        if styles.fill_styles.len() >= 0xff {
            self.write_u8(0xff)?;
            self.write_u16(styles.fill_styles.len() as u16)?;
        } else {
            self.write_u8(styles.fill_styles.len() as u8)?;
        }
        for fill_style in &styles.fill_styles {
            self.write_fill_style(fill_style, shape_version)?;
        }

        if styles.line_styles.len() >= 0xff {
            self.write_u8(0xff)?;
            self.write_u16(styles.line_styles.len() as u16)?;
        } else {
            self.write_u8(styles.line_styles.len() as u8)?;
        }
        for line_style in &styles.line_styles {
            self.write_line_style(line_style, shape_version)?;
        }

        let num_fill_bits = count_ubits(styles.fill_styles.len() as u32);
        let num_line_bits = count_ubits(styles.line_styles.len() as u32);
        self.write_ubits(4, num_fill_bits.into())?;
        self.write_ubits(4, num_line_bits.into())?;
        self.num_fill_bits = num_fill_bits;
        self.num_line_bits = num_line_bits;
        Ok(())
    }

    fn write_shape_record(&mut self, record: &ShapeRecord, shape_version: u8) -> Result<()> {
        match *record {
            ShapeRecord::StraightEdge { delta_x, delta_y } => {
                self.write_ubits(2, 0b11)?; // Straight edge
                                            // TODO: Check underflow?
                let mut num_bits = max(count_sbits_twips(delta_x), count_sbits_twips(delta_y));
                num_bits = max(2, num_bits);
                let is_axis_aligned = delta_x.get() == 0 || delta_y.get() == 0;
                self.write_ubits(4, u32::from(num_bits) - 2)?;
                self.write_bit(!is_axis_aligned)?;
                if is_axis_aligned {
                    self.write_bit(delta_x.get() == 0)?;
                }
                if delta_x.get() != 0 {
                    self.write_sbits_twips(num_bits, delta_x)?;
                }
                if delta_y.get() != 0 {
                    self.write_sbits_twips(num_bits, delta_y)?;
                }
            }
            ShapeRecord::CurvedEdge {
                control_delta_x,
                control_delta_y,
                anchor_delta_x,
                anchor_delta_y,
            } => {
                self.write_ubits(2, 0b10)?; // Curved edge
                let num_bits = [
                    control_delta_x,
                    control_delta_y,
                    anchor_delta_x,
                    anchor_delta_y,
                ]
                .iter()
                .map(|x| count_sbits_twips(*x))
                .max()
                .unwrap();
                self.write_ubits(4, u32::from(num_bits) - 2)?;
                self.write_sbits_twips(num_bits, control_delta_x)?;
                self.write_sbits_twips(num_bits, control_delta_y)?;
                self.write_sbits_twips(num_bits, anchor_delta_x)?;
                self.write_sbits_twips(num_bits, anchor_delta_y)?;
            }
            ShapeRecord::StyleChange(ref style_change) => {
                self.write_bit(false)?; // Style change
                let num_fill_bits = self.num_fill_bits;
                let num_line_bits = self.num_line_bits;
                self.write_bit(style_change.new_styles.is_some())?;
                self.write_bit(style_change.line_style.is_some())?;
                self.write_bit(style_change.fill_style_1.is_some())?;
                self.write_bit(style_change.fill_style_0.is_some())?;
                self.write_bit(style_change.move_to.is_some())?;
                if let Some((move_x, move_y)) = style_change.move_to {
                    let num_bits = max(count_sbits_twips(move_x), count_sbits_twips(move_y));
                    self.write_ubits(5, num_bits.into())?;
                    self.write_sbits_twips(num_bits, move_x)?;
                    self.write_sbits_twips(num_bits, move_y)?;
                }
                if let Some(fill_style_index) = style_change.fill_style_0 {
                    self.write_ubits(num_fill_bits, fill_style_index)?;
                }
                if let Some(fill_style_index) = style_change.fill_style_1 {
                    self.write_ubits(num_fill_bits, fill_style_index)?;
                }
                if let Some(line_style_index) = style_change.line_style {
                    self.write_ubits(num_line_bits, line_style_index)?;
                }
                if let Some(ref new_styles) = style_change.new_styles {
                    if shape_version < 2 {
                        return Err(Error::invalid_data(
                            "Only DefineShape2 and higher may change styles.",
                        ));
                    }
                    self.write_shape_styles(new_styles, shape_version)?;
                }
            }
        }
        Ok(())
    }

    fn write_fill_style(&mut self, fill_style: &FillStyle, shape_version: u8) -> Result<()> {
        match *fill_style {
            FillStyle::Color(ref color) => {
                self.write_u8(0x00)?; // Solid color.
                if shape_version >= 3 {
                    self.write_rgba(color)?
                } else {
                    self.write_rgb(color)?;
                }
            }

            FillStyle::LinearGradient(ref gradient) => {
                self.write_u8(0x10)?; // Linear gradient.
                self.write_gradient(gradient, shape_version)?;
            }

            FillStyle::RadialGradient(ref gradient) => {
                self.write_u8(0x12)?; // Linear gradient.
                self.write_gradient(gradient, shape_version)?;
            }

            FillStyle::FocalGradient {
                ref gradient,
                focal_point,
            } => {
                if self.version < 8 {
                    return Err(Error::invalid_data(
                        "Focal gradients are only support in SWF version 8 \
                         and higher.",
                    ));
                }

                self.write_u8(0x13)?; // Focal gradient.
                self.write_gradient(gradient, shape_version)?;
                self.write_fixed8(focal_point)?;
            }

            FillStyle::Bitmap {
                id,
                ref matrix,
                is_smoothed,
                is_repeating,
            } => {
                // Bitmap smoothing only an option in SWF version 8+.
                // Lower versions use 0x40 and 0x41 type even when unsmoothed.
                let fill_style_type = match (is_smoothed || self.version < 8, is_repeating) {
                    (true, true) => 0x40,
                    (true, false) => 0x41,
                    (false, true) => 0x42,
                    (false, false) => 0x43,
                };
                self.write_u8(fill_style_type)?;
                self.write_u16(id)?;
                self.write_matrix(matrix)?;
            }
        }
        Ok(())
    }

    fn write_line_style(&mut self, line_style: &LineStyle, shape_version: u8) -> Result<()> {
        // TODO(Herschel): Handle overflow.
        self.write_u16(line_style.width.get() as u16)?;
        if shape_version >= 4 {
            // LineStyle2
            self.write_ubits(
                2,
                match line_style.start_cap {
                    LineCapStyle::Round => 0,
                    LineCapStyle::None => 1,
                    LineCapStyle::Square => 2,
                },
            )?;
            self.write_ubits(
                2,
                match line_style.join_style {
                    LineJoinStyle::Round => 0,
                    LineJoinStyle::Bevel => 1,
                    LineJoinStyle::Miter(_) => 2,
                },
            )?;
            self.write_bit(line_style.fill_style.is_some())?;
            self.write_bit(!line_style.allow_scale_x)?;
            self.write_bit(!line_style.allow_scale_y)?;
            self.write_bit(line_style.is_pixel_hinted)?;
            self.write_ubits(5, 0)?;
            self.write_bit(!line_style.allow_close)?;
            self.write_ubits(
                2,
                match line_style.end_cap {
                    LineCapStyle::Round => 0,
                    LineCapStyle::None => 1,
                    LineCapStyle::Square => 2,
                },
            )?;
            if let LineJoinStyle::Miter(miter_factor) = line_style.join_style {
                self.write_fixed8(miter_factor)?;
            }
            match line_style.fill_style {
                None => self.write_rgba(&line_style.color)?,
                Some(ref fill) => self.write_fill_style(fill, shape_version)?,
            }
        } else if shape_version >= 3 {
            // LineStyle1 with RGBA
            self.write_rgba(&line_style.color)?;
        } else {
            // LineStyle1 with RGB
            self.write_rgb(&line_style.color)?;
        }
        Ok(())
    }

    fn write_gradient(&mut self, gradient: &Gradient, shape_version: u8) -> Result<()> {
        self.write_matrix(&gradient.matrix)?;
        self.flush_bits()?;
        self.write_gradient_flags(gradient)?;
        for record in &gradient.records {
            self.write_u8(record.ratio)?;
            if shape_version >= 3 {
                self.write_rgba(&record.color)?;
            } else {
                self.write_rgb(&record.color)?;
            }
        }
        Ok(())
    }

    fn write_gradient_flags(&mut self, gradient: &Gradient) -> Result<()> {
        let mut flags = 0;
        flags |= match &gradient.spread {
            GradientSpread::Pad => 0,
            GradientSpread::Reflect => 0b0100_0000,
            GradientSpread::Repeat => 0b1000_0000,
        };

        flags |= match &gradient.interpolation {
            GradientInterpolation::RGB => 0b00_0000,
            GradientInterpolation::LinearRGB => 0b_01_0000,
        };

        flags |= (gradient.records.len() as u8) & 0b1111;
        self.write_u8(flags)?;
        Ok(())
    }

    fn write_place_object(&mut self, place_object: &PlaceObject) -> Result<()> {
        // TODO: Assert that the extraneous fields are the defaults.
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            if let PlaceObjectAction::Place(character_id) = place_object.action {
                writer.write_u16(character_id)?;
            } else {
                return Err(Error::invalid_data(
                    "PlaceObject version 1 can only use a Place action.",
                ));
            }
            writer.write_u16(place_object.depth)?;
            if let Some(ref matrix) = place_object.matrix {
                writer.write_matrix(matrix)?;
            } else {
                writer.write_matrix(&Matrix::new())?;
            }
            if let Some(ref color_transform) = place_object.color_transform {
                writer.write_color_transform_no_alpha(color_transform)?;
            }
        }
        self.write_tag_header(TagCode::PlaceObject, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_place_object_2_or_3(
        &mut self,
        place_object: &PlaceObject,
        place_object_version: u8,
    ) -> Result<()> {
        let mut buf = Vec::new();
        {
            // TODO: Assert version.
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u8(
                if !place_object.clip_actions.is_empty() {
                    0b1000_0000
                } else {
                    0
                } | if place_object.clip_depth.is_some() {
                    0b0100_0000
                } else {
                    0
                } | if place_object.name.is_some() {
                    0b0010_0000
                } else {
                    0
                } | if place_object.ratio.is_some() {
                    0b0001_0000
                } else {
                    0
                } | if place_object.color_transform.is_some() {
                    0b0000_1000
                } else {
                    0
                } | if place_object.matrix.is_some() {
                    0b0000_0100
                } else {
                    0
                } | match place_object.action {
                    PlaceObjectAction::Place(_) => 0b10,
                    PlaceObjectAction::Modify => 0b01,
                    PlaceObjectAction::Replace(_) => 0b11,
                },
            )?;
            if place_object_version >= 3 {
                writer.write_u8(
                    if place_object.background_color.is_some() {
                        0b100_0000
                    } else {
                        0
                    } | if !place_object.is_visible {
                        0b10_0000
                    } else {
                        0
                    } | if place_object.is_image { 0b1_0000 } else { 0 }
                        | if place_object.class_name.is_some() {
                            0b1000
                        } else {
                            0
                        }
                        | if place_object.is_bitmap_cached {
                            0b100
                        } else {
                            0
                        }
                        | if place_object.blend_mode != BlendMode::Normal {
                            0b10
                        } else {
                            0
                        }
                        | if !place_object.filters.is_empty() {
                            0b1
                        } else {
                            0
                        },
                )?;
            }
            writer.write_u16(place_object.depth)?;

            if place_object_version >= 3 {
                if let Some(ref class_name) = place_object.class_name {
                    writer.write_c_string(class_name)?;
                }
            }

            match place_object.action {
                PlaceObjectAction::Place(character_id)
                | PlaceObjectAction::Replace(character_id) => writer.write_u16(character_id)?,
                PlaceObjectAction::Modify => (),
            }
            if let Some(ref matrix) = place_object.matrix {
                writer.write_matrix(matrix)?;
            };
            if let Some(ref color_transform) = place_object.color_transform {
                writer.write_color_transform(color_transform)?;
            };
            if let Some(ratio) = place_object.ratio {
                writer.write_u16(ratio)?;
            }
            if let Some(ref name) = place_object.name {
                writer.write_c_string(name)?;
            };
            if let Some(clip_depth) = place_object.clip_depth {
                writer.write_u16(clip_depth)?;
            }

            if place_object_version >= 3 {
                if !place_object.filters.is_empty() {
                    writer.write_u8(place_object.filters.len() as u8)?;
                    for filter in &place_object.filters {
                        writer.write_filter(filter)?;
                    }
                }

                if place_object.blend_mode != BlendMode::Normal {
                    writer.write_blend_mode(place_object.blend_mode)?;
                }

                if place_object.is_bitmap_cached {
                    writer.write_u8(1)?;
                }

                if !place_object.is_visible {
                    writer.write_u8(0)?;
                }

                if let Some(ref background_color) = place_object.background_color {
                    writer.write_rgba(background_color)?;
                }
            }

            if !place_object.clip_actions.is_empty() {
                writer.write_clip_actions(&place_object.clip_actions)?;
            }
            writer.flush_bits()?;

            // PlaceObject4 adds some embedded AMF data per instance.
            if place_object_version >= 4 {
                if let Some(ref data) = place_object.amf_data {
                    writer.output.write_all(data)?;
                }
            }
        }
        let tag_code = match place_object_version {
            2 => TagCode::PlaceObject2,
            3 => TagCode::PlaceObject3,
            4 => TagCode::PlaceObject4,
            _ => return Err(Error::invalid_data("Invalid PlaceObject version.")),
        };
        self.write_tag_header(tag_code, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_filter(&mut self, filter: &Filter) -> Result<()> {
        match *filter {
            Filter::DropShadowFilter(ref drop_shadow) => {
                self.write_u8(0)?;
                self.write_rgba(&drop_shadow.color)?;
                self.write_fixed16(drop_shadow.blur_x)?;
                self.write_fixed16(drop_shadow.blur_y)?;
                self.write_fixed16(drop_shadow.angle)?;
                self.write_fixed16(drop_shadow.distance)?;
                self.write_fixed8(drop_shadow.strength)?;
                self.write_bit(drop_shadow.is_inner)?;
                self.write_bit(drop_shadow.is_knockout)?;
                self.write_bit(true)?;
                self.write_ubits(5, drop_shadow.num_passes.into())?;
            }

            Filter::BlurFilter(ref blur) => {
                self.write_u8(1)?;
                self.write_fixed16(blur.blur_x)?;
                self.write_fixed16(blur.blur_y)?;
                self.write_u8(blur.num_passes << 3)?;
            }

            Filter::GlowFilter(ref glow) => {
                self.write_u8(2)?;
                self.write_rgba(&glow.color)?;
                self.write_fixed16(glow.blur_x)?;
                self.write_fixed16(glow.blur_y)?;
                self.write_fixed8(glow.strength)?;
                self.write_bit(glow.is_inner)?;
                self.write_bit(glow.is_knockout)?;
                self.write_bit(true)?;
                self.write_ubits(5, glow.num_passes.into())?;
            }

            Filter::BevelFilter(ref bevel) => {
                self.write_u8(3)?;
                self.write_rgba(&bevel.shadow_color)?;
                self.write_rgba(&bevel.highlight_color)?;
                self.write_fixed16(bevel.blur_x)?;
                self.write_fixed16(bevel.blur_y)?;
                self.write_fixed16(bevel.angle)?;
                self.write_fixed16(bevel.distance)?;
                self.write_fixed8(bevel.strength)?;
                self.write_bit(bevel.is_inner)?;
                self.write_bit(bevel.is_knockout)?;
                self.write_bit(true)?;
                self.write_bit(bevel.is_on_top)?;
                self.write_ubits(4, bevel.num_passes.into())?;
            }

            Filter::GradientGlowFilter(ref glow) => {
                self.write_u8(4)?;
                self.write_u8(glow.colors.len() as u8)?;
                for gradient_record in &glow.colors {
                    self.write_rgba(&gradient_record.color)?;
                }
                for gradient_record in &glow.colors {
                    self.write_u8(gradient_record.ratio)?;
                }
                self.write_fixed16(glow.blur_x)?;
                self.write_fixed16(glow.blur_y)?;
                self.write_fixed16(glow.angle)?;
                self.write_fixed16(glow.distance)?;
                self.write_fixed8(glow.strength)?;
                self.write_bit(glow.is_inner)?;
                self.write_bit(glow.is_knockout)?;
                self.write_bit(true)?;
                self.write_bit(glow.is_on_top)?;
                self.write_ubits(4, glow.num_passes.into())?;
            }

            Filter::ConvolutionFilter(ref convolve) => {
                self.write_u8(5)?;
                self.write_u8(convolve.num_matrix_cols)?;
                self.write_u8(convolve.num_matrix_rows)?;
                self.write_fixed16(convolve.divisor)?;
                self.write_fixed16(convolve.bias)?;
                for val in &convolve.matrix {
                    self.write_fixed16(*val)?;
                }
                self.write_rgba(&convolve.default_color)?;
                self.write_u8(
                    if convolve.is_clamped { 0b10 } else { 0 }
                        | if convolve.is_preserve_alpha { 0b1 } else { 0 },
                )?;
            }

            Filter::ColorMatrixFilter(ref color_matrix) => {
                self.write_u8(6)?;
                for i in 0..20 {
                    self.write_fixed16(color_matrix.matrix[i])?;
                }
            }

            Filter::GradientBevelFilter(ref bevel) => {
                self.write_u8(7)?;
                self.write_u8(bevel.colors.len() as u8)?;
                for gradient_record in &bevel.colors {
                    self.write_rgba(&gradient_record.color)?;
                }
                for gradient_record in &bevel.colors {
                    self.write_u8(gradient_record.ratio)?;
                }
                self.write_fixed16(bevel.blur_x)?;
                self.write_fixed16(bevel.blur_y)?;
                self.write_fixed16(bevel.angle)?;
                self.write_fixed16(bevel.distance)?;
                self.write_fixed8(bevel.strength)?;
                self.write_bit(bevel.is_inner)?;
                self.write_bit(bevel.is_knockout)?;
                self.write_bit(true)?;
                self.write_bit(bevel.is_on_top)?;
                self.write_ubits(4, bevel.num_passes.into())?;
            }
        }
        self.flush_bits()?;
        Ok(())
    }

    fn write_clip_actions(&mut self, clip_actions: &[ClipAction]) -> Result<()> {
        self.write_u16(0)?; // Reserved
        {
            let mut all_events = EnumSet::new();
            for action in clip_actions {
                all_events |= action.events;
            }
            self.write_clip_event_flags(all_events)?;
        }
        for action in clip_actions {
            self.write_clip_event_flags(action.events)?;
            let action_length =
                action.action_data.len() as u32 + if action.key_code.is_some() { 1 } else { 0 };
            self.write_u32(action_length)?;
            if let Some(k) = action.key_code {
                self.write_u8(k)?;
            }
            self.output.write_all(&action.action_data)?;
        }
        if self.version <= 5 {
            self.write_u16(0)?;
        } else {
            self.write_u32(0)?;
        }
        Ok(())
    }

    fn write_clip_event_flags(&mut self, clip_events: EnumSet<ClipEventFlag>) -> Result<()> {
        // TODO: Assert proper version.
        self.write_bit(clip_events.contains(ClipEventFlag::KeyUp))?;
        self.write_bit(clip_events.contains(ClipEventFlag::KeyDown))?;
        self.write_bit(clip_events.contains(ClipEventFlag::MouseUp))?;
        self.write_bit(clip_events.contains(ClipEventFlag::MouseDown))?;
        self.write_bit(clip_events.contains(ClipEventFlag::MouseMove))?;
        self.write_bit(clip_events.contains(ClipEventFlag::Unload))?;
        self.write_bit(clip_events.contains(ClipEventFlag::EnterFrame))?;
        self.write_bit(clip_events.contains(ClipEventFlag::Load))?;
        self.write_bit(clip_events.contains(ClipEventFlag::DragOver))?;
        self.write_bit(clip_events.contains(ClipEventFlag::RollOut))?;
        self.write_bit(clip_events.contains(ClipEventFlag::RollOver))?;
        self.write_bit(clip_events.contains(ClipEventFlag::ReleaseOutside))?;
        self.write_bit(clip_events.contains(ClipEventFlag::Release))?;
        self.write_bit(clip_events.contains(ClipEventFlag::Press))?;
        self.write_bit(clip_events.contains(ClipEventFlag::Initialize))?;
        self.write_bit(clip_events.contains(ClipEventFlag::Data))?;
        if self.version >= 6 {
            self.write_ubits(5, 0)?;
            let has_construct = self.version >= 7 && clip_events.contains(ClipEventFlag::Construct);
            self.write_bit(has_construct)?;
            self.write_bit(clip_events.contains(ClipEventFlag::KeyPress))?;
            self.write_bit(clip_events.contains(ClipEventFlag::DragOut))?;
            self.write_u8(0)?;
        }
        self.flush_bits()?;
        Ok(())
    }

    fn write_sound_stream_head(
        &mut self,
        stream_head: &SoundStreamHead,
        version: u8,
    ) -> Result<()> {
        let tag_code = if version >= 2 {
            TagCode::SoundStreamHead2
        } else {
            TagCode::SoundStreamHead
        };
        // MP3 compression has added latency seek field.
        let length = if stream_head.stream_format.compression == AudioCompression::Mp3 {
            6
        } else {
            4
        };
        self.write_tag_header(tag_code, length)?;
        self.write_sound_format(&stream_head.playback_format)?;
        self.write_sound_format(&stream_head.stream_format)?;
        self.write_u16(stream_head.num_samples_per_block)?;
        if stream_head.stream_format.compression == AudioCompression::Mp3 {
            self.write_i16(stream_head.latency_seek)?;
        }
        Ok(())
    }

    fn write_sound_format(&mut self, sound_format: &SoundFormat) -> Result<()> {
        self.write_ubits(
            4,
            match sound_format.compression {
                AudioCompression::UncompressedUnknownEndian => 0,
                AudioCompression::Adpcm => 1,
                AudioCompression::Mp3 => 2,
                AudioCompression::Uncompressed => 3,
                AudioCompression::Nellymoser16Khz => 4,
                AudioCompression::Nellymoser8Khz => 5,
                AudioCompression::Nellymoser => 6,
                AudioCompression::Speex => 11,
            },
        )?;
        self.write_ubits(
            2,
            match sound_format.sample_rate {
                5512 => 0,
                11025 => 1,
                22050 => 2,
                44100 => 3,
                _ => return Err(Error::invalid_data("Invalid sample rate.")),
            },
        )?;
        self.write_bit(sound_format.is_16_bit)?;
        self.write_bit(sound_format.is_stereo)?;
        self.flush_bits()?;
        Ok(())
    }

    fn write_sound_info(&mut self, sound_info: &SoundInfo) -> Result<()> {
        let flags = match sound_info.event {
            SoundEvent::Event => 0b00_0000u8,
            SoundEvent::Start => 0b01_0000u8,
            SoundEvent::Stop => 0b10_0000u8,
        } | if sound_info.in_sample.is_some() {
            0b1
        } else {
            0
        } | if sound_info.out_sample.is_some() {
            0b10
        } else {
            0
        } | if sound_info.num_loops > 1 { 0b100 } else { 0 }
            | if sound_info.envelope.is_some() {
                0b1000
            } else {
                0
            };
        self.write_u8(flags)?;
        if let Some(n) = sound_info.in_sample {
            self.write_u32(n)?;
        }
        if let Some(n) = sound_info.out_sample {
            self.write_u32(n)?;
        }
        if sound_info.num_loops > 1 {
            self.write_u16(sound_info.num_loops)?;
        }
        if let Some(ref envelope) = sound_info.envelope {
            self.write_u8(envelope.len() as u8)?;
            for point in envelope {
                self.write_u32(point.sample)?;
                self.write_u16((point.left_volume * 32768f32) as u16)?;
                self.write_u16((point.right_volume * 32768f32) as u16)?;
            }
        }
        Ok(())
    }

    fn write_define_font_2(&mut self, font: &Font) -> Result<()> {
        let mut buf = Vec::new();
        {
            let num_glyphs = font.glyphs.len();

            // We must write the glyph shapes into a temporary buffer
            // so that we can calculate their offsets.
            let mut offsets = vec![];
            let mut has_wide_offsets = false;
            let has_wide_codes = !font.is_ansi;
            let mut shape_buf = Vec::new();
            {
                let mut shape_writer = Writer::new(&mut shape_buf, self.version);

                // ShapeTable
                shape_writer.num_fill_bits = 1;
                shape_writer.num_line_bits = 0;
                for glyph in &font.glyphs {
                    // Store offset for later.
                    let offset = num_glyphs * 4 + shape_writer.output.len();
                    offsets.push(offset);
                    if offset > 0xFFFF {
                        has_wide_offsets = true;
                    }

                    shape_writer.write_ubits(4, 1)?;
                    shape_writer.write_ubits(4, 0)?;

                    for shape_record in &glyph.shape_records {
                        shape_writer.write_shape_record(shape_record, 1)?;
                    }
                    // End shape record.
                    shape_writer.write_ubits(6, 0)?;
                    shape_writer.flush_bits()?;
                }
            }

            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(font.id)?;
            writer.write_u8(
                if font.layout.is_some() { 0b10000000 } else { 0 }
                    | if font.is_shift_jis { 0b1000000 } else { 0 }
                    | if font.is_small_text { 0b100000 } else { 0 }
                    | if font.is_ansi { 0b10000 } else { 0 }
                    | if has_wide_offsets { 0b1000 } else { 0 }
                    | if has_wide_codes { 0b100 } else { 0 }
                    | if font.is_italic { 0b10 } else { 0 }
                    | if font.is_bold { 0b1 } else { 0 },
            )?;
            writer.write_language(font.language)?;
            writer.write_u8(font.name.len() as u8)?;
            writer.output.write_all(font.name.as_bytes())?;
            writer.write_u16(num_glyphs as u16)?;

            // If there are no glyphs, then the following tables are omitted.
            if num_glyphs > 0 {
                // OffsetTable
                for offset in offsets {
                    if has_wide_offsets {
                        writer.write_u32(offset as u32)?;
                    } else {
                        writer.write_u16(offset as u16)?;
                    }
                }

                // CodeTableOffset
                let code_table_offset =
                    (num_glyphs + 1) * if has_wide_offsets { 4 } else { 2 } + shape_buf.len();
                if has_wide_offsets {
                    writer.write_u32(code_table_offset as u32)?;
                } else {
                    writer.write_u16(code_table_offset as u16)?;
                }

                writer.output.write_all(&shape_buf)?;

                // CodeTable
                for glyph in &font.glyphs {
                    if has_wide_codes {
                        writer.write_u16(glyph.code)?;
                    } else {
                        writer.write_u8(glyph.code as u8)?;
                    }
                }
            }

            if let Some(ref layout) = font.layout {
                writer.write_u16(layout.ascent)?;
                writer.write_u16(layout.descent)?;
                writer.write_i16(layout.leading)?;
                for glyph in &font.glyphs {
                    writer.write_i16(
                        glyph
                            .advance
                            .ok_or_else(|| Error::invalid_data("glyph.advance cannot be None"))?,
                    )?;
                }
                for glyph in &font.glyphs {
                    writer.write_rectangle(
                        glyph
                            .bounds
                            .as_ref()
                            .ok_or_else(|| Error::invalid_data("glyph.bounds cannot be None"))?,
                    )?;
                }
                writer.write_u16(layout.kerning.len() as u16)?;
                for kerning_record in &layout.kerning {
                    writer.write_kerning_record(kerning_record, has_wide_codes)?;
                }
            }
        }

        let tag_code = if font.version == 2 {
            TagCode::DefineFont2
        } else {
            TagCode::DefineFont3
        };
        self.write_tag_header(tag_code, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_font_4(&mut self, font: &Font4) -> Result<()> {
        let mut tag_len = 4 + font.name.len();
        if let Some(ref data) = font.data {
            tag_len += data.len()
        };
        self.write_tag_header(TagCode::DefineFont4, tag_len as u32)?;
        self.write_character_id(font.id)?;
        self.write_u8(
            if font.data.is_some() { 0b100 } else { 0 }
                | if font.is_italic { 0b10 } else { 0 }
                | if font.is_bold { 0b1 } else { 0 },
        )?;
        self.write_c_string(&font.name)?;
        if let Some(ref data) = font.data {
            self.output.write_all(data)?;
        }
        Ok(())
    }

    fn write_kerning_record(
        &mut self,
        kerning: &KerningRecord,
        has_wide_codes: bool,
    ) -> Result<()> {
        if has_wide_codes {
            self.write_u16(kerning.left_code)?;
            self.write_u16(kerning.right_code)?;
        } else {
            self.write_u8(kerning.left_code as u8)?;
            self.write_u8(kerning.right_code as u8)?;
        }
        self.write_i16(kerning.adjustment.get() as i16)?; // TODO(Herschel): Handle overflow
        Ok(())
    }

    fn write_define_text(&mut self, text: &Text) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(text.id)?;
            writer.write_rectangle(&text.bounds)?;
            writer.write_matrix(&text.matrix)?;
            let num_glyph_bits = text
                .records
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| count_ubits(g.index)))
                .max()
                .unwrap_or(0);
            let num_advance_bits = text
                .records
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| count_sbits(g.advance)))
                .max()
                .unwrap_or(0);
            writer.write_u8(num_glyph_bits)?;
            writer.write_u8(num_advance_bits)?;

            for record in &text.records {
                let flags = 0b10000000
                    | if record.font_id.is_some() { 0b1000 } else { 0 }
                    | if record.color.is_some() { 0b100 } else { 0 }
                    | if record.y_offset.is_some() { 0b10 } else { 0 }
                    | if record.x_offset.is_some() { 0b1 } else { 0 };
                writer.write_u8(flags)?;
                if let Some(id) = record.font_id {
                    writer.write_character_id(id)?;
                }
                if let Some(ref color) = record.color {
                    writer.write_rgb(color)?;
                }
                if let Some(x) = record.x_offset {
                    writer.write_i16(x.get() as i16)?; // TODO(Herschel): Handle overflow.
                }
                if let Some(y) = record.y_offset {
                    writer.write_i16(y.get() as i16)?;
                }
                if let Some(height) = record.height {
                    writer.write_u16(height.get() as u16)?;
                }
                writer.write_u8(record.glyphs.len() as u8)?;
                for glyph in &record.glyphs {
                    writer.write_ubits(num_glyph_bits, glyph.index)?;
                    writer.write_sbits(num_advance_bits, glyph.advance)?;
                }
            }
            writer.write_u8(0)?; // End of text records.
        }
        self.write_tag_header(TagCode::DefineText, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_edit_text(&mut self, edit_text: &EditText) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(edit_text.id)?;
            writer.write_rectangle(&edit_text.bounds)?;
            let flags = if edit_text.initial_text.is_some() {
                0b10000000
            } else {
                0
            } | if edit_text.is_word_wrap { 0b1000000 } else { 0 }
                | if edit_text.is_multiline { 0b100000 } else { 0 }
                | if edit_text.is_password { 0b10000 } else { 0 }
                | if edit_text.is_read_only { 0b1000 } else { 0 }
                | if edit_text.color.is_some() { 0b100 } else { 0 }
                | if edit_text.max_length.is_some() {
                    0b10
                } else {
                    0
                }
                | if edit_text.font_id.is_some() { 0b1 } else { 0 };
            let flags2 = if edit_text.font_class_name.is_some() {
                0b10000000
            } else {
                0
            } | if edit_text.is_auto_size { 0b1000000 } else { 0 }
                | if edit_text.layout.is_some() {
                    0b100000
                } else {
                    0
                }
                | if !edit_text.is_selectable { 0b10000 } else { 0 }
                | if edit_text.has_border { 0b1000 } else { 0 }
                | if edit_text.was_static { 0b100 } else { 0 }
                | if edit_text.is_html { 0b10 } else { 0 }
                | if !edit_text.is_device_font { 0b1 } else { 0 };

            writer.write_u8(flags)?;
            writer.write_u8(flags2)?;

            if let Some(font_id) = edit_text.font_id {
                writer.write_character_id(font_id)?;
            }

            // TODO(Herschel): Check SWF version.
            if let Some(ref class) = edit_text.font_class_name {
                writer.write_c_string(class)?;
            }

            // TODO(Herschel): Height only exists iff HasFontId, maybe for HasFontClass too?
            if let Some(height) = edit_text.height {
                writer.write_u16(height.get() as u16)?
            }

            if let Some(ref color) = edit_text.color {
                writer.write_rgba(color)?
            }

            if let Some(len) = edit_text.max_length {
                writer.write_u16(len)?;
            }

            if let Some(ref layout) = edit_text.layout {
                writer.write_u8(match layout.align {
                    TextAlign::Left => 0,
                    TextAlign::Right => 1,
                    TextAlign::Center => 2,
                    TextAlign::Justify => 3,
                })?;
                writer.write_u16(layout.left_margin.get() as u16)?; // TODO: Handle overflow
                writer.write_u16(layout.right_margin.get() as u16)?;
                writer.write_u16(layout.indent.get() as u16)?;
                writer.write_i16(layout.leading.get() as i16)?;
            }

            writer.write_c_string(&edit_text.variable_name)?;
            if let Some(ref text) = edit_text.initial_text {
                writer.write_c_string(text)?;
            }
        }

        self.write_tag_header(TagCode::DefineEditText, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_video_stream(&mut self, video: &DefineVideoStream) -> Result<()> {
        self.write_tag_header(TagCode::DefineVideoStream, 10)?;
        self.write_character_id(video.id)?;
        self.write_u16(video.num_frames)?;
        self.write_u16(video.width)?;
        self.write_u16(video.height)?;
        self.write_u8(
            match video.deblocking {
                VideoDeblocking::UseVideoPacketValue => 0b000_0,
                VideoDeblocking::None => 0b001_0,
                VideoDeblocking::Level1 => 0b010_0,
                VideoDeblocking::Level2 => 0b011_0,
                VideoDeblocking::Level3 => 0b100_0,
                VideoDeblocking::Level4 => 0b101_0,
            } | if video.is_smoothed { 0b1 } else { 0 },
        )?;
        self.write_u8(match video.codec {
            VideoCodec::H263 => 2,
            VideoCodec::ScreenVideo => 3,
            VideoCodec::VP6 => 4,
            VideoCodec::VP6WithAlpha => 5,
        })?;
        Ok(())
    }

    fn write_product_info(&mut self, product_info: &ProductInfo) -> Result<()> {
        self.write_tag_header(TagCode::ProductInfo, 26)?;
        self.write_u32(product_info.product_id)?;
        self.write_u32(product_info.edition)?;
        self.write_u8(product_info.major_version)?;
        self.write_u8(product_info.minor_version)?;
        self.write_u64(product_info.build_number)?;
        self.write_u64(product_info.compilation_date)?;
        Ok(())
    }

    fn write_debug_id(&mut self, debug_id: &DebugId) -> Result<()> {
        self.get_inner().write_all(debug_id)?;
        Ok(())
    }

    fn write_tag_header(&mut self, tag_code: TagCode, length: u32) -> Result<()> {
        self.write_tag_code_and_length(tag_code as u16, length)
    }

    fn write_tag_code_and_length(&mut self, tag_code: u16, length: u32) -> Result<()> {
        // TODO: Test for tag code/length overflow.
        let mut tag_code_and_length: u16 = tag_code << 6;
        if length < 0b111111 {
            tag_code_and_length |= length as u16;
            self.write_u16(tag_code_and_length)?;
        } else {
            tag_code_and_length |= 0b111111;
            self.write_u16(tag_code_and_length)?;
            self.write_u32(length)?;
        }
        Ok(())
    }

    fn write_tag_list(&mut self, tags: &[Tag]) -> Result<()> {
        // TODO: Better error handling. Can skip errored tags, unless EOF.
        for tag in tags {
            self.write_tag(tag)?;
        }
        // Implicit end tag.
        self.write_tag(&Tag::End)?;
        Ok(())
    }
}

fn count_ubits(mut n: u32) -> u8 {
    let mut num_bits = 0;
    while n > 0 {
        n >>= 1;
        num_bits += 1;
    }
    num_bits
}

fn count_sbits(n: i32) -> u8 {
    if n == 0 {
        0
    } else if n == -1 {
        1
    } else if n < 0 {
        count_ubits((!n) as u32) + 1
    } else {
        count_ubits(n as u32) + 1
    }
}

fn count_sbits_twips(n: Twips) -> u8 {
    let n = n.get();
    if n == 0 {
        0
    } else if n == -1 {
        1
    } else if n < 0 {
        count_ubits((!n) as u32) + 1
    } else {
        count_ubits(n as u32) + 1
    }
}

fn count_fbits(n: f32) -> u8 {
    count_sbits((n * 65536f32) as i32)
}

#[cfg(test)]
mod tests {
    use super::Writer;
    use super::*;
    use crate::test_data;

    fn new_swf() -> Swf {
        Swf {
            header: Header {
                version: 13,
                compression: Compression::Zlib,
                stage_size: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(640.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(480.0),
                },
                frame_rate: 60.0,
                num_frames: 1,
            },
            tags: vec![],
        }
    }

    #[test]
    fn write_swfs() {
        fn write_dummy_swf(compression: Compression) -> Result<()> {
            let mut buf = Vec::new();
            let mut swf = new_swf();
            swf.header.compression = compression;
            write_swf(&swf, &mut buf)?;
            Ok(())
        }
        assert!(
            write_dummy_swf(Compression::None).is_ok(),
            "Failed to write uncompressed SWF."
        );
        assert!(
            write_dummy_swf(Compression::Zlib).is_ok(),
            "Failed to write zlib SWF."
        );
        if cfg!(feature = "lzma") {
            assert!(
                write_dummy_swf(Compression::Lzma).is_ok(),
                "Failed to write LZMA SWF."
            );
        }
    }

    #[test]
    fn write_fixed8() {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_fixed8(0f32).unwrap();
            writer.write_fixed8(1f32).unwrap();
            writer.write_fixed8(6.5f32).unwrap();
            writer.write_fixed8(-20.75f32).unwrap();
        }
        assert_eq!(
            buf,
            [
                0b00000000, 0b00000000, 0b00000000, 0b00000001, 0b10000000, 0b00000110, 0b01000000,
                0b11101011
            ]
        );
    }

    #[test]
    fn write_encoded_u32() {
        fn write_to_buf(n: u32) -> Vec<u8> {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_encoded_u32(n).unwrap();
            }
            buf
        }

        assert_eq!(write_to_buf(0), [0]);
        assert_eq!(write_to_buf(2), [2]);
        assert_eq!(write_to_buf(129), [0b1_0000001, 0b0_0000001]);
        assert_eq!(
            write_to_buf(0b1100111_0000001_0000001),
            [0b1_0000001, 0b1_0000001, 0b0_1100111]
        );
        assert_eq!(
            write_to_buf(0b1111_0000000_0000000_0000000_0000000u32),
            [
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b0000_1111
            ]
        );
    }

    #[test]
    fn write_bit() {
        let bits = [
            false, true, false, true, false, true, false, true, false, false, true, false, false,
            true, false, true,
        ];
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            for b in &bits {
                writer.write_bit(*b).unwrap();
            }
        }
        assert_eq!(buf, [0b01010101, 0b00100101]);
    }

    #[test]
    fn write_ubits() {
        let num_bits = 2;
        let nums = [1, 1, 1, 1, 0, 2, 1, 1];
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            for n in &nums {
                writer.write_ubits(num_bits, *n).unwrap();
            }
            writer.flush_bits().unwrap();
        }
        assert_eq!(buf, [0b01010101, 0b00100101]);
    }

    #[test]
    fn write_sbits() {
        let num_bits = 2;
        let nums = [1, 1, 1, 1, 0, -2, 1, 1];
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            for n in &nums {
                writer.write_sbits(num_bits, *n).unwrap();
            }
            writer.flush_bits().unwrap();
        }
        assert_eq!(buf, [0b01010101, 0b00100101]);
    }

    #[test]
    fn write_fbits() {
        let num_bits = 18;
        let nums = [1f32, -1f32];
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            for n in &nums {
                writer.write_fbits(num_bits, *n).unwrap();
            }
            writer.flush_bits().unwrap();
        }
        assert_eq!(
            buf,
            [
                0b01_000000,
                0b00000000,
                0b00_11_0000,
                0b00000000,
                0b0000_0000
            ]
        );
    }

    #[test]
    fn count_ubits() {
        assert_eq!(super::count_ubits(0), 0u8);
        assert_eq!(super::count_ubits(1u32), 1);
        assert_eq!(super::count_ubits(2u32), 2);
        assert_eq!(super::count_ubits(0b_00111101_00000000u32), 14);
    }

    #[test]
    fn count_sbits() {
        assert_eq!(super::count_sbits(0), 0u8);
        assert_eq!(super::count_sbits(1), 2u8);
        assert_eq!(super::count_sbits(2), 3u8);
        assert_eq!(super::count_sbits(0b_00111101_00000000), 15u8);

        assert_eq!(super::count_sbits(-1), 1u8);
        assert_eq!(super::count_sbits(-2), 2u8);
        assert_eq!(super::count_sbits(-0b_00110101_01010101), 15u8);
    }

    #[test]
    fn write_c_string() {
        {
            let mut buf = Vec::new();
            {
                // TODO: What if I use a cursor instead of buf ?
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_c_string("Hello!").unwrap();
            }
            assert_eq!(buf, "Hello!\0".bytes().collect::<Vec<_>>());
        }

        {
            let mut buf = Vec::new();
            {
                // TODO: What if I use a cursor instead of buf ?
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_c_string("😀😂!🐼").unwrap();
            }
            assert_eq!(buf, "😀😂!🐼\0".bytes().collect::<Vec<_>>());
        }
    }

    #[test]
    fn write_rectangle_zero() {
        let rect: Rectangle = Default::default();
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_rectangle(&rect).unwrap();
            writer.flush_bits().unwrap();
        }
        assert_eq!(buf, [0]);
    }

    #[test]
    fn write_rectangle_signed() {
        let rect = Rectangle {
            x_min: Twips::from_pixels(-1.0),
            x_max: Twips::from_pixels(1.0),
            y_min: Twips::from_pixels(-1.0),
            y_max: Twips::from_pixels(1.0),
        };
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_rectangle(&rect).unwrap();
            writer.flush_bits().unwrap();
        }
        assert_eq!(buf, [0b_00110_101, 0b100_01010, 0b0_101100_0, 0b_10100_000]);
    }

    #[test]
    fn write_color() {
        {
            let color = Color {
                r: 1,
                g: 128,
                b: 255,
                a: 255,
            };
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_rgb(&color).unwrap();
            }
            assert_eq!(buf, [1, 128, 255]);
        }
        {
            let color = Color {
                r: 1,
                g: 2,
                b: 3,
                a: 11,
            };
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_rgba(&color).unwrap();
            }
            assert_eq!(buf, [1, 2, 3, 11]);
        }
    }

    #[test]
    fn write_matrix() {
        fn write_to_buf(m: &Matrix) -> Vec<u8> {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_matrix(m).unwrap();
                writer.flush_bits().unwrap();
            }
            buf
        }

        let m = Matrix::new();
        assert_eq!(write_to_buf(&m), [0]);
    }

    #[test]
    fn write_tags() {
        for (swf_version, tag, expected_tag_bytes) in test_data::tag_tests() {
            let mut written_tag_bytes = Vec::new();
            Writer::new(&mut written_tag_bytes, swf_version)
                .write_tag(&tag)
                .unwrap();
            if written_tag_bytes != expected_tag_bytes {
                panic!(
                    "Error reading tag.\nTag:\n{:?}\n\nWrote:\n{:?}\n\nExpected:\n{:?}",
                    tag, written_tag_bytes, expected_tag_bytes
                );
            }
        }
    }

    #[test]
    fn write_tag_to_buf_list() {
        {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_tag_list(&[]).unwrap();
            }
            assert_eq!(buf, [0, 0]);
        }
        {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_tag_list(&[Tag::ShowFrame]).unwrap();
            }
            assert_eq!(buf, [0b01_000000, 0b00000000, 0, 0]);
        }
        {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer
                    .write_tag_list(&[
                        Tag::Unknown {
                            tag_code: 512,
                            data: vec![0; 100],
                        },
                        Tag::ShowFrame,
                    ])
                    .unwrap();
            }
            let mut expected = vec![0b00_111111, 0b10000000, 100, 0, 0, 0];
            expected.extend_from_slice(&[0; 100]);
            expected.extend_from_slice(&[0b01_000000, 0b00000000, 0, 0]);
            assert_eq!(buf, expected);
        }
    }
}
