use crate::{
    error::{Error, Result},
    string::SwfStr,
    tag_code::TagCode,
    types::*,
};
use bitstream_io::BitWrite;
use byteorder::{LittleEndian, WriteBytesExt};
use std::cmp::max;
use std::io::{self, Write};

/// Writes an SWF file to an output stream.
/// # Example
/// ```
/// use swf::*;
///
/// let header = Header {
///     compression: Compression::Zlib,
///     version: 6,
///     stage_size: Rectangle { x_min: Twips::from_pixels(0.0), x_max: Twips::from_pixels(400.0), y_min: Twips::from_pixels(0.0), y_max: Twips::from_pixels(400.0) },
///     frame_rate: Fixed8::from_f32(60.0),
///     num_frames: 1,
/// };
/// let tags = [
///     Tag::SetBackgroundColor(SetBackgroundColor(Color { r: 255, g: 0, b: 0, a: 255 })),
///     Tag::ShowFrame,
/// ];
/// let output = Vec::new();
/// swf::write_swf(&header, &tags, output).unwrap();
/// ```
pub fn write_swf<W: Write>(header: &Header, tags: &[Tag<'_>], mut output: W) -> Result<()> {
    let signature = match header.compression {
        Compression::None => b"FWS",
        Compression::Zlib => b"CWS",
        Compression::Lzma => b"ZWS",
    };
    output.write_all(&signature[..])?;
    output.write_u8(header.version)?;

    // Write SWF body.
    let mut swf_body = Vec::new();
    {
        let mut writer = Writer::new(&mut swf_body, header.version);

        writer.write_rectangle(&header.stage_size)?;
        writer.write_fixed8(header.frame_rate)?;
        writer.write_u16(header.num_frames)?;

        // Write main timeline tag list.
        writer.write_tag_list(tags)?;
    }

    // Write SWF header.
    // Uncompressed SWF length.
    output.write_u32::<LittleEndian>(swf_body.len() as u32 + 8)?;

    // Compress SWF body.
    match header.compression {
        Compression::None => output.write_all(&swf_body)?,

        Compression::Zlib => write_zlib_swf(&mut output, &swf_body)?,

        Compression::Lzma => {
            write_lzma_swf(&mut output, &swf_body)?;
            // 5 bytes of garbage data?
            //output.write_all(&[0xFF, 0xB5, 0xE6, 0xF8, 0xCB])?;
        }
    }

    Ok(())
}

#[cfg(feature = "flate2")]
fn write_zlib_swf<W: Write>(mut output: W, swf_body: &[u8]) -> Result<()> {
    use flate2::write::ZlibEncoder;
    use flate2::Compression;
    let mut encoder = ZlibEncoder::new(&mut output, Compression::best());
    encoder.write_all(swf_body)?;
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
fn write_lzma_swf<W: Write>(mut output: W, mut swf_body: &[u8]) -> Result<()> {
    // Compress data using LZMA.
    let mut data = vec![];
    lzma_rs::lzma_compress(&mut swf_body, &mut data)?;

    // Flash uses a mangled LZMA header, so we have to massage it into the SWF format.
    // https://helpx.adobe.com/flash-player/kb/exception-thrown-you-decompress-lzma-compressed.html
    output.write_u32::<LittleEndian>(data.len() as u32 - 13)?; // Compressed length (- 13 to not include lzma header)
    output.write_all(&data[0..5])?; // LZMA properties
    output.write_all(&data[13..])?; // Data
    Ok(())
}

#[cfg(not(feature = "lzma"))]
fn write_lzma_swf<W: Write>(_output: W, _swf_body: &[u8]) -> Result<()> {
    Err(Error::unsupported(
        "Support for LZMA compressed SWFs is not enabled.",
    ))
}

pub trait SwfWriteExt {
    fn write_u8(&mut self, n: u8) -> io::Result<()>;
    fn write_u16(&mut self, n: u16) -> io::Result<()>;
    fn write_u32(&mut self, n: u32) -> io::Result<()>;
    fn write_u64(&mut self, n: u64) -> io::Result<()>;
    fn write_i8(&mut self, n: i8) -> io::Result<()>;
    fn write_i16(&mut self, n: i16) -> io::Result<()>;
    fn write_i32(&mut self, n: i32) -> io::Result<()>;
    fn write_f32(&mut self, n: f32) -> io::Result<()>;
    fn write_f64(&mut self, n: f64) -> io::Result<()>;
    fn write_string(&mut self, s: &'_ SwfStr) -> io::Result<()>;
}

pub struct BitWriter<W: Write> {
    bits: bitstream_io::BitWriter<W, bitstream_io::BigEndian>,
}

impl<W: Write> BitWriter<W> {
    #[inline]
    fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        self.bits.write_bit(bit)
    }

    #[inline]
    fn write_ubits(&mut self, num_bits: u32, n: u32) -> io::Result<()> {
        if num_bits > 0 {
            self.bits.write(num_bits, n)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn write_sbits(&mut self, num_bits: u32, n: i32) -> io::Result<()> {
        if num_bits > 0 {
            self.bits.write_signed(num_bits, n)
        } else {
            Ok(())
        }
    }

    #[inline]
    fn write_sbits_fixed8(&mut self, num_bits: u32, n: Fixed8) -> io::Result<()> {
        self.write_sbits(num_bits, n.get().into())
    }

    #[inline]
    fn write_sbits_twips(&mut self, num_bits: u32, twips: Twips) -> io::Result<()> {
        self.write_sbits(num_bits, twips.get())
    }

    #[inline]
    fn write_fbits(&mut self, num_bits: u32, n: Fixed16) -> io::Result<()> {
        self.write_sbits(num_bits, n.get())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.bits.byte_align()
    }

    #[inline]
    fn writer(&mut self) -> &mut W {
        let _ = self.bits.flush();
        self.bits.writer().unwrap()
    }
}

impl<W: Write> Drop for BitWriter<W> {
    #[inline]
    fn drop(&mut self) {
        let _ = self.flush();
        let _ = self.bits.flush();
    }
}

struct Writer<W: Write> {
    pub output: W,
    pub version: u8,
}

impl<W: Write> SwfWriteExt for Writer<W> {
    #[inline]
    fn write_u8(&mut self, n: u8) -> io::Result<()> {
        self.output.write_u8(n)
    }

    #[inline]
    fn write_u16(&mut self, n: u16) -> io::Result<()> {
        self.output.write_u16::<LittleEndian>(n)
    }

    #[inline]
    fn write_u32(&mut self, n: u32) -> io::Result<()> {
        self.output.write_u32::<LittleEndian>(n)
    }

    #[inline]
    fn write_u64(&mut self, n: u64) -> io::Result<()> {
        self.output.write_u64::<LittleEndian>(n)
    }

    #[inline]
    fn write_i8(&mut self, n: i8) -> io::Result<()> {
        self.output.write_i8(n)
    }

    #[inline]
    fn write_i16(&mut self, n: i16) -> io::Result<()> {
        self.output.write_i16::<LittleEndian>(n)
    }

    #[inline]
    fn write_i32(&mut self, n: i32) -> io::Result<()> {
        self.output.write_i32::<LittleEndian>(n)
    }

    #[inline]
    fn write_f32(&mut self, n: f32) -> io::Result<()> {
        self.output.write_f32::<LittleEndian>(n)
    }

    #[inline]
    fn write_f64(&mut self, n: f64) -> io::Result<()> {
        self.output.write_f64::<LittleEndian>(n)
    }

    #[inline]
    fn write_string(&mut self, s: &'_ SwfStr) -> io::Result<()> {
        self.output.write_all(s.as_bytes())?;
        self.write_u8(0)
    }
}

impl<W: Write> Writer<W> {
    fn new(output: W, version: u8) -> Self {
        Self { output, version }
    }

    #[inline]
    fn bits(&mut self) -> BitWriter<&mut W> {
        BitWriter {
            bits: bitstream_io::BitWriter::new(&mut self.output),
        }
    }

    //=========================================================================
    // Basic types
    //=========================================================================

    #[inline]
    fn write_fixed8(&mut self, n: Fixed8) -> io::Result<()> {
        self.write_i16(n.get())
    }

    #[inline]
    fn write_fixed16(&mut self, n: Fixed16) -> io::Result<()> {
        self.write_i32(n.get())
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

    fn write_rectangle(&mut self, rectangle: &Rectangle<Twips>) -> Result<()> {
        let num_bits = [
            rectangle.x_min,
            rectangle.x_max,
            rectangle.y_min,
            rectangle.y_max,
        ]
        .iter()
        .map(|x| count_sbits_twips(*x))
        .max()
        .unwrap();
        let mut bits = self.bits();
        bits.write_ubits(5, num_bits)?;
        bits.write_sbits_twips(num_bits, rectangle.x_min)?;
        bits.write_sbits_twips(num_bits, rectangle.x_max)?;
        bits.write_sbits_twips(num_bits, rectangle.y_min)?;
        bits.write_sbits_twips(num_bits, rectangle.y_max)?;
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
        let has_mult = !color_transform.r_multiply.is_one()
            || !color_transform.g_multiply.is_one()
            || !color_transform.b_multiply.is_one();
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
        let mut bits = self.bits();
        bits.write_bit(has_mult)?;
        bits.write_bit(has_add)?;
        let mut num_bits = if has_mult {
            multiply
                .iter()
                .map(|n| count_sbits(n.get().into()))
                .max()
                .unwrap()
        } else {
            0
        };
        if has_add {
            num_bits = max(
                num_bits,
                add.iter().map(|n| count_sbits((*n).into())).max().unwrap(),
            );
        }
        bits.write_ubits(4, num_bits)?;
        if has_mult {
            bits.write_sbits_fixed8(num_bits, color_transform.r_multiply)?;
            bits.write_sbits_fixed8(num_bits, color_transform.g_multiply)?;
            bits.write_sbits_fixed8(num_bits, color_transform.b_multiply)?;
        }
        if has_add {
            bits.write_sbits(num_bits, color_transform.r_add.into())?;
            bits.write_sbits(num_bits, color_transform.g_add.into())?;
            bits.write_sbits(num_bits, color_transform.b_add.into())?;
        }
        Ok(())
    }

    fn write_color_transform(&mut self, color_transform: &ColorTransform) -> Result<()> {
        let has_mult = !color_transform.r_multiply.is_one()
            || !color_transform.g_multiply.is_one()
            || !color_transform.b_multiply.is_one()
            || !color_transform.a_multiply.is_one();
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
        let mut bits = self.bits();
        bits.write_bit(has_add)?;
        bits.write_bit(has_mult)?;
        let mut num_bits = if has_mult {
            multiply
                .iter()
                .map(|n| count_sbits(n.get().into()))
                .max()
                .unwrap()
        } else {
            0
        };
        if has_add {
            num_bits = max(
                num_bits,
                add.iter().map(|n| count_sbits((*n).into())).max().unwrap(),
            );
        }
        bits.write_ubits(4, num_bits)?;
        if has_mult {
            bits.write_sbits_fixed8(num_bits, color_transform.r_multiply)?;
            bits.write_sbits_fixed8(num_bits, color_transform.g_multiply)?;
            bits.write_sbits_fixed8(num_bits, color_transform.b_multiply)?;
            bits.write_sbits_fixed8(num_bits, color_transform.a_multiply)?;
        }
        if has_add {
            bits.write_sbits(num_bits, color_transform.r_add.into())?;
            bits.write_sbits(num_bits, color_transform.g_add.into())?;
            bits.write_sbits(num_bits, color_transform.b_add.into())?;
            bits.write_sbits(num_bits, color_transform.a_add.into())?;
        }
        Ok(())
    }

    fn write_matrix(&mut self, m: &Matrix) -> Result<()> {
        let mut bits = self.bits();
        // Scale
        let has_scale = m.a != Fixed16::ONE || m.d != Fixed16::ONE;
        bits.write_bit(has_scale)?;
        if has_scale {
            let num_bits = max(count_fbits(m.a), count_fbits(m.d));
            bits.write_ubits(5, num_bits)?;
            bits.write_fbits(num_bits, m.a)?;
            bits.write_fbits(num_bits, m.d)?;
        }
        // Rotate/Skew
        let has_rotate_skew = m.b != Fixed16::ZERO || m.c != Fixed16::ZERO;
        bits.write_bit(has_rotate_skew)?;
        if has_rotate_skew {
            let num_bits = max(count_fbits(m.b), count_fbits(m.c));
            bits.write_ubits(5, num_bits)?;
            bits.write_fbits(num_bits, m.b)?;
            bits.write_fbits(num_bits, m.c)?;
        }
        // Translate (always written)
        let num_bits = max(count_sbits_twips(m.tx), count_sbits_twips(m.ty));
        bits.write_ubits(5, num_bits)?;
        bits.write_sbits_twips(num_bits, m.tx)?;
        bits.write_sbits_twips(num_bits, m.ty)?;
        Ok(())
    }

    fn write_language(&mut self, language: Language) -> Result<()> {
        self.write_u8(language as u8)?;
        Ok(())
    }

    fn write_blend_mode(&mut self, blend_mode: BlendMode) -> Result<()> {
        self.write_u8(blend_mode as u8)?;
        Ok(())
    }

    //=========================================================================
    // Tag types
    //=========================================================================

    fn write_tag(&mut self, tag: &Tag) -> Result<()> {
        match tag {
            Tag::CsmTextSettings(tag) => self.write_csm_text_settings(tag),
            Tag::DebugId(tag) => self.write_debug_id(tag),
            Tag::DefineBinaryData(tag) => self.write_define_binary_data(tag),
            Tag::DefineBits(tag) => self.write_define_bits(tag),
            Tag::DefineBitsJpeg2(tag) => self.write_define_bits_jpeg_2(tag),
            Tag::DefineBitsJpeg3(tag) => self.write_define_bits_jpeg_3(tag),
            Tag::DefineBitsLossless(tag) => self.write_define_bits_lossless(tag),
            Tag::DefineButton(tag) => self.write_define_button(tag),
            Tag::DefineButton2(tag) => self.write_define_button_2(tag),
            Tag::DefineButtonColorTransform(tag) => self.write_define_button_color_transform(tag),
            Tag::DefineButtonSound(tag) => self.write_define_button_sound(tag),
            Tag::DefineEditText(tag) => self.write_define_edit_text(tag),
            Tag::DefineFont(tag) => self.write_define_font(tag),
            Tag::DefineFont2(tag) => self.write_define_font_2(tag),
            Tag::DefineFont4(tag) => self.write_define_font_4(tag),
            Tag::DefineFontAlignZones(tag) => self.write_define_font_align_zones(tag),
            Tag::DefineFontInfo(tag) => self.write_define_font_info(tag),
            Tag::DefineFontName(tag) => self.write_define_font_name(tag),
            Tag::DefineMorphShape(tag) => self.write_define_morph_shape(tag),
            Tag::DefineScalingGrid(tag) => self.write_define_scaling_grid(tag),
            Tag::DefineSceneAndFrameLabelData(tag) => {
                self.write_define_scene_and_frame_label_data(tag)
            }
            Tag::DefineShape(tag) => self.write_define_shape(tag),
            Tag::DefineSound(tag) => self.write_define_sound(tag),
            Tag::DefineSprite(tag) => self.write_define_sprite(tag),
            Tag::DefineText(tag) => self.write_define_text(tag),
            Tag::DefineVideoStream(tag) => self.write_define_video_stream(tag),
            Tag::DoAbc(tag) => self.write_do_abc(tag),
            Tag::DoAbc2(tag) => self.write_do_abc_2(tag),
            Tag::DoAction(tag) => self.write_do_action(tag),
            Tag::DoInitAction(tag) => self.write_do_init_action(tag),
            Tag::EnableDebugger(tag) => self.write_enable_debugger(tag),
            Tag::EnableTelemetry(tag) => self.write_enable_telemetry(tag),
            Tag::End => self.write_tag_header(TagCode::End, 0),
            Tag::ExportAssets(tag) => self.write_export_assets(tag),
            Tag::FileAttributes(tag) => self.write_file_attributes(tag),
            Tag::FrameLabel(tag) => self.write_frame_label(tag),
            Tag::ImportAssets(tag) => self.write_import_assets(tag),
            Tag::JpegTables(tag) => self.write_jpeg_tables(tag),
            Tag::Metadata(tag) => self.write_metadata(tag),
            Tag::NameCharacter(tag) => self.write_name_character(tag),
            Tag::PlaceObject(tag) => self.write_place_object(tag),
            Tag::ProductInfo(tag) => self.write_product_info(tag),
            Tag::Protect(tag) => self.write_protect(tag),
            Tag::RemoveObject(tag) => self.write_remove_object(tag),
            Tag::ScriptLimits(tag) => self.write_script_limits(tag),
            Tag::SetBackgroundColor(tag) => self.write_set_background_color(tag),
            Tag::SetTabIndex(tag) => self.write_set_tab_index(tag),
            Tag::ShowFrame => self.write_tag_header(TagCode::ShowFrame, 0),
            Tag::SoundStreamBlock(tag) => self.write_sound_stream_block(tag),
            Tag::SoundStreamHead(tag) => self.write_sound_stream_head(tag, 1),
            Tag::SoundStreamHead2(tag) => self.write_sound_stream_head(tag, 2),
            Tag::StartSound(tag) => self.write_start_sound(tag),
            Tag::StartSound2(tag) => self.write_start_sound_2(tag),
            Tag::SymbolClass(tag) => self.write_symbol_class(tag),
            Tag::Unknown(tag) => self.write_unknown(tag),
            Tag::VideoFrame(tag) => self.write_video_frame(tag),
        }
    }

    fn write_tag_header(&mut self, tag_code: TagCode, length: u32) -> Result<()> {
        self.write_tag_code_and_length(tag_code as u16, length)
    }

    fn write_tag_code_and_length(&mut self, tag_code: u16, length: u32) -> Result<()> {
        // TODO: Test for tag code/length overflow.
        let mut tag_code_and_length = tag_code << 6;
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

    fn write_csm_text_settings(&mut self, tag: &CsmTextSettings) -> Result<()> {
        self.write_tag_header(TagCode::CsmTextSettings, 12)?;
        self.write_character_id(tag.id)?;
        self.write_u8(
            if tag.use_advanced_rendering {
                0b0100_0000
            } else {
                0
            } | ((tag.grid_fit as u8) << 3),
        )?;
        self.write_f32(tag.thickness)?;
        self.write_f32(tag.sharpness)?;
        self.write_u8(0)?; // Reserved (0).
        Ok(())
    }

    fn write_debug_id(&mut self, tag: &DebugId) -> Result<()> {
        self.write_tag_header(TagCode::DebugId, tag.uuid.len() as u32)?;
        self.output.write_all(&tag.uuid)?;
        Ok(())
    }

    fn write_define_binary_data(&mut self, tag: &DefineBinaryData) -> Result<()> {
        self.write_tag_header(TagCode::DefineBinaryData, tag.data.len() as u32 + 6)?;
        self.write_u16(tag.id)?;
        self.write_u32(0)?; // Reserved
        self.output.write_all(tag.data)?;
        Ok(())
    }

    fn write_define_bits(&mut self, tag: &DefineBits) -> Result<()> {
        self.write_tag_header(TagCode::DefineBits, tag.jpeg_data.len() as u32 + 2)?;
        self.write_u16(tag.id)?;
        self.output.write_all(tag.jpeg_data)?;
        Ok(())
    }

    fn write_define_bits_jpeg_2(&mut self, tag: &DefineBitsJpeg2) -> Result<()> {
        self.write_tag_header(TagCode::DefineBitsJpeg2, tag.jpeg_data.len() as u32 + 2)?;
        self.write_u16(tag.id)?;
        self.output.write_all(tag.jpeg_data)?;
        Ok(())
    }

    fn write_define_bits_jpeg_3(&mut self, tag: &DefineBitsJpeg3) -> Result<()> {
        self.write_tag_header(
            TagCode::DefineBitsJpeg3,
            (tag.data.len() + tag.alpha_data.len() + 6) as u32,
        )?;
        self.write_u16(tag.id)?;
        if tag.version >= 4 {
            self.write_fixed8(tag.deblocking)?;
        }
        // TODO(Herschel): Verify deblocking parameter is zero in version 3.
        self.write_u32(tag.data.len() as u32)?;
        self.output.write_all(tag.data)?;
        self.output.write_all(tag.alpha_data)?;
        Ok(())
    }

    fn write_define_bits_lossless(&mut self, tag: &DefineBitsLossless) -> Result<()> {
        let mut length = 7 + tag.data.len();
        if let BitmapFormat::ColorMap8 { .. } = tag.format {
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
            BitmapFormat::ColorMap8 { .. } => 3,
            BitmapFormat::Rgb15 => 4,
            BitmapFormat::Rgb32 => 5,
        };
        self.write_u8(format_id)?;
        self.write_u16(tag.width)?;
        self.write_u16(tag.height)?;
        if let BitmapFormat::ColorMap8 { num_colors } = tag.format {
            self.write_u8(num_colors)?;
        }
        self.output.write_all(tag.data)?;
        Ok(())
    }

    fn write_define_button(&mut self, tag: &Button) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(tag.id)?;
            for record in &tag.records {
                writer.write_button_record(record, 1)?;
            }
            writer.write_u8(0)?; // End button records
                                 // TODO: Assert we have some action.
            writer.output.write_all(tag.actions[0].action_data)?;
        }
        self.write_tag_header(TagCode::DefineButton, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_button_2(&mut self, tag: &Button) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(tag.id)?;
            let flags = if tag.is_track_as_menu { 1 } else { 0 };
            writer.write_u8(flags)?;

            let mut record_data = Vec::new();
            {
                let mut writer_2 = Writer::new(&mut record_data, self.version);
                for record in &tag.records {
                    writer_2.write_button_record(record, 2)?;
                }
                writer_2.write_u8(0)?; // End button records
            }
            writer.write_u16(record_data.len() as u16 + 2)?;
            writer.output.write_all(&record_data)?;

            let mut iter = tag.actions.iter().peekable();
            while let Some(action) = iter.next() {
                if iter.peek().is_some() {
                    let length = action.action_data.len() as u16 + 4;
                    writer.write_u16(length)?;
                } else {
                    writer.write_u16(0)?;
                }
                let mut flags = action.conditions.bits();
                if action.conditions.contains(ButtonActionCondition::KEY_PRESS) {
                    if let Some(key_code) = action.key_code {
                        flags |= (key_code as u16) << 9;
                    }
                }
                writer.write_u16(flags)?;
                writer.output.write_all(action.action_data)?;
            }
        }
        self.write_tag_header(TagCode::DefineButton2, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_button_color_transform(&mut self, tag: &ButtonColorTransform) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(tag.id)?;
            for color_transform in &tag.color_transforms {
                writer.write_color_transform_no_alpha(color_transform)?;
            }
        }
        self.write_tag_header(TagCode::DefineButtonCxform, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_button_sound(&mut self, tag: &ButtonSounds) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(tag.id)?;
            if let Some(ref sound) = tag.over_to_up_sound {
                writer.write_u16(sound.0)?;
                writer.write_sound_info(&sound.1)?;
            } else {
                writer.write_u16(0)?
            };
            if let Some(ref sound) = tag.up_to_over_sound {
                writer.write_u16(sound.0)?;
                writer.write_sound_info(&sound.1)?;
            } else {
                writer.write_u16(0)?
            };
            if let Some(ref sound) = tag.over_to_down_sound {
                writer.write_u16(sound.0)?;
                writer.write_sound_info(&sound.1)?;
            } else {
                writer.write_u16(0)?
            };
            if let Some(ref sound) = tag.down_to_over_sound {
                writer.write_u16(sound.0)?;
                writer.write_sound_info(&sound.1)?;
            } else {
                writer.write_u16(0)?
            };
        }
        self.write_tag_header(TagCode::DefineButtonSound, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_edit_text(&mut self, tag: &EditText) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(tag.id)?;
            writer.write_rectangle(&tag.bounds)?;
            writer.write_u16(tag.flags.bits())?;

            if let Some(font_id) = tag.font_id() {
                writer.write_character_id(font_id)?;
            }

            // TODO(Herschel): Check SWF version.
            if let Some(class) = tag.font_class() {
                writer.write_string(class)?;
            }

            if let Some(height) = tag.height() {
                writer.write_u16(height.get() as u16)?
            }

            if let Some(color) = tag.color() {
                writer.write_rgba(color)?
            }

            if let Some(len) = tag.max_length() {
                writer.write_u16(len)?;
            }

            if let Some(layout) = tag.layout() {
                writer.write_u8(layout.align as u8)?;
                writer.write_u16(layout.left_margin.get() as u16)?; // TODO: Handle overflow
                writer.write_u16(layout.right_margin.get() as u16)?;
                writer.write_u16(layout.indent.get() as u16)?;
                writer.write_i16(layout.leading.get() as i16)?;
            }

            writer.write_string(tag.variable_name)?;

            if let Some(text) = tag.initial_text() {
                writer.write_string(text)?;
            }
        }

        self.write_tag_header(TagCode::DefineEditText, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_font(&mut self, tag: &FontV1) -> Result<()> {
        let num_glyphs = tag.glyphs.len();
        let mut offsets = Vec::with_capacity(num_glyphs);
        let mut buf = vec![];
        {
            let mut writer = Writer::new(&mut buf, self.version);
            for glyph in &tag.glyphs {
                let offset = num_glyphs * 2 + writer.output.len();
                offsets.push(offset as u16);

                // Bit length for fill and line indices.
                // TODO: This theoretically could be >1?
                let mut shape_context = ShapeContext {
                    swf_version: self.version,
                    shape_version: 1,
                    num_fill_bits: 1,
                    num_line_bits: 0,
                };
                writer.write_u8(0b0001_0000)?;
                let mut bits = writer.bits();

                for shape_record in glyph {
                    Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
                }
                // End shape record.
                bits.write_ubits(6, 0)?;
            }
        }

        let tag_len = (2 + 2 * tag.glyphs.len() + buf.len()) as u32;
        self.write_tag_header(TagCode::DefineFont, tag_len)?;
        self.write_u16(tag.id)?;
        for offset in offsets {
            self.write_u16(offset)?;
        }
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_font_2(&mut self, tag: &Font) -> Result<()> {
        let mut buf = Vec::new();
        {
            let num_glyphs = tag.glyphs.len();

            // We must write the glyph shapes into a temporary buffer
            // so that we can calculate their offsets.
            let mut offsets = Vec::with_capacity(num_glyphs);
            let mut has_wide_offsets = false;
            let has_wide_codes = !tag.flags.contains(FontFlag::IS_ANSI);
            let mut shape_buf = Vec::new();
            {
                let mut shape_writer = Writer::new(&mut shape_buf, self.version);

                // ShapeTable
                let mut shape_context = ShapeContext {
                    swf_version: self.version,
                    shape_version: 1,
                    num_fill_bits: 1,
                    num_line_bits: 0,
                };
                for glyph in &tag.glyphs {
                    // Store offset for later.
                    let offset = num_glyphs * 4 + shape_writer.output.len();
                    offsets.push(offset);
                    if offset > 0xFFFF {
                        has_wide_offsets = true;
                    }

                    shape_writer.write_u8(0b0001_0000)?;
                    let mut bits = shape_writer.bits();
                    for shape_record in &glyph.shape_records {
                        Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
                    }
                    // End shape record.
                    bits.write_ubits(6, 0)?;
                }
            }

            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(tag.id)?;
            writer.write_u8(tag.flags.bits())?;
            writer.write_language(tag.language)?;
            writer.write_u8(tag.name.len() as u8)?;
            writer.output.write_all(tag.name.as_bytes())?;
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
                for glyph in &tag.glyphs {
                    if has_wide_codes {
                        writer.write_u16(glyph.code)?;
                    } else {
                        writer.write_u8(glyph.code as u8)?;
                    }
                }
            }

            if let Some(ref layout) = tag.layout {
                writer.write_u16(layout.ascent)?;
                writer.write_u16(layout.descent)?;
                writer.write_i16(layout.leading)?;
                for glyph in &tag.glyphs {
                    writer.write_i16(glyph.advance)?;
                }
                for glyph in &tag.glyphs {
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

        let tag_code = if tag.version == 2 {
            TagCode::DefineFont2
        } else {
            TagCode::DefineFont3
        };
        self.write_tag_header(tag_code, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_font_4(&mut self, tag: &Font4) -> Result<()> {
        let mut tag_len = 4 + tag.name.len();
        if let Some(data) = tag.data {
            tag_len += data.len()
        };
        self.write_tag_header(TagCode::DefineFont4, tag_len as u32)?;
        self.write_character_id(tag.id)?;
        self.write_u8(
            if tag.data.is_some() { 0b100 } else { 0 }
                | if tag.is_italic { 0b10 } else { 0 }
                | if tag.is_bold { 0b1 } else { 0 },
        )?;
        self.write_string(tag.name)?;
        if let Some(data) = tag.data {
            self.output.write_all(data)?;
        }
        Ok(())
    }

    fn write_define_font_align_zones(&mut self, tag: &DefineFontAlignZones) -> Result<()> {
        self.write_tag_header(
            TagCode::DefineFontAlignZones,
            3 + 10 * tag.zones.len() as u32,
        )?;
        self.write_character_id(tag.id)?;
        self.write_u8((tag.thickness as u8) << 6)?;
        for zone in &tag.zones {
            self.write_u8(2)?; // Always 2 dimensions.
            self.write_i16(zone.left)?;
            self.write_i16(zone.width)?;
            self.write_i16(zone.bottom)?;
            self.write_i16(zone.height)?;
            self.write_u8(0b0000_0011)?; // Always 2 dimensions.
        }
        Ok(())
    }

    fn write_define_font_name(&mut self, tag: &DefineFontName) -> Result<()> {
        let len = tag.name.len() + tag.copyright_info.len() + 4;
        self.write_tag_header(TagCode::DefineFontName, len as u32)?;
        self.write_character_id(tag.id)?;
        self.write_string(tag.name)?;
        self.write_string(tag.copyright_info)?;
        Ok(())
    }

    fn write_define_font_info(&mut self, tag: &FontInfo) -> Result<()> {
        let use_wide_codes = self.version >= 6 || tag.version >= 2;

        let len = tag.name.len()
            + if use_wide_codes { 2 } else { 1 } * tag.code_table.len()
            + if tag.version >= 2 { 1 } else { 0 }
            + 4;

        let tag_id = if tag.version == 1 {
            TagCode::DefineFontInfo
        } else {
            TagCode::DefineFontInfo2
        };
        self.write_tag_header(tag_id, len as u32)?;
        self.write_u16(tag.id)?;

        // SWF19 has ANSI and Shift-JIS backwards?
        self.write_u8(tag.name.len() as u8)?;
        self.output.write_all(tag.name.as_bytes())?;

        let mut flags = tag.flags;
        flags.set(FontInfoFlag::HAS_WIDE_CODES, use_wide_codes);
        self.write_u8(flags.bits())?;

        // TODO(Herschel): Assert language is unknown for v1.
        if tag.version >= 2 {
            self.write_language(tag.language)?;
        }
        for &code in &tag.code_table {
            if use_wide_codes {
                self.write_u16(code)?;
            } else {
                self.write_u8(code as u8)?;
            }
        }
        Ok(())
    }

    fn write_define_morph_shape(&mut self, tag: &DefineMorphShape) -> Result<()> {
        if tag.start.fill_styles.len() != tag.end.fill_styles.len()
            || tag.start.line_styles.len() != tag.end.line_styles.len()
        {
            return Err(Error::invalid_data(
                "Start and end state of a morph shape must have the same number of styles.",
            ));
        }

        let num_fill_styles = tag.start.fill_styles.len();
        let num_line_styles = tag.start.line_styles.len();
        let num_fill_bits = count_ubits(num_fill_styles as u32) as u8;
        let num_line_bits = count_ubits(num_line_styles as u32) as u8;

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
            for (start, end) in tag.start.fill_styles.iter().zip(tag.end.fill_styles.iter()) {
                writer.write_morph_fill_style(start, end)?;
            }

            if num_line_styles >= 0xff {
                writer.write_u8(0xff)?;
                writer.write_u16(num_line_styles as u16)?;
            } else {
                writer.write_u8(num_line_styles as u8)?;
            }
            for (start, end) in tag.start.line_styles.iter().zip(tag.end.line_styles.iter()) {
                writer.write_morph_line_style(start, end, tag.version)?;
            }

            // TODO(Herschel): Make fn write_shape.
            writer.write_u8((num_fill_bits << 4) | (num_line_bits & 0b1111))?;

            let mut shape_context = ShapeContext {
                swf_version: self.version,
                shape_version: 1,
                num_fill_bits,
                num_line_bits,
            };
            let mut bits = writer.bits();
            for shape_record in &tag.start.shape {
                Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
            }
            // End shape record.
            bits.write_ubits(6, 0)?;
        }

        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(tag.id)?;
            writer.write_rectangle(&tag.start.shape_bounds)?;
            writer.write_rectangle(&tag.end.shape_bounds)?;
            if tag.version >= 2 {
                writer.write_rectangle(&tag.start.edge_bounds)?;
                writer.write_rectangle(&tag.end.edge_bounds)?;
                writer.write_u8(tag.flags.bits())?;
            }

            // Offset to EndEdges.
            writer.write_u32(start_buf.len() as u32)?;

            writer.output.write_all(&start_buf)?;

            // EndEdges.
            writer.write_u8(0)?; // NumFillBits and NumLineBits are written as 0 for the end shape.
            let mut shape_context = ShapeContext {
                swf_version: self.version,
                shape_version: 1,
                num_fill_bits,
                num_line_bits,
            };
            let mut bits = writer.bits();
            for shape_record in &tag.end.shape {
                Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
            }
            // End shape record.
            bits.write_ubits(6, 0)?;
        }

        let tag_code = if tag.version == 1 {
            TagCode::DefineMorphShape
        } else {
            TagCode::DefineMorphShape2
        };
        self.write_tag_header(tag_code, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_scaling_grid(&mut self, tag: &DefineScalingGrid) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(tag.id)?;
            writer.write_rectangle(&tag.splitter_rect)?;
        }
        self.write_tag_header(TagCode::DefineScalingGrid, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_scene_and_frame_label_data(
        &mut self,
        tag: &DefineSceneAndFrameLabelData,
    ) -> Result<()> {
        let mut buf = Vec::with_capacity((tag.scenes.len() + tag.frame_labels.len()) * 4);
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_encoded_u32(tag.scenes.len() as u32)?;
            for scene in &tag.scenes {
                writer.write_encoded_u32(scene.frame_num)?;
                writer.write_string(scene.label)?;
            }
            writer.write_encoded_u32(tag.frame_labels.len() as u32)?;
            for frame_label in &tag.frame_labels {
                writer.write_encoded_u32(frame_label.frame_num)?;
                writer.write_string(frame_label.label)?;
            }
        }
        self.write_tag_header(TagCode::DefineSceneAndFrameLabelData, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_shape(&mut self, tag: &Shape) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(tag.id)?;
            writer.write_rectangle(&tag.shape_bounds)?;
            if tag.version >= 4 {
                writer.write_rectangle(&tag.edge_bounds)?;
                writer.write_u8(tag.flags.bits())?;
            }

            let (num_fill_bits, num_line_bits) =
                writer.write_shape_styles(&tag.styles, tag.version)?;
            let mut shape_context = ShapeContext {
                swf_version: self.version,
                shape_version: tag.version,
                num_fill_bits,
                num_line_bits,
            };
            let mut bits = writer.bits();
            for shape_record in &tag.shape {
                Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
            }
            // End shape record.
            bits.write_ubits(6, 0)?;
        }

        let tag_code = match tag.version {
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

    fn write_define_sound(&mut self, tag: &Sound) -> Result<()> {
        self.write_tag_header(TagCode::DefineSound, 7 + tag.data.len() as u32)?;
        self.write_u16(tag.id)?;
        self.write_sound_format(&tag.format)?;
        self.write_u32(tag.num_samples)?;
        self.output.write_all(tag.data)?;
        Ok(())
    }

    fn write_define_sprite(&mut self, tag: &Sprite) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(tag.id)?;
            writer.write_u16(tag.num_frames)?;
            writer.write_tag_list(&tag.tags)?;
        };
        self.write_tag_header(TagCode::DefineSprite, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_text(&mut self, tag: &Text) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(tag.id)?;
            writer.write_rectangle(&tag.bounds)?;
            writer.write_matrix(&tag.matrix)?;
            let num_glyph_bits = tag
                .records
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| count_ubits(g.index)))
                .max()
                .unwrap_or(0);
            let num_advance_bits = tag
                .records
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| count_sbits(g.advance)))
                .max()
                .unwrap_or(0);
            writer.write_u8(num_glyph_bits as u8)?;
            writer.write_u8(num_advance_bits as u8)?;

            for record in &tag.records {
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
                let mut bits = writer.bits();
                for glyph in &record.glyphs {
                    bits.write_ubits(num_glyph_bits, glyph.index)?;
                    bits.write_sbits(num_advance_bits, glyph.advance)?;
                }
            }
            writer.write_u8(0)?; // End of text records.
        }
        self.write_tag_header(TagCode::DefineText, buf.len() as u32)?;
        self.output.write_all(&buf)?;
        Ok(())
    }

    fn write_define_video_stream(&mut self, tag: &DefineVideoStream) -> Result<()> {
        self.write_tag_header(TagCode::DefineVideoStream, 10)?;
        self.write_character_id(tag.id)?;
        self.write_u16(tag.num_frames)?;
        self.write_u16(tag.width)?;
        self.write_u16(tag.height)?;
        self.write_u8(((tag.deblocking as u8) << 1) | if tag.is_smoothed { 0b1 } else { 0 })?;
        self.write_u8(tag.codec as u8)?;
        Ok(())
    }

    fn write_do_abc(&mut self, tag: &DoAbc) -> Result<()> {
        self.write_tag_header(TagCode::DoAbc, tag.data.len() as u32)?;
        self.output.write_all(tag.data)?;
        Ok(())
    }

    fn write_do_abc_2(&mut self, tag: &DoAbc2) -> Result<()> {
        let len = tag.data.len() + tag.name.len() + 5;
        self.write_tag_header(TagCode::DoAbc2, len as u32)?;
        self.write_u32(tag.flags.bits())?;
        self.write_string(tag.name)?;
        self.output.write_all(tag.data)?;
        Ok(())
    }

    fn write_do_action(&mut self, tag: &DoAction) -> Result<()> {
        self.write_tag_header(TagCode::DoAction, tag.action_data.len() as u32)?;
        self.output.write_all(tag.action_data)?;
        Ok(())
    }

    fn write_do_init_action(&mut self, tag: &DoInitAction) -> Result<()> {
        self.write_tag_header(TagCode::DoInitAction, tag.action_data.len() as u32 + 2)?;
        self.write_u16(tag.id)?;
        self.output.write_all(tag.action_data)?;
        Ok(())
    }

    fn write_enable_debugger(&mut self, tag: &EnableDebugger) -> Result<()> {
        let len = tag.password_hash.len() as u32 + 1;
        if self.version >= 6 {
            // SWF v6+ uses EnableDebugger2 tag.
            self.write_tag_header(TagCode::EnableDebugger2, len + 2)?;
            self.write_u16(0)?; // Reserved
        } else {
            self.write_tag_header(TagCode::EnableDebugger, len)?;
        }

        self.write_string(tag.password_hash)?;
        Ok(())
    }

    fn write_enable_telemetry(&mut self, tag: &EnableTelemetry) -> Result<()> {
        if !tag.password_hash.is_empty() {
            self.write_tag_header(TagCode::EnableTelemetry, 34)?;
            self.write_u16(0)?;
            self.output.write_all(&tag.password_hash[0..32])?;
        } else {
            self.write_tag_header(TagCode::EnableTelemetry, 2)?;
            self.write_u16(0)?;
        }
        Ok(())
    }

    fn write_export_assets(&mut self, tag: &ExportAssets) -> Result<()> {
        let len =
            tag.0.iter().map(|e| e.name.len() as u32 + 1).sum::<u32>() + tag.0.len() as u32 * 2 + 2;
        self.write_tag_header(TagCode::ExportAssets, len)?;
        self.write_u16(tag.0.len() as u16)?;
        for asset in &tag.0 {
            self.write_u16(asset.id)?;
            self.write_string(asset.name)?;
        }
        Ok(())
    }

    fn write_file_attributes(&mut self, tag: &FileAttributes) -> Result<()> {
        self.write_tag_header(TagCode::FileAttributes, 4)?;
        self.write_u32(tag.bits() as u32)?;
        Ok(())
    }

    fn write_frame_label(&mut self, tag: &FrameLabel) -> Result<()> {
        // TODO: Assert proper version
        let is_anchor = tag.is_anchor && self.version >= 6;
        let length = tag.label.len() as u32 + if is_anchor { 2 } else { 1 };
        self.write_tag_header(TagCode::FrameLabel, length)?;
        self.write_string(tag.label)?;
        if is_anchor {
            self.write_u8(1)?;
        }
        Ok(())
    }

    fn write_import_assets(&mut self, tag: &ImportAssets) -> Result<()> {
        let len = tag
            .imports
            .iter()
            .map(|e| e.name.len() as u32 + 3)
            .sum::<u32>()
            + tag.url.len() as u32
            + 1
            + 2;
        // SWF v8 and later use ImportAssets2 tag.
        if self.version >= 8 {
            self.write_tag_header(TagCode::ImportAssets2, len + 2)?;
            self.write_string(tag.url)?;
            self.write_u8(1)?;
            self.write_u8(0)?;
        } else {
            self.write_tag_header(TagCode::ImportAssets, len)?;
            self.write_string(tag.url)?;
        }
        self.write_u16(tag.imports.len() as u16)?;
        for &ExportedAsset { id, name } in &tag.imports {
            self.write_u16(id)?;
            self.write_string(name)?;
        }
        Ok(())
    }

    fn write_jpeg_tables(&mut self, tag: &JpegTables) -> Result<()> {
        self.write_tag_header(TagCode::JpegTables, tag.0.len() as u32)?;
        self.output.write_all(tag.0)?;
        Ok(())
    }

    fn write_metadata(&mut self, tag: &Metadata) -> Result<()> {
        self.write_tag_header(TagCode::Metadata, tag.metadata.len() as u32 + 1)?;
        self.write_string(tag.metadata)?;
        Ok(())
    }

    fn write_name_character(&mut self, tag: &NameCharacter) -> Result<()> {
        self.write_tag_header(TagCode::NameCharacter, 3 + tag.name.len() as u32)?;
        self.write_character_id(tag.id)?;
        self.write_string(tag.name)?;
        Ok(())
    }

    fn write_place_object(&mut self, tag: &PlaceObject) -> Result<()> {
        match tag.version {
            1 => self.write_place_object_1(tag),
            2 => self.write_place_object_2_or_3(tag, 2),
            3 => self.write_place_object_2_or_3(tag, 3),
            4 => self.write_place_object_2_or_3(tag, 4),
            _ => Err(Error::invalid_data("Invalid PlaceObject version.")),
        }
    }

    fn write_place_object_1(&mut self, place_object: &PlaceObject) -> Result<()> {
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
                writer.write_matrix(&Matrix::IDENTITY)?;
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

            let mut flags = PlaceFlag::empty();
            flags.set(
                PlaceFlag::MOVE,
                matches!(
                    place_object.action,
                    PlaceObjectAction::Modify | PlaceObjectAction::Replace(_)
                ),
            );
            flags.set(
                PlaceFlag::HAS_CHARACTER,
                matches!(
                    place_object.action,
                    PlaceObjectAction::Place(_) | PlaceObjectAction::Replace(_)
                ),
            );
            flags.set(PlaceFlag::HAS_MATRIX, place_object.matrix.is_some());
            flags.set(
                PlaceFlag::HAS_COLOR_TRANSFORM,
                place_object.color_transform.is_some(),
            );
            flags.set(PlaceFlag::HAS_RATIO, place_object.ratio.is_some());
            flags.set(PlaceFlag::HAS_NAME, place_object.name.is_some());
            flags.set(PlaceFlag::HAS_CLIP_DEPTH, place_object.clip_depth.is_some());
            flags.set(
                PlaceFlag::HAS_CLIP_ACTIONS,
                place_object.clip_actions.is_some(),
            );

            if place_object_version >= 3 {
                flags.set(PlaceFlag::HAS_FILTER_LIST, place_object.filters.is_some());
                flags.set(PlaceFlag::HAS_BLEND_MODE, place_object.blend_mode.is_some());
                flags.set(
                    PlaceFlag::HAS_CACHE_AS_BITMAP,
                    place_object.is_bitmap_cached.is_some(),
                );
                flags.set(PlaceFlag::HAS_CLASS_NAME, place_object.class_name.is_some());
                flags.set(PlaceFlag::HAS_IMAGE, place_object.has_image);
                flags.set(PlaceFlag::HAS_VISIBLE, place_object.is_visible.is_some());
                flags.set(
                    PlaceFlag::OPAQUE_BACKGROUND,
                    place_object.background_color.is_some(),
                );
                writer.write_u16(flags.bits())?;
            } else {
                writer.write_u8(flags.bits() as u8)?;
            }

            writer.write_u16(place_object.depth)?;

            if place_object_version >= 3 {
                if let Some(class_name) = place_object.class_name {
                    writer.write_string(class_name)?;
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
            if let Some(name) = place_object.name {
                writer.write_string(name)?;
            };
            if let Some(clip_depth) = place_object.clip_depth {
                writer.write_u16(clip_depth)?;
            }

            if place_object_version >= 3 {
                if let Some(filters) = &place_object.filters {
                    writer.write_u8(filters.len() as u8)?;
                    for filter in filters {
                        writer.write_filter(filter)?;
                    }
                }

                if let Some(blend_mode) = place_object.blend_mode {
                    writer.write_blend_mode(blend_mode)?;
                }

                if let Some(is_bitmap_cached) = place_object.is_bitmap_cached {
                    writer.write_u8(if is_bitmap_cached { 1 } else { 0 })?;
                }

                if let Some(is_visible) = place_object.is_visible {
                    writer.write_u8(if is_visible { 1 } else { 0 })?;
                }

                if let Some(ref background_color) = place_object.background_color {
                    writer.write_rgba(background_color)?;
                }
            }

            if let Some(clip_actions) = &place_object.clip_actions {
                writer.write_clip_actions(clip_actions)?;
            }

            // PlaceObject4 adds some embedded AMF data per instance.
            if place_object_version >= 4 {
                if let Some(data) = place_object.amf_data {
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

    fn write_product_info(&mut self, tag: &ProductInfo) -> Result<()> {
        self.write_tag_header(TagCode::ProductInfo, 26)?;
        self.write_u32(tag.product_id)?;
        self.write_u32(tag.edition)?;
        self.write_u8(tag.major_version)?;
        self.write_u8(tag.minor_version)?;
        self.write_u64(tag.build_number)?;
        self.write_u64(tag.compilation_date)?;
        Ok(())
    }

    fn write_protect(&mut self, tag: &Protect) -> Result<()> {
        if let Some(password_hash) = &tag.password_hash {
            self.write_tag_header(TagCode::Protect, password_hash.len() as u32 + 3)?;
            self.write_u16(0)?; // Two null bytes? Not specified in SWF19.
            self.write_string(password_hash)?;
        } else {
            self.write_tag_header(TagCode::Protect, 0)?;
        }
        Ok(())
    }

    fn write_remove_object(&mut self, tag: &RemoveObject) -> Result<()> {
        if let Some(id) = tag.character_id {
            self.write_tag_header(TagCode::RemoveObject, 4)?;
            self.write_u16(id)?;
        } else {
            self.write_tag_header(TagCode::RemoveObject2, 2)?;
        }
        self.write_u16(tag.depth)?;
        Ok(())
    }

    fn write_script_limits(&mut self, tag: &ScriptLimits) -> Result<()> {
        self.write_tag_header(TagCode::ScriptLimits, 4)?;
        self.write_u16(tag.max_recursion_depth)?;
        self.write_u16(tag.timeout_in_seconds)?;
        Ok(())
    }

    fn write_set_background_color(&mut self, tag: &SetBackgroundColor) -> Result<()> {
        self.write_tag_header(TagCode::SetBackgroundColor, 3)?;
        self.write_rgb(&tag.0)?;
        Ok(())
    }

    fn write_set_tab_index(&mut self, tag: &SetTabIndex) -> Result<()> {
        self.write_tag_header(TagCode::SetTabIndex, 4)?;
        self.write_u16(tag.depth)?;
        self.write_u16(tag.tab_index)?;
        Ok(())
    }

    fn write_sound_stream_block(&mut self, tag: &SoundStreamBlock) -> Result<()> {
        self.write_tag_header(TagCode::SoundStreamBlock, tag.data.len() as u32)?;
        self.output.write_all(tag.data)?;
        Ok(())
    }

    fn write_sound_stream_head(&mut self, tag: &SoundStreamHead, version: u8) -> Result<()> {
        let tag_code = if version >= 2 {
            TagCode::SoundStreamHead2
        } else {
            TagCode::SoundStreamHead
        };
        // MP3 compression has added latency seek field.
        let length = if tag.stream_format.compression == AudioCompression::Mp3 {
            6
        } else {
            4
        };
        self.write_tag_header(tag_code, length)?;
        self.write_sound_format(&tag.playback_format)?;
        self.write_sound_format(&tag.stream_format)?;
        self.write_u16(tag.num_samples_per_block)?;
        if tag.stream_format.compression == AudioCompression::Mp3 {
            self.write_i16(tag.latency_seek)?;
        }
        Ok(())
    }

    fn write_start_sound(&mut self, tag: &StartSound) -> Result<()> {
        let sound_info = &tag.sound_info;
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
        self.write_u16(tag.id)?;
        self.write_sound_info(sound_info)?;
        Ok(())
    }

    fn write_start_sound_2(&mut self, tag: &StartSound2) -> Result<()> {
        let length = tag.class_name.len() as u32
            + 2
            + if tag.sound_info.in_sample.is_some() {
                4
            } else {
                0
            }
            + if tag.sound_info.out_sample.is_some() {
                4
            } else {
                0
            }
            + if tag.sound_info.num_loops > 1 { 2 } else { 0 }
            + if let Some(ref e) = tag.sound_info.envelope {
                e.len() as u32 * 8 + 1
            } else {
                0
            };
        self.write_tag_header(TagCode::StartSound2, length)?;
        self.write_string(tag.class_name)?;
        self.write_sound_info(&tag.sound_info)?;
        Ok(())
    }

    fn write_symbol_class(&mut self, tag: &SymbolClass) -> Result<()> {
        let len = tag
            .0
            .iter()
            .map(|e| e.class_name.len() as u32 + 3)
            .sum::<u32>()
            + 2;
        self.write_tag_header(TagCode::SymbolClass, len)?;
        self.write_u16(tag.0.len() as u16)?;
        for symbol in &tag.0 {
            self.write_u16(symbol.id)?;
            self.write_string(symbol.class_name)?;
        }
        Ok(())
    }

    fn write_unknown(&mut self, tag: &Unknown) -> Result<()> {
        self.write_tag_code_and_length(tag.tag_code, tag.data.len() as u32)?;
        self.output.write_all(tag.data)?;
        Ok(())
    }

    fn write_video_frame(&mut self, tag: &VideoFrame) -> Result<()> {
        self.write_tag_header(TagCode::VideoFrame, 4 + tag.data.len() as u32)?;
        self.write_character_id(tag.stream_id)?;
        self.write_u16(tag.frame_num)?;
        self.output.write_all(tag.data)?;
        Ok(())
    }

    //=========================================================================
    // Shape types
    //=========================================================================

    fn write_shape_styles(&mut self, styles: &ShapeStyles, shape_version: u8) -> Result<(u8, u8)> {
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

        let num_fill_bits = count_ubits(styles.fill_styles.len() as u32) as u8;
        let num_line_bits = count_ubits(styles.line_styles.len() as u32) as u8;
        self.write_u8((num_fill_bits << 4) | (num_line_bits & 0b1111))?;

        Ok((num_fill_bits, num_line_bits))
    }

    fn write_shape_record<T: Write>(
        record: &ShapeRecord,
        bits: &mut BitWriter<T>,
        context: &mut ShapeContext,
    ) -> Result<()> {
        match *record {
            ShapeRecord::StraightEdge { delta_x, delta_y } => {
                bits.write_ubits(2, 0b11)?; // Straight edge
                                            // TODO: Check underflow?
                let mut num_bits = max(count_sbits_twips(delta_x), count_sbits_twips(delta_y));
                num_bits = max(2, num_bits);
                let is_axis_aligned = delta_x.get() == 0 || delta_y.get() == 0;
                bits.write_ubits(4, num_bits - 2)?;
                bits.write_bit(!is_axis_aligned)?;
                if is_axis_aligned {
                    bits.write_bit(delta_x.get() == 0)?;
                }
                if delta_x.get() != 0 {
                    bits.write_sbits_twips(num_bits, delta_x)?;
                }
                if delta_y.get() != 0 {
                    bits.write_sbits_twips(num_bits, delta_y)?;
                }
            }
            ShapeRecord::CurvedEdge {
                control_delta_x,
                control_delta_y,
                anchor_delta_x,
                anchor_delta_y,
            } => {
                bits.write_ubits(2, 0b10)?; // Curved edge
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
                bits.write_ubits(4, num_bits - 2)?;
                bits.write_sbits_twips(num_bits, control_delta_x)?;
                bits.write_sbits_twips(num_bits, control_delta_y)?;
                bits.write_sbits_twips(num_bits, anchor_delta_x)?;
                bits.write_sbits_twips(num_bits, anchor_delta_y)?;
            }
            ShapeRecord::StyleChange(ref style_change) => {
                bits.write_bit(false)?; // Style change
                let num_fill_bits = context.num_fill_bits.into();
                let num_line_bits = context.num_line_bits.into();
                bits.write_bit(style_change.new_styles.is_some())?;
                bits.write_bit(style_change.line_style.is_some())?;
                bits.write_bit(style_change.fill_style_1.is_some())?;
                bits.write_bit(style_change.fill_style_0.is_some())?;
                bits.write_bit(style_change.move_to.is_some())?;
                if let Some((move_x, move_y)) = style_change.move_to {
                    let num_bits = max(count_sbits_twips(move_x), count_sbits_twips(move_y));
                    bits.write_ubits(5, num_bits)?;
                    bits.write_sbits_twips(num_bits, move_x)?;
                    bits.write_sbits_twips(num_bits, move_y)?;
                }
                if let Some(fill_style_index) = style_change.fill_style_0 {
                    bits.write_ubits(num_fill_bits, fill_style_index)?;
                }
                if let Some(fill_style_index) = style_change.fill_style_1 {
                    bits.write_ubits(num_fill_bits, fill_style_index)?;
                }
                if let Some(line_style_index) = style_change.line_style {
                    bits.write_ubits(num_line_bits, line_style_index)?;
                }
                if let Some(ref new_styles) = style_change.new_styles {
                    if context.shape_version < 2 {
                        return Err(Error::invalid_data(
                            "Only DefineShape2 and higher may change styles.",
                        ));
                    }
                    bits.flush()?;
                    let mut writer = Writer::new(bits.writer(), context.swf_version);
                    let (num_fill_bits, num_line_bits) =
                        writer.write_shape_styles(new_styles, context.shape_version)?;
                    context.num_fill_bits = num_fill_bits;
                    context.num_line_bits = num_line_bits;
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
            self.write_u16(line_style.flags.bits())?;
            if let LineJoinStyle::Miter(miter_factor) = line_style.join_style() {
                self.write_fixed8(miter_factor)?;
            }
            if line_style.flags.contains(LineStyleFlag::HAS_FILL) {
                self.write_fill_style(&line_style.fill_style, shape_version)?;
            } else if let FillStyle::Color(color) = &line_style.fill_style {
                self.write_rgba(color)?;
            } else {
                return Err(Error::invalid_data("Unexpected line style fill type"));
            }
        } else {
            // LineStyle1
            let color = if let FillStyle::Color(color) = &line_style.fill_style {
                color
            } else {
                return Err(Error::invalid_data(
                    "Complex line styles can only be used in DefineShape4 tags",
                ));
            };
            if shape_version >= 3 {
                self.write_rgba(color)?
            } else {
                self.write_rgb(color)?
            }
        }
        Ok(())
    }

    fn write_gradient(&mut self, gradient: &Gradient, shape_version: u8) -> Result<()> {
        self.write_matrix(&gradient.matrix)?;
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
        let flags = ((gradient.spread as u8) << 6)
            | ((gradient.interpolation as u8) << 4)
            | ((gradient.records.len() as u8) & 0b1111);
        self.write_u8(flags)?;
        Ok(())
    }

    //=========================================================================
    // Morph shapes types
    //=========================================================================

    fn write_morph_fill_style(&mut self, start: &FillStyle, end: &FillStyle) -> Result<()> {
        match (start, end) {
            (FillStyle::Color(start_color), FillStyle::Color(end_color)) => {
                self.write_u8(0x00)?; // Solid color.
                self.write_rgba(start_color)?;
                self.write_rgba(end_color)?;
            }

            (
                FillStyle::LinearGradient(start_gradient),
                FillStyle::LinearGradient(end_gradient),
            ) => {
                self.write_u8(0x10)?; // Linear gradient.
                self.write_morph_gradient(start_gradient, end_gradient)?;
            }

            (
                FillStyle::RadialGradient(start_gradient),
                FillStyle::RadialGradient(end_gradient),
            ) => {
                self.write_u8(0x12)?; // Linear gradient.
                self.write_morph_gradient(start_gradient, end_gradient)?;
            }

            (
                FillStyle::FocalGradient {
                    gradient: start_gradient,
                    focal_point: start_focal_point,
                },
                FillStyle::FocalGradient {
                    gradient: end_gradient,
                    focal_point: end_focal_point,
                },
            ) => {
                self.write_u8(0x13)?; // Focal gradient.
                self.write_morph_gradient(start_gradient, end_gradient)?;
                self.write_fixed8(*start_focal_point)?;
                self.write_fixed8(*end_focal_point)?;
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
            match (&start.fill_style, &end.fill_style) {
                (FillStyle::Color(start), FillStyle::Color(end)) => {
                    self.write_rgba(start)?;
                    self.write_rgba(end)?;
                }
                _ => {
                    return Err(Error::invalid_data(
                        "Complex line styles can only be used in DefineMorphShape2 tags",
                    ));
                }
            }
        } else {
            if start.flags != end.flags {
                return Err(Error::invalid_data(
                    "Morph start and end line styles must have the same join parameters.",
                ));
            }

            // TODO(Herschel): Handle overflow.
            self.write_u16(start.width.get() as u16)?;
            self.write_u16(end.width.get() as u16)?;

            // MorphLineStyle2
            self.write_u16(start.flags.bits())?;
            if let LineJoinStyle::Miter(miter_factor) = start.join_style() {
                self.write_fixed8(miter_factor)?;
            }
            if start.flags.contains(LineStyleFlag::HAS_FILL) {
                self.write_morph_fill_style(&start.fill_style, &end.fill_style)?;
            } else {
                match (&start.fill_style, &end.fill_style) {
                    (FillStyle::Color(start), FillStyle::Color(end)) => {
                        self.write_rgba(start)?;
                        self.write_rgba(end)?;
                    }
                    _ => {
                        return Err(Error::invalid_data("Unexpected line fill style fill type"));
                    }
                }
            }
        }
        Ok(())
    }

    //=========================================================================
    // Button types
    //=========================================================================

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
        } | record.states.bits();
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

    //=========================================================================
    // Sprite types
    //=========================================================================

    fn write_clip_actions(&mut self, clip_actions: &[ClipAction]) -> Result<()> {
        self.write_u16(0)?; // Reserved
        {
            let mut all_events = ClipEventFlag::empty();
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
            self.output.write_all(action.action_data)?;
        }
        if self.version <= 5 {
            self.write_u16(0)?;
        } else {
            self.write_u32(0)?;
        }
        Ok(())
    }

    fn write_clip_event_flags(&mut self, clip_events: ClipEventFlag) -> Result<()> {
        // TODO: Assert proper version.
        let bits = clip_events.bits();
        if self.version >= 6 {
            self.write_u32(bits)?;
        } else {
            self.write_u16((bits as u8).into())?;
        }
        Ok(())
    }

    //=========================================================================
    // Text types
    //=========================================================================

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

    //=========================================================================
    // Filter types
    //=========================================================================

    fn write_filter(&mut self, filter: &Filter) -> Result<()> {
        match filter {
            Filter::DropShadowFilter(filter) => {
                self.write_u8(0)?;
                self.write_drop_shadow_filter(filter)
            }
            Filter::BlurFilter(filter) => {
                self.write_u8(1)?;
                self.write_blur_filter(filter)
            }
            Filter::GlowFilter(filter) => {
                self.write_u8(2)?;
                self.write_glow_filter(filter)
            }
            Filter::BevelFilter(filter) => {
                self.write_u8(3)?;
                self.write_bevel_filter(filter)
            }
            Filter::GradientGlowFilter(filter) => {
                self.write_u8(4)?;
                self.write_gradient_filter(filter)
            }
            Filter::ConvolutionFilter(filter) => {
                self.write_u8(5)?;
                self.write_convolution_filter(filter)
            }
            Filter::ColorMatrixFilter(filter) => {
                self.write_u8(6)?;
                self.write_color_matrix_filter(filter)
            }
            Filter::GradientBevelFilter(filter) => {
                self.write_u8(7)?;
                self.write_gradient_filter(filter)
            }
        }
    }

    fn write_drop_shadow_filter(&mut self, filter: &DropShadowFilter) -> Result<()> {
        self.write_rgba(&filter.color)?;
        self.write_fixed16(filter.blur_x)?;
        self.write_fixed16(filter.blur_y)?;
        self.write_fixed16(filter.angle)?;
        self.write_fixed16(filter.distance)?;
        self.write_fixed8(filter.strength)?;
        self.write_u8(filter.flags.bits())?;
        Ok(())
    }

    fn write_blur_filter(&mut self, filter: &BlurFilter) -> Result<()> {
        self.write_fixed16(filter.blur_x)?;
        self.write_fixed16(filter.blur_y)?;
        self.write_u8(filter.flags.bits())?;
        Ok(())
    }

    fn write_glow_filter(&mut self, filter: &GlowFilter) -> Result<()> {
        self.write_rgba(&filter.color)?;
        self.write_fixed16(filter.blur_x)?;
        self.write_fixed16(filter.blur_y)?;
        self.write_fixed8(filter.strength)?;
        self.write_u8(filter.flags.bits())?;
        Ok(())
    }

    fn write_bevel_filter(&mut self, filter: &BevelFilter) -> Result<()> {
        self.write_rgba(&filter.shadow_color)?;
        self.write_rgba(&filter.highlight_color)?;
        self.write_fixed16(filter.blur_x)?;
        self.write_fixed16(filter.blur_y)?;
        self.write_fixed16(filter.angle)?;
        self.write_fixed16(filter.distance)?;
        self.write_fixed8(filter.strength)?;
        self.write_u8(filter.flags.bits())?;
        Ok(())
    }

    fn write_gradient_filter(&mut self, filter: &GradientFilter) -> Result<()> {
        self.write_u8(filter.colors.len() as u8)?;
        for gradient_record in &filter.colors {
            self.write_rgba(&gradient_record.color)?;
        }
        for gradient_record in &filter.colors {
            self.write_u8(gradient_record.ratio)?;
        }
        self.write_fixed16(filter.blur_x)?;
        self.write_fixed16(filter.blur_y)?;
        self.write_fixed16(filter.angle)?;
        self.write_fixed16(filter.distance)?;
        self.write_fixed8(filter.strength)?;
        self.write_u8(filter.flags.bits())?;
        Ok(())
    }

    fn write_convolution_filter(&mut self, filter: &ConvolutionFilter) -> Result<()> {
        self.write_u8(filter.num_matrix_cols)?;
        self.write_u8(filter.num_matrix_rows)?;
        self.write_fixed16(filter.divisor)?;
        self.write_fixed16(filter.bias)?;
        for val in &filter.matrix {
            self.write_fixed16(*val)?;
        }
        self.write_rgba(&filter.default_color)?;
        self.write_u8(filter.flags.bits())?;
        Ok(())
    }

    fn write_color_matrix_filter(&mut self, filter: &ColorMatrixFilter) -> Result<()> {
        for m in filter.matrix {
            self.write_f32(m)?;
        }
        Ok(())
    }

    //=========================================================================
    // Sound types
    //=========================================================================

    fn write_sound_format(&mut self, sound_format: &SoundFormat) -> Result<()> {
        let mut bits = self.bits();
        bits.write_ubits(4, sound_format.compression as u32)?;
        bits.write_ubits(
            2,
            match sound_format.sample_rate {
                5512 => 0,
                11025 => 1,
                22050 => 2,
                44100 => 3,
                _ => return Err(Error::invalid_data("Invalid sample rate.")),
            },
        )?;
        bits.write_bit(sound_format.is_16_bit)?;
        bits.write_bit(sound_format.is_stereo)?;
        Ok(())
    }

    fn write_sound_info(&mut self, sound_info: &SoundInfo) -> Result<()> {
        let flags = (sound_info.event as u8) << 4
            | if sound_info.in_sample.is_some() {
                0b1
            } else {
                0
            }
            | if sound_info.out_sample.is_some() {
                0b10
            } else {
                0
            }
            | if sound_info.num_loops > 1 { 0b100 } else { 0 }
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
}

fn count_ubits(n: u32) -> u32 {
    32 - n.leading_zeros()
}

fn count_sbits(n: i32) -> u32 {
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

fn count_sbits_twips(n: Twips) -> u32 {
    count_sbits(n.get())
}

fn count_fbits(n: Fixed16) -> u32 {
    count_sbits(n.get())
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
mod tests {
    use super::Writer;
    use super::*;
    use crate::test_data;

    #[test]
    fn write_swfs() {
        fn write_dummy_swf(compression: Compression) -> Result<()> {
            let mut buf = Vec::new();
            let header = Header {
                compression,
                version: 13,
                stage_size: Rectangle {
                    x_min: Twips::from_pixels(0.0),
                    x_max: Twips::from_pixels(640.0),
                    y_min: Twips::from_pixels(0.0),
                    y_max: Twips::from_pixels(480.0),
                },
                frame_rate: Fixed8::from_f32(60.0),
                num_frames: 1,
            };
            write_swf(&header, &[], &mut buf)?;
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
            writer.write_fixed8(Fixed8::ZERO).unwrap();
            writer.write_fixed8(Fixed8::ONE).unwrap();
            writer.write_fixed8(Fixed8::from_f32(6.5)).unwrap();
            writer.write_fixed8(Fixed8::from_f32(-20.75)).unwrap();
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
        let out_bits = [
            false, true, false, true, false, true, false, true, false, false, true, false, false,
            true, false, true,
        ];
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            let mut bits = writer.bits();
            for b in &out_bits {
                bits.write_bit(*b).unwrap();
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
            let mut bits = writer.bits();
            for n in &nums {
                bits.write_ubits(num_bits, *n).unwrap();
            }
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
            let mut bits = writer.bits();
            for n in &nums {
                bits.write_sbits(num_bits, *n).unwrap();
            }
        }
        assert_eq!(buf, [0b01010101, 0b00100101]);
    }

    #[test]
    fn write_fbits() {
        let num_bits = 18;
        let nums = [1.0, -1.0];
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            let mut bits = writer.bits();
            for n in &nums {
                bits.write_fbits(num_bits, Fixed16::from_f32(*n)).unwrap();
            }
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
        assert_eq!(super::count_ubits(0), 0);
        assert_eq!(super::count_ubits(1u32), 1);
        assert_eq!(super::count_ubits(2u32), 2);
        assert_eq!(super::count_ubits(0b_00111101_00000000u32), 14);
    }

    #[test]
    fn count_sbits() {
        assert_eq!(super::count_sbits(0), 0);
        assert_eq!(super::count_sbits(1), 2);
        assert_eq!(super::count_sbits(2), 3);
        assert_eq!(super::count_sbits(0b_00111101_00000000), 15);

        assert_eq!(super::count_sbits(-1), 1);
        assert_eq!(super::count_sbits(-2), 2);
        assert_eq!(super::count_sbits(-0b_00110101_01010101), 15);
    }

    #[test]
    fn write_c_string() {
        {
            let mut buf = Vec::new();
            {
                // TODO: What if I use a cursor instead of buf ?
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_string("Hello!".into()).unwrap();
            }
            assert_eq!(buf, "Hello!\0".bytes().collect::<Vec<_>>());
        }

        {
            let mut buf = Vec::new();
            {
                // TODO: What if I use a cursor instead of buf ?
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_string("😀😂!🐼".into()).unwrap();
            }
            assert_eq!(buf, "😀😂!🐼\0".bytes().collect::<Vec<_>>());
        }
    }

    #[test]
    fn write_rectangle_zero() {
        let rectangle = Rectangle {
            x_min: Twips::ZERO,
            y_min: Twips::ZERO,
            x_max: Twips::ZERO,
            y_max: Twips::ZERO,
        };
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_rectangle(&rectangle).unwrap();
        }
        assert_eq!(buf, [0]);
    }

    #[test]
    fn write_rectangle_signed() {
        let rectangle = Rectangle {
            x_min: Twips::from_pixels(-1.0),
            x_max: Twips::from_pixels(1.0),
            y_min: Twips::from_pixels(-1.0),
            y_max: Twips::from_pixels(1.0),
        };
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_rectangle(&rectangle).unwrap();
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
            }
            buf
        }

        let m = Matrix::IDENTITY;
        assert_eq!(write_to_buf(&m), [0]);
    }

    #[test]
    fn write_tags() {
        for (swf_version, tag, expected_tag_bytes) in test_data::tag_tests() {
            let mut written_tag_bytes = Vec::new();
            Writer::new(&mut written_tag_bytes, swf_version)
                .write_tag(&tag)
                .unwrap();
            assert_eq!(
                written_tag_bytes, expected_tag_bytes,
                "Error reading tag.\nTag:\n{tag:?}\n\nWrote:\n{written_tag_bytes:?}\n\nExpected:\n{expected_tag_bytes:?}",
            );
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
                        Tag::Unknown(Unknown {
                            tag_code: 512,
                            data: &[0; 100],
                        }),
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
