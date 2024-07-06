use crate::extensions::ReadSwfExt;
use crate::{
    error::{Error, Result},
    string::{Encoding, SwfStr},
    tag_code::TagCode,
    types::*,
};
use bitstream_io::BitRead;
use byteorder::{LittleEndian, ReadBytesExt};
use simple_asn1::ASN1Block;
use std::borrow::Cow;
use std::io::{self, Read};

/// Parse a decompressed SWF.
///
/// # Example
/// ```
/// # std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"));
/// let data = std::fs::read("tests/swfs/DefineSprite.swf").unwrap();
/// let stream = swf::decompress_swf(&data[..]).unwrap();
/// let swf = swf::parse_swf(&stream).unwrap();
/// println!("Number of frames: {}", swf.header.num_frames());
/// ```
pub fn parse_swf(swf_buf: &SwfBuf) -> Result<Swf<'_>> {
    let mut reader = Reader::new(&swf_buf.data[..], swf_buf.header.version());

    Ok(Swf {
        header: swf_buf.header.clone(),
        tags: reader.read_tag_list()?,
    })
}

/// Extracts an SWF inside of an SWZ file.
pub fn extract_swz(input: &[u8]) -> Result<Vec<u8>> {
    let asn1_blocks =
        simple_asn1::from_der(input).map_err(|_| Error::invalid_data("Invalid ASN1 blob"))?;
    if let Some(ASN1Block::Sequence(_, s)) = asn1_blocks.into_iter().next() {
        for t in s {
            if let ASN1Block::Explicit(_, _, _, block) = t {
                if let ASN1Block::Sequence(_, s) = *block {
                    if let Some(ASN1Block::Sequence(_, s)) = s.into_iter().nth(2) {
                        if let Some(ASN1Block::Explicit(_, _, _, octet)) = s.into_iter().nth(1) {
                            if let ASN1Block::OctetString(_, bytes) = *octet {
                                return Ok(bytes);
                            }
                        }
                    }
                }
            }
        }
    }
    Err(Error::invalid_data("Invalid ASN1 blob"))
}

/// Parses an SWF header and returns a `Reader` that can be used
/// to read the SWF tags inside the SWF file.
///
/// Returns an `Error` if this is not a valid SWF file.
///
/// This will also parse the first two tags of the SWF file searching
/// for the FileAttributes and SetBackgroundColor tags; this info is
/// returned as an extended header.
///
/// # Example
/// ```
/// # std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"));
/// let data = std::fs::read("tests/swfs/DefineSprite.swf").unwrap();
/// let swf_stream = swf::decompress_swf(&data[..]).unwrap();
/// println!("FPS: {}", swf_stream.header.frame_rate());
/// ```
pub fn decompress_swf<'a, R: Read + 'a>(mut input: R) -> Result<SwfBuf> {
    // Read SWF header.
    let compression = read_compression_type(&mut input)?;
    let version = input.read_u8()?;
    let uncompressed_len = input.read_u32::<LittleEndian>()?;

    // Check whether the SWF version is 0.
    // Note that the behavior should actually vary, depending on the player version:
    // - Flash Player 9 and later bail out (the behavior we implement).
    // - Flash Player 8 loops through all the frames, without running any AS code.
    // - Flash Player 7 and older don't fail and use the player version instead: a
    // function like `getSWFVersion()` in AVM1 will then return the player version.
    if version == 0 {
        return Err(Error::invalid_data("Invalid SWF version"));
    }

    // Now the SWF switches to a compressed stream.
    let mut decompress_stream: Box<dyn Read> = match compression {
        Compression::None => Box::new(input),
        Compression::Zlib => {
            if version < 6 {
                log::warn!(
                    "zlib compressed SWF is version {} but minimum version is 6",
                    version
                );
            }
            make_zlib_reader(input)?
        }
        Compression::Lzma => {
            if version < 13 {
                log::warn!(
                    "LZMA compressed SWF is version {} but minimum version is 13",
                    version
                );
            }
            // Uncompressed length includes the 4-byte header and 4-byte uncompressed length itself,
            // subtract it here.
            make_lzma_reader(input, uncompressed_len - 8)?
        }
    };

    // Decompress the entire SWF.
    let mut data = Vec::with_capacity(uncompressed_len as usize);
    if let Err(e) = decompress_stream.read_to_end(&mut data) {
        log::error!("Error decompressing SWF: {}", e);
    }

    // Some SWF streams may not be compressed correctly,
    // (e.g. incorrect data length in the stream), so decompressing
    // may throw an error even though the data otherwise comes
    // through the stream.
    // We'll still try to parse what we get if the full decompression fails.
    // (+ 8 for header size)
    if data.len() as u64 + 8 != uncompressed_len as u64 {
        log::warn!("SWF length doesn't match header, may be corrupt");
    }

    let mut reader = Reader::new(&data, version);
    let stage_size = reader.read_rectangle()?;
    let frame_rate = reader.read_fixed8()?;
    let num_frames = reader.read_u16()?;
    let header = Header {
        compression,
        version,
        stage_size,
        frame_rate,
        num_frames,
    };
    let data = reader.get_ref().to_vec();

    // Parse the first two tags, searching for the FileAttributes and SetBackgroundColor tags.
    // This metadata is useful, so we want to return it along with the header.
    // In SWF8+, FileAttributes should be the first tag in the SWF.
    // FileAttributes anywhere else in the SWF are ignored.
    let mut tag = reader.read_tag();
    let file_attributes = if let Ok(Tag::FileAttributes(attributes)) = tag {
        tag = reader.read_tag();
        attributes
    } else {
        FileAttributes::default()
    };

    // In most SWFs, SetBackgroundColor will be the second or third tag after FileAttributes + Metadata.
    // It's possible for the SetBackgroundColor tag to be missing or appear later in wacky SWFs, so let's
    // return `None` in this case.
    let mut background_color = None;
    for _ in 0..2 {
        if let Ok(Tag::SetBackgroundColor(color)) = tag {
            background_color = Some(color);
            break;
        };
        tag = reader.read_tag();
    }

    Ok(SwfBuf {
        header: HeaderExt {
            header,
            file_attributes,
            background_color,
            uncompressed_len: uncompressed_len as i32,
        },
        data,
    })
}

#[cfg(feature = "flate2")]
fn make_zlib_reader<'a, R: Read + 'a>(input: R) -> Result<Box<dyn Read + 'a>> {
    use flate2::read::ZlibDecoder;
    Ok(Box::new(ZlibDecoder::new(input)))
}

#[cfg(not(feature = "flate2"))]
fn make_zlib_reader<'a, R: Read + 'a>(_input: R) -> Result<Box<dyn Read + 'a>> {
    Err(Error::unsupported(
        "Support for Zlib compressed SWFs is not enabled.",
    ))
}

#[cfg(feature = "lzma")]
fn make_lzma_reader<'a, R: Read + 'a>(
    mut input: R,
    uncompressed_length: u32,
) -> Result<Box<dyn Read + 'a>> {
    use lzma_rs::{
        decompress::{Options, UnpackedSize},
        lzma_decompress_with_options,
    };
    // Flash uses a mangled LZMA header, so we have to massage it into the normal format.
    // https://helpx.adobe.com/flash-player/kb/exception-thrown-you-decompress-lzma-compressed.html
    // LZMA SWF header:
    // Bytes 0..3: ZWS header
    // Byte 3: SWF version
    // Bytes 4..8: Uncompressed length
    // Bytes 8..12: Compressed length
    // Bytes 12..17: LZMA properties
    //
    // LZMA standard header
    // Bytes 0..5: LZMA properties
    // Bytes 5..13: Uncompressed length
    //
    // To deal with the mangled header, use lzma_rs options to manually provide uncompressed length.

    // Read compressed length (ignored)
    let _ = input.read_u32::<LittleEndian>()?;

    // TODO: Switch to lzma-rs streaming API when stable.
    let mut output = Vec::with_capacity(uncompressed_length as usize);
    lzma_decompress_with_options(
        &mut io::BufReader::new(input),
        &mut output,
        &Options {
            unpacked_size: UnpackedSize::UseProvided(Some(uncompressed_length.into())),
            allow_incomplete: true,
            memlimit: None,
        },
    )
    .map_err(|_| Error::invalid_data("Unable to decompress LZMA SWF."))?;

    Ok(Box::new(io::Cursor::new(output)))
}

#[cfg(not(feature = "lzma"))]
fn make_lzma_reader<'a, R: Read + 'a>(
    _input: R,
    _uncompressed_length: u32,
) -> Result<Box<dyn Read + 'a>> {
    Err(Error::unsupported(
        "Support for LZMA compressed SWFs is not enabled.",
    ))
}

struct BitReader<'a, 'b> {
    bits: bitstream_io::BitReader<&'b mut &'a [u8], bitstream_io::BigEndian>,
}

impl<'a, 'b> BitReader<'a, 'b> {
    #[inline]
    fn new(data: &'b mut &'a [u8]) -> Self {
        Self {
            bits: bitstream_io::BitReader::new(data),
        }
    }

    #[inline]
    fn read_bit(&mut self) -> io::Result<bool> {
        self.bits.read_bit()
    }

    #[inline]
    fn read_ubits(&mut self, num_bits: u32) -> io::Result<u32> {
        if num_bits > 0 {
            self.bits.read(num_bits)
        } else {
            Ok(0)
        }
    }

    #[inline]
    fn read_sbits(&mut self, num_bits: u32) -> io::Result<i32> {
        if num_bits > 0 {
            self.bits.read_signed(num_bits)
        } else {
            Ok(0)
        }
    }

    #[inline]
    fn read_sbits_fixed8(&mut self, num_bits: u32) -> io::Result<Fixed8> {
        self.read_sbits(num_bits)
            .map(|n| Fixed8::from_bits(n as i16))
    }

    #[inline]
    fn read_sbits_twips(&mut self, num_bits: u32) -> io::Result<Twips> {
        self.read_sbits(num_bits).map(Twips::new)
    }

    #[inline]
    fn read_fbits(&mut self, num_bits: u32) -> io::Result<Fixed16> {
        self.read_sbits(num_bits).map(Fixed16::from_bits)
    }

    #[inline]
    fn reader(&mut self) -> &mut &'a [u8] {
        self.bits.byte_align();
        self.bits.reader().unwrap()
    }
}

pub struct Reader<'a> {
    input: &'a [u8],
    version: u8,
}

impl<'a> ReadSwfExt<'a> for Reader<'a> {
    #[inline(always)]
    fn as_mut_slice(&mut self) -> &mut &'a [u8] {
        &mut self.input
    }

    #[inline(always)]
    fn as_slice(&self) -> &'a [u8] {
        self.input
    }
}

impl<'a> Reader<'a> {
    #[inline]
    pub const fn new(input: &'a [u8], version: u8) -> Self {
        Self { input, version }
    }

    /// Returns the suggested string encoding for this SWF.
    /// For SWF version 6 and higher, this is always UTF-8.
    /// For SWF version 5 and lower, this is locale-dependent,
    /// and we default to WINDOWS-1252.
    #[inline]
    pub fn encoding(&self) -> &'static Encoding {
        SwfStr::encoding_for_version(self.version)
    }

    #[inline]
    pub const fn version(&self) -> u8 {
        self.version
    }

    /// Returns a reference to the underlying `Reader`.
    #[inline]
    pub const fn get_ref(&self) -> &'a [u8] {
        self.input
    }

    /// Returns a mutable reference to the underlying `Reader`.
    ///
    /// Reading from this reference is not recommended.
    #[inline]
    pub fn get_mut(&mut self) -> &mut &'a [u8] {
        &mut self.input
    }

    fn bits<'b>(&'b mut self) -> BitReader<'a, 'b> {
        BitReader::new(&mut self.input)
    }

    #[inline]
    fn read_fixed8(&mut self) -> Result<Fixed8> {
        Ok(Fixed8::from_bits(self.read_i16()?))
    }

    #[inline]
    fn read_fixed16(&mut self) -> Result<Fixed16> {
        Ok(Fixed16::from_bits(self.read_i32()?))
    }

    /// Reads the next SWF tag from the stream.
    /// # Example
    /// ```
    /// # std::env::set_current_dir(env!("CARGO_MANIFEST_DIR"));
    /// let data = std::fs::read("tests/swfs/DefineSprite.swf").unwrap();
    /// let mut swf_buf = swf::decompress_swf(&data[..]).unwrap();
    /// let mut reader = swf::read::Reader::new(&swf_buf.data[..], swf_buf.header.version());
    /// while let Ok(tag) = reader.read_tag() {
    ///     println!("Tag: {:?}", tag);
    /// }
    /// ```
    pub fn read_tag(&mut self) -> Result<Tag<'a>> {
        let (tag_code, length) = self.read_tag_code_and_length()?;

        if let Some(tag_code) = TagCode::from_u16(tag_code) {
            self.read_tag_with_code(tag_code, length)
        } else {
            self.read_slice(length)
                .map(|data| Tag::Unknown { tag_code, data })
        }
        .map_err(|e| Error::swf_parse_error(tag_code, e))
    }

    fn read_tag_with_code(&mut self, tag_code: TagCode, length: usize) -> Result<Tag<'a>> {
        let mut tag_reader = Reader::new(self.read_slice(length)?, self.version);
        let tag = match tag_code {
            TagCode::End => Tag::End,
            TagCode::ShowFrame => Tag::ShowFrame,
            TagCode::CsmTextSettings => Tag::CsmTextSettings(tag_reader.read_csm_text_settings()?),
            TagCode::DefineBinaryData => {
                Tag::DefineBinaryData(tag_reader.read_define_binary_data()?)
            }
            TagCode::DefineBits => {
                let id = tag_reader.read_u16()?;
                let jpeg_data = tag_reader.read_slice_to_end();
                Tag::DefineBits { id, jpeg_data }
            }
            TagCode::DefineBitsJpeg2 => {
                let id = tag_reader.read_u16()?;
                let jpeg_data = tag_reader.read_slice_to_end();
                Tag::DefineBitsJpeg2 { id, jpeg_data }
            }
            TagCode::DefineBitsJpeg3 => {
                Tag::DefineBitsJpeg3(tag_reader.read_define_bits_jpeg_3(3)?)
            }
            TagCode::DefineBitsJpeg4 => {
                Tag::DefineBitsJpeg3(tag_reader.read_define_bits_jpeg_3(4)?)
            }
            TagCode::DefineButton => {
                Tag::DefineButton(Box::new(tag_reader.read_define_button_1()?))
            }
            TagCode::DefineButton2 => {
                Tag::DefineButton2(Box::new(tag_reader.read_define_button_2()?))
            }
            TagCode::DefineButtonCxform => {
                Tag::DefineButtonColorTransform(tag_reader.read_define_button_cxform()?)
            }
            TagCode::DefineButtonSound => {
                Tag::DefineButtonSound(Box::new(tag_reader.read_define_button_sound()?))
            }
            TagCode::DefineEditText => {
                Tag::DefineEditText(Box::new(tag_reader.read_define_edit_text()?))
            }
            TagCode::DefineFont => Tag::DefineFont(Box::new(tag_reader.read_define_font_1()?)),
            TagCode::DefineFont2 => Tag::DefineFont2(Box::new(tag_reader.read_define_font_2(2)?)),
            TagCode::DefineFont3 => Tag::DefineFont2(Box::new(tag_reader.read_define_font_2(3)?)),
            TagCode::DefineFont4 => Tag::DefineFont4(tag_reader.read_define_font_4()?),
            TagCode::DefineFontAlignZones => tag_reader.read_define_font_align_zones()?,
            TagCode::DefineFontInfo => {
                Tag::DefineFontInfo(Box::new(tag_reader.read_define_font_info(1)?))
            }
            TagCode::DefineFontInfo2 => {
                Tag::DefineFontInfo(Box::new(tag_reader.read_define_font_info(2)?))
            }
            TagCode::DefineFontName => tag_reader.read_define_font_name()?,
            TagCode::DefineMorphShape => {
                Tag::DefineMorphShape(Box::new(tag_reader.read_define_morph_shape(1)?))
            }
            TagCode::DefineMorphShape2 => {
                Tag::DefineMorphShape(Box::new(tag_reader.read_define_morph_shape(2)?))
            }
            TagCode::DefineShape => Tag::DefineShape(tag_reader.read_define_shape(1)?),
            TagCode::DefineShape2 => Tag::DefineShape(tag_reader.read_define_shape(2)?),
            TagCode::DefineShape3 => Tag::DefineShape(tag_reader.read_define_shape(3)?),
            TagCode::DefineShape4 => Tag::DefineShape(tag_reader.read_define_shape(4)?),
            TagCode::DefineSound => Tag::DefineSound(Box::new(tag_reader.read_define_sound()?)),
            TagCode::DefineText => Tag::DefineText(Box::new(tag_reader.read_define_text(1)?)),
            TagCode::DefineText2 => Tag::DefineText2(Box::new(tag_reader.read_define_text(2)?)),
            TagCode::DefineVideoStream => {
                Tag::DefineVideoStream(tag_reader.read_define_video_stream()?)
            }
            TagCode::EnableTelemetry => {
                tag_reader.read_u16()?; // Reserved
                let password_hash = if length > 2 {
                    tag_reader.read_slice(32)?
                } else {
                    &[]
                };
                Tag::EnableTelemetry { password_hash }
            }
            TagCode::ImportAssets => {
                let import_assets = tag_reader.read_import_assets()?;
                Tag::ImportAssets {
                    url: import_assets.0,
                    imports: import_assets.1,
                }
            }
            TagCode::ImportAssets2 => {
                let import_assets = tag_reader.read_import_assets_2()?;
                Tag::ImportAssets {
                    url: import_assets.0,
                    imports: import_assets.1,
                }
            }

            TagCode::JpegTables => {
                let data = tag_reader.read_slice_to_end();
                Tag::JpegTables(data)
            }

            TagCode::Metadata => Tag::Metadata(tag_reader.read_str()?),

            TagCode::SetBackgroundColor => Tag::SetBackgroundColor(tag_reader.read_rgb()?),

            TagCode::SoundStreamBlock => {
                let data = tag_reader.read_slice_to_end();
                Tag::SoundStreamBlock(data)
            }

            TagCode::SoundStreamHead => Tag::SoundStreamHead(
                // TODO: Disallow certain compressions.
                Box::new(tag_reader.read_sound_stream_head()?),
            ),

            TagCode::SoundStreamHead2 => {
                Tag::SoundStreamHead2(Box::new(tag_reader.read_sound_stream_head()?))
            }

            TagCode::StartSound => Tag::StartSound(tag_reader.read_start_sound_1()?),

            TagCode::StartSound2 => Tag::StartSound2 {
                class_name: tag_reader.read_str()?,
                sound_info: Box::new(tag_reader.read_sound_info()?),
            },

            TagCode::DebugId => Tag::DebugId(tag_reader.read_debug_id()?),

            TagCode::DefineBitsLossless => {
                Tag::DefineBitsLossless(tag_reader.read_define_bits_lossless(1)?)
            }
            TagCode::DefineBitsLossless2 => {
                Tag::DefineBitsLossless(tag_reader.read_define_bits_lossless(2)?)
            }

            TagCode::DefineScalingGrid => Tag::DefineScalingGrid {
                id: tag_reader.read_u16()?,
                splitter_rect: tag_reader.read_rectangle()?,
            },

            TagCode::DoAbc => {
                let data = tag_reader.read_slice_to_end();
                Tag::DoAbc(data)
            }
            TagCode::DoAbc2 => Tag::DoAbc2(tag_reader.read_do_abc_2()?),

            TagCode::DoAction => {
                let action_data = tag_reader.read_slice_to_end();
                Tag::DoAction(action_data)
            }

            TagCode::DoInitAction => {
                let id = tag_reader.read_u16()?;
                let action_data = tag_reader.read_slice_to_end();
                Tag::DoInitAction { id, action_data }
            }

            TagCode::EnableDebugger => Tag::EnableDebugger(tag_reader.read_str()?),
            TagCode::EnableDebugger2 => {
                tag_reader.read_u16()?; // Reserved
                Tag::EnableDebugger(tag_reader.read_str()?)
            }

            TagCode::ScriptLimits => Tag::ScriptLimits {
                max_recursion_depth: tag_reader.read_u16()?,
                timeout_in_seconds: tag_reader.read_u16()?,
            },

            TagCode::SetTabIndex => Tag::SetTabIndex {
                depth: tag_reader.read_u16()?,
                tab_index: tag_reader.read_u16()?,
            },

            TagCode::SymbolClass => {
                let num_symbols = tag_reader.read_u16()?;
                let mut symbols = Vec::with_capacity(num_symbols as usize);
                for _ in 0..num_symbols {
                    symbols.push(SymbolClassLink {
                        id: tag_reader.read_u16()?,
                        class_name: tag_reader.read_str()?,
                    });
                }
                Tag::SymbolClass(symbols)
            }

            TagCode::ExportAssets => Tag::ExportAssets(tag_reader.read_export_assets()?),

            TagCode::FileAttributes => Tag::FileAttributes(tag_reader.read_file_attributes()?),

            TagCode::Protect => {
                Tag::Protect(if length > 0 {
                    tag_reader.read_u16()?; // TODO(Herschel): Two null bytes? Not specified in SWF19.
                    Some(tag_reader.read_str()?)
                } else {
                    None
                })
            }

            TagCode::DefineSceneAndFrameLabelData => Tag::DefineSceneAndFrameLabelData(
                tag_reader.read_define_scene_and_frame_label_data()?,
            ),

            TagCode::FrameLabel => Tag::FrameLabel(tag_reader.read_frame_label()?),

            TagCode::DefineSprite => Tag::DefineSprite(tag_reader.read_define_sprite()?),

            TagCode::PlaceObject => Tag::PlaceObject(Box::new(tag_reader.read_place_object()?)),
            TagCode::PlaceObject2 => {
                Tag::PlaceObject(Box::new(tag_reader.read_place_object_2_or_3(2)?))
            }
            TagCode::PlaceObject3 => {
                Tag::PlaceObject(Box::new(tag_reader.read_place_object_2_or_3(3)?))
            }
            TagCode::PlaceObject4 => {
                Tag::PlaceObject(Box::new(tag_reader.read_place_object_2_or_3(4)?))
            }

            TagCode::RemoveObject => Tag::RemoveObject(tag_reader.read_remove_object_1()?),

            TagCode::RemoveObject2 => Tag::RemoveObject(tag_reader.read_remove_object_2()?),

            TagCode::VideoFrame => Tag::VideoFrame(tag_reader.read_video_frame()?),
            TagCode::ProductInfo => Tag::ProductInfo(tag_reader.read_product_info()?),
            TagCode::NameCharacter => Tag::NameCharacter(tag_reader.read_name_character()?),
        };

        if !tag_reader.input.is_empty() {
            // There should be no data remaining in the tag if we read it correctly.
            // If there is data remaining, the most likely scenario is we screwed up parsing.
            // But sometimes tools will export SWF tags that are larger than they should be.
            // TODO: It might be worthwhile to have a "strict mode" to determine
            // whether this should error or not.
            log::warn!("Data remaining in buffer when parsing {:?}", tag_code);
        }

        Ok(tag)
    }

    pub fn read_rectangle(&mut self) -> Result<Rectangle<Twips>> {
        let mut bits = self.bits();
        let num_bits = bits.read_ubits(5)?;
        Ok(Rectangle {
            x_min: bits.read_sbits_twips(num_bits)?,
            x_max: bits.read_sbits_twips(num_bits)?,
            y_min: bits.read_sbits_twips(num_bits)?,
            y_max: bits.read_sbits_twips(num_bits)?,
        })
    }

    pub fn read_character_id(&mut self) -> Result<CharacterId> {
        let id = self.read_u16()?;
        Ok(id)
    }

    pub fn read_rgb(&mut self) -> Result<Color> {
        let r = self.read_u8()?;
        let g = self.read_u8()?;
        let b = self.read_u8()?;
        Ok(Color { r, g, b, a: 255 })
    }

    pub fn read_rgba(&mut self) -> Result<Color> {
        let r = self.read_u8()?;
        let g = self.read_u8()?;
        let b = self.read_u8()?;
        let a = self.read_u8()?;
        Ok(Color { r, g, b, a })
    }

    fn read_color_transform(&mut self, has_alpha: bool) -> Result<ColorTransform> {
        let mut bits = self.bits();
        let has_add = bits.read_bit()?;
        let has_mult = bits.read_bit()?;
        let num_bits = bits.read_ubits(4)?;
        let mut color_transform = ColorTransform::default();
        if has_mult {
            color_transform.r_multiply = bits.read_sbits_fixed8(num_bits)?;
            color_transform.g_multiply = bits.read_sbits_fixed8(num_bits)?;
            color_transform.b_multiply = bits.read_sbits_fixed8(num_bits)?;
            if has_alpha {
                color_transform.a_multiply = bits.read_sbits_fixed8(num_bits)?;
            }
        }
        if has_add {
            color_transform.r_add = bits.read_sbits(num_bits)? as i16;
            color_transform.g_add = bits.read_sbits(num_bits)? as i16;
            color_transform.b_add = bits.read_sbits(num_bits)? as i16;
            if has_alpha {
                color_transform.a_add = bits.read_sbits(num_bits)? as i16;
            }
        }
        Ok(color_transform)
    }

    fn read_matrix(&mut self) -> Result<Matrix> {
        let mut bits = self.bits();
        let mut m = Matrix::IDENTITY;
        // Scale
        if bits.read_bit()? {
            let num_bits = bits.read_ubits(5)?;
            m.a = bits.read_fbits(num_bits)?;
            m.d = bits.read_fbits(num_bits)?;
        }
        // Rotate/Skew
        if bits.read_bit()? {
            let num_bits = bits.read_ubits(5)?;
            m.b = bits.read_fbits(num_bits)?;
            m.c = bits.read_fbits(num_bits)?;
        }
        // Translate (always present)
        let num_bits = bits.read_ubits(5)?;
        m.tx = bits.read_sbits_twips(num_bits)?;
        m.ty = bits.read_sbits_twips(num_bits)?;
        Ok(m)
    }

    fn read_language(&mut self) -> Result<Language> {
        Language::from_u8(self.read_u8()?)
            .ok_or_else(|| Error::invalid_data("Invalid language code"))
    }

    fn read_tag_list(&mut self) -> Result<Vec<Tag<'a>>> {
        let mut tags = Vec::new();
        loop {
            let tag = self.read_tag()?;
            if tag == Tag::End {
                break;
            }
            tags.push(tag);
        }
        Ok(tags)
    }

    pub fn read_tag_code(&mut self) -> Result<u16> {
        Ok(self.read_u16()? >> 6)
    }

    pub fn read_tag_code_and_length(&mut self) -> Result<(u16, usize)> {
        let tag_code_and_length = self.read_u16()?;
        let tag_code = tag_code_and_length >> 6;
        let mut length = (tag_code_and_length & 0b111111) as usize;
        if length == 0b111111 {
            // Extended tag.
            length = self.read_u32()? as usize;
        }
        Ok((tag_code, length))
    }

    pub fn read_define_button_1(&mut self) -> Result<Button<'a>> {
        let id = self.read_u16()?;
        let mut records = Vec::new();
        while let Some(record) = self.read_button_record(1)? {
            records.push(record);
        }
        let action_data = self.read_slice_to_end();
        Ok(Button {
            id,
            is_track_as_menu: false,
            records,
            actions: vec![ButtonAction {
                conditions: ButtonActionCondition::OVER_DOWN_TO_OVER_UP,
                action_data,
            }],
        })
    }

    pub fn read_define_button_2(&mut self) -> Result<Button<'a>> {
        let id = self.read_u16()?;
        let flags = self.read_u8()?;
        let is_track_as_menu = (flags & 0b1) != 0;
        let action_offset = self.read_u16()?;

        let mut records = Vec::new();
        while let Some(record) = self.read_button_record(2)? {
            records.push(record);
        }

        let mut actions = Vec::new();
        if action_offset != 0 {
            loop {
                let (button_action, has_more_actions) = self.read_button_action()?;
                actions.push(button_action);
                if !has_more_actions {
                    break;
                }
            }
        }

        Ok(Button {
            id,
            is_track_as_menu,
            records,
            actions,
        })
    }

    pub fn read_define_button_cxform(&mut self) -> Result<ButtonColorTransform> {
        // SWF19 is incorrect here. You can have >1 color transforms in this tag. They apply
        // to the characters in a button in sequence.

        let id = self.read_character_id()?;
        let mut color_transforms = Vec::new();

        // Read all color transforms.
        while let Ok(color_transform) = self.read_color_transform(false) {
            color_transforms.push(color_transform);
        }

        Ok(ButtonColorTransform {
            id,
            color_transforms,
        })
    }

    pub fn read_define_button_sound(&mut self) -> Result<ButtonSounds> {
        let button_id = self.read_u16()?;

        // Some SWFs (third-party soundboard creator?) create SWFs with a malformed
        // DefineButtonSound tag that has fewer than all 4 sound IDs.
        let over_to_up_sound = match self.read_u16() {
            Ok(sound_id) if sound_id != 0 => Some((sound_id, self.read_sound_info()?)),
            _ => None,
        };

        let up_to_over_sound = match self.read_u16() {
            Ok(sound_id) if sound_id != 0 => Some((sound_id, self.read_sound_info()?)),
            _ => None,
        };

        let over_to_down_sound = match self.read_u16() {
            Ok(sound_id) if sound_id != 0 => Some((sound_id, self.read_sound_info()?)),
            _ => None,
        };

        let down_to_over_sound = match self.read_u16() {
            Ok(sound_id) if sound_id != 0 => Some((sound_id, self.read_sound_info()?)),
            _ => None,
        };

        Ok(ButtonSounds {
            id: button_id,
            over_to_up_sound,
            up_to_over_sound,
            over_to_down_sound,
            down_to_over_sound,
        })
    }

    fn read_button_record(&mut self, version: u8) -> Result<Option<ButtonRecord>> {
        let flags = self.read_u8()?;
        if flags == 0 {
            return Ok(None);
        }
        let states = ButtonState::from_bits_truncate(flags);
        let id = self.read_u16()?;
        let depth = self.read_u16()?;
        let matrix = self.read_matrix()?;
        let color_transform = if version >= 2 {
            self.read_color_transform(true)?
        } else {
            ColorTransform::IDENTITY
        };
        let mut filters = vec![];
        if (flags & 0b1_0000) != 0 {
            let num_filters = self.read_u8()?;
            filters.reserve(num_filters as usize);
            for _ in 0..num_filters {
                filters.push(self.read_filter()?);
            }
        }
        let blend_mode = if (flags & 0b10_0000) != 0 {
            self.read_blend_mode()?
        } else {
            BlendMode::Normal
        };
        Ok(Some(ButtonRecord {
            states,
            id,
            depth,
            matrix,
            color_transform,
            filters,
            blend_mode,
        }))
    }

    fn read_button_action(&mut self) -> Result<(ButtonAction<'a>, bool)> {
        let length = self.read_u16()?;
        let flags = self.read_u16()?;
        let conditions = ButtonActionCondition::from_bits_retain(flags);
        let action_data = if length >= 4 {
            self.read_slice(length as usize - 4)?
        } else if length == 0 {
            // Last action, read to end.
            self.read_slice_to_end()
        } else {
            // Some SWFs have phantom action records with an invalid length.
            // See 401799_pre_Scene_1.swf
            // TODO: How does Flash handle this?
            return Err(Error::invalid_data("Button action length is too short"));
        };
        Ok((
            ButtonAction {
                conditions,
                action_data,
            },
            length != 0,
        ))
    }

    pub fn read_csm_text_settings(&mut self) -> Result<CsmTextSettings> {
        let id = self.read_character_id()?;
        let flags = self.read_u8()?;
        let thickness = self.read_f32()?;
        let sharpness = self.read_f32()?;
        self.read_u8()?; // Reserved (0).
        Ok(CsmTextSettings {
            id,
            use_advanced_rendering: flags & 0b01000000 != 0,
            grid_fit: TextGridFit::from_u8((flags >> 3) & 0b11)
                .ok_or_else(|| Error::invalid_data("Invalid text grid fitting"))?,
            thickness,
            sharpness,
        })
    }

    pub fn read_frame_label(&mut self) -> Result<FrameLabel<'a>> {
        let label = self.read_str()?;
        let is_anchor = self.version >= 6 && self.read_u8().unwrap_or_default() != 0;
        Ok(FrameLabel { label, is_anchor })
    }

    pub fn read_define_scene_and_frame_label_data(
        &mut self,
    ) -> Result<DefineSceneAndFrameLabelData<'a>> {
        let num_scenes = self.read_encoded_u32()? as usize;
        let mut scenes = Vec::with_capacity(num_scenes);
        for _ in 0..num_scenes {
            scenes.push(FrameLabelData {
                frame_num: self.read_encoded_u32()?,
                label: self.read_str()?,
            });
        }

        let num_frame_labels = self.read_encoded_u32()? as usize;
        let mut frame_labels = Vec::with_capacity(num_frame_labels);
        for _ in 0..num_frame_labels {
            frame_labels.push(FrameLabelData {
                frame_num: self.read_encoded_u32()?,
                label: self.read_str()?,
            });
        }

        Ok(DefineSceneAndFrameLabelData {
            scenes,
            frame_labels,
        })
    }

    pub fn read_define_font_1(&mut self) -> Result<FontV1> {
        let id = self.read_u16()?;
        let num_glyphs = self.read_u16()? / 2;

        for _ in 0..num_glyphs.saturating_sub(1) {
            self.read_u16()?;
        }

        let mut glyphs = Vec::with_capacity(num_glyphs as usize);
        for _ in 0..num_glyphs {
            let mut glyph = vec![];
            let num_bits = self.read_u8()?;
            let mut shape_context = ShapeContext {
                swf_version: self.version,
                shape_version: 1,
                num_fill_bits: num_bits >> 4,
                num_line_bits: num_bits & 0b1111,
            };
            let mut bits = self.bits();
            while let Some(record) = Self::read_shape_record(&mut bits, &mut shape_context)? {
                glyph.push(record);
            }
            glyphs.push(glyph);
        }

        Ok(FontV1 { id, glyphs })
    }

    pub fn read_define_font_2(&mut self, version: u8) -> Result<Font<'a>> {
        let id = self.read_character_id()?;
        let flags = FontFlag::from_bits_truncate(self.read_u8()?);
        let language = self.read_language()?;
        // SWF19 states that the font name should not have a terminating null byte,
        // but it often does (depends on Flash IDE version?)
        let name = self.read_str_with_len()?;

        let num_glyphs = self.read_u16()? as usize;
        let mut glyphs = vec![
            Glyph {
                shape_records: vec![],
                code: 0,
                advance: 0,
                bounds: None,
            };
            num_glyphs
        ];

        // SWF19 p. 164 doesn't make it super clear: If there are no glyphs,
        // then the following tables are omitted. But the table offset values
        // may or may not be written... (depending on Flash IDE version that was used?)
        if num_glyphs == 0 {
            // Try to read the CodeTableOffset. It may or may not be present,
            // so just dump any error.
            if flags.contains(FontFlag::HAS_WIDE_OFFSETS) {
                let _ = self.read_u32();
            } else {
                let _ = self.read_u16();
            }
        } else {
            let offsets_ref = self.get_ref();

            // OffsetTable
            let offsets: Result<Vec<_>> = (0..num_glyphs)
                .map(|_| {
                    if flags.contains(FontFlag::HAS_WIDE_OFFSETS) {
                        self.read_u32()
                    } else {
                        self.read_u16().map(u32::from)
                    }
                })
                .collect();
            let offsets = offsets?;

            // CodeTableOffset
            let code_table_offset = if flags.contains(FontFlag::HAS_WIDE_OFFSETS) {
                self.read_u32()?
            } else {
                self.read_u16()?.into()
            };

            // GlyphShapeTable
            for (i, glyph) in glyphs.iter_mut().enumerate() {
                // The glyph shapes are assumed to be positioned per the offset table.
                // Panic on debug builds if this assumption is wrong, maybe we need to
                // seek into these offsets instead?
                debug_assert_eq!(self.pos(offsets_ref), offsets[i] as usize);

                // The glyph shapes must not overlap. Avoid exceeding to the next one.
                // TODO: What happens on decreasing offsets?
                let available_bytes = if i < num_glyphs - 1 {
                    offsets[i + 1] - offsets[i]
                } else {
                    code_table_offset - offsets[i]
                };

                if available_bytes == 0 {
                    continue;
                }

                let num_bits = self.read_u8()?;
                let mut shape_context = ShapeContext {
                    swf_version: self.version,
                    shape_version: 1,
                    num_fill_bits: num_bits >> 4,
                    num_line_bits: num_bits & 0b1111,
                };

                if available_bytes == 1 {
                    continue;
                }

                // TODO: Avoid reading more than `available_bytes - 1`?
                let mut bits = self.bits();
                while let Some(record) = Self::read_shape_record(&mut bits, &mut shape_context)? {
                    glyph.shape_records.push(record);
                }
            }

            // The code table is assumed to be positioned right after the glyph shapes.
            // Panic on debug builds if this assumption is wrong, maybe we need to seek
            // into the code table offset instead?
            debug_assert_eq!(self.pos(offsets_ref), code_table_offset as usize);

            // CodeTable
            for glyph in &mut glyphs {
                glyph.code = if flags.contains(FontFlag::HAS_WIDE_CODES) {
                    self.read_u16()?
                } else {
                    self.read_u8()?.into()
                };
            }
        }

        // TODO: Is it possible to have a layout when there are no glyphs?
        let layout = if flags.contains(FontFlag::HAS_LAYOUT) {
            let ascent = self.read_u16()?;
            let descent = self.read_u16()?;
            let leading = self.read_i16()?;

            for glyph in &mut glyphs {
                glyph.advance = self.read_i16()?;
            }

            // Some older SWFs end the tag here, as this data isn't used until v7.
            if !self.as_slice().is_empty() {
                for glyph in &mut glyphs {
                    glyph.bounds = Some(self.read_rectangle()?);
                }
            }

            let kerning_records = if !self.as_slice().is_empty() {
                let num_kerning_records = self.read_u16()? as usize;

                let mut kerning_records = Vec::with_capacity(num_kerning_records);
                for _ in 0..num_kerning_records {
                    kerning_records
                        .push(self.read_kerning_record(flags.contains(FontFlag::HAS_WIDE_CODES))?);
                }
                kerning_records
            } else {
                vec![]
            };

            Some(FontLayout {
                ascent,
                descent,
                leading,
                kerning: kerning_records,
            })
        } else {
            None
        };

        Ok(Font {
            version,
            id,
            name,
            language,
            layout,
            glyphs,
            flags,
        })
    }

    pub fn read_define_font_4(&mut self) -> Result<Font4<'a>> {
        let id = self.read_character_id()?;
        let flags = self.read_u8()?;
        let name = self.read_str()?;
        let has_font_data = flags & 0b100 != 0;
        let data = if has_font_data {
            Some(self.read_slice_to_end())
        } else {
            None
        };
        Ok(Font4 {
            id,
            is_italic: flags & 0b10 != 0,
            is_bold: flags & 0b1 != 0,
            name,
            data,
        })
    }

    fn read_kerning_record(&mut self, has_wide_codes: bool) -> Result<KerningRecord> {
        Ok(KerningRecord {
            left_code: if has_wide_codes {
                self.read_u16()?
            } else {
                self.read_u8()?.into()
            },
            right_code: if has_wide_codes {
                self.read_u16()?
            } else {
                self.read_u8()?.into()
            },
            adjustment: Twips::new(self.read_i16()?.into()),
        })
    }

    fn read_define_font_align_zones(&mut self) -> Result<Tag<'a>> {
        let id = self.read_character_id()?;
        let thickness = FontThickness::from_u8(self.read_u8()? >> 6)
            .ok_or_else(|| Error::invalid_data("Invalid font thickness type."))?;
        let mut zones = vec![];
        while let Ok(zone) = self.read_font_align_zone() {
            zones.push(zone);
        }
        Ok(Tag::DefineFontAlignZones {
            id,
            thickness,
            zones,
        })
    }

    fn read_font_align_zone(&mut self) -> Result<FontAlignZone> {
        self.read_u8()?; // Always 2.
        let zone = FontAlignZone {
            left: self.read_i16()?,
            width: self.read_i16()?,
            bottom: self.read_i16()?,
            height: self.read_i16()?,
        };
        self.read_u8()?; // Always 0b000000_11 (2 dimensions).
        Ok(zone)
    }

    fn read_define_font_info(&mut self, version: u8) -> Result<FontInfo<'a>> {
        let id = self.read_u16()?;
        let name = self.read_str_with_len()?;
        let flags = FontInfoFlag::from_bits_truncate(self.read_u8()?);

        let language = if version >= 2 {
            self.read_language()?
        } else {
            Language::Unknown
        };

        let mut code_table = vec![];
        if flags.contains(FontInfoFlag::HAS_WIDE_CODES) {
            while let Ok(code) = self.read_u16() {
                code_table.push(code);
            }
        } else {
            // TODO(Herschel): Warn for version 2.
            while let Ok(code) = self.read_u8() {
                code_table.push(code.into());
            }
        }

        // SWF19 has ANSI and Shift-JIS backwards?
        Ok(FontInfo {
            id,
            version,
            name,
            flags,
            language,
            code_table,
        })
    }

    fn read_define_font_name(&mut self) -> Result<Tag<'a>> {
        Ok(Tag::DefineFontName {
            id: self.read_character_id()?,
            name: self.read_str()?,
            copyright_info: self.read_str()?,
        })
    }

    pub fn read_define_morph_shape(&mut self, version: u8) -> Result<DefineMorphShape> {
        let id = self.read_character_id()?;
        let start_shape_bounds = self.read_rectangle()?;
        let end_shape_bounds = self.read_rectangle()?;
        let start_edge_bounds;
        let end_edge_bounds;
        let flags;
        if version >= 2 {
            start_edge_bounds = self.read_rectangle()?;
            end_edge_bounds = self.read_rectangle()?;
            flags = DefineMorphShapeFlag::from_bits_truncate(self.read_u8()?);
        } else {
            start_edge_bounds = start_shape_bounds.clone();
            end_edge_bounds = end_shape_bounds.clone();
            flags = DefineMorphShapeFlag::HAS_NON_SCALING_STROKES;
        }

        self.read_u32()?; // Offset to EndEdges.

        let num_fill_styles = match self.read_u8()? {
            0xff => self.read_u16()? as usize,
            n => n as usize,
        };
        let mut start_fill_styles = Vec::with_capacity(num_fill_styles);
        let mut end_fill_styles = Vec::with_capacity(num_fill_styles);
        for _ in 0..num_fill_styles {
            let (start, end) = self.read_morph_fill_style()?;
            start_fill_styles.push(start);
            end_fill_styles.push(end);
        }

        let num_line_styles = match self.read_u8()? {
            0xff => self.read_u16()? as usize,
            n => n as usize,
        };
        let mut start_line_styles = Vec::with_capacity(num_line_styles);
        let mut end_line_styles = Vec::with_capacity(num_line_styles);
        for _ in 0..num_line_styles {
            let (start, end) = self.read_morph_line_style(version)?;
            start_line_styles.push(start);
            end_line_styles.push(end);
        }

        // TODO(Herschel): Add read_shape
        let swf_version = self.version;
        let mut bits = self.bits();
        let mut shape_context = ShapeContext {
            swf_version,
            shape_version: version,
            num_fill_bits: bits.read_ubits(4)? as u8,
            num_line_bits: bits.read_ubits(4)? as u8,
        };
        let mut start_shape = Vec::new();
        while let Some(record) = Self::read_shape_record(&mut bits, &mut shape_context)? {
            start_shape.push(record);
        }

        let mut end_shape = Vec::new();
        self.read_u8()?; // NumFillBits and NumLineBits are written as 0 for the end shape.
        let mut shape_context = ShapeContext {
            swf_version: self.version,
            shape_version: version,
            num_fill_bits: 0,
            num_line_bits: 0,
        };
        let mut bits = self.bits();
        while let Some(record) = Self::read_shape_record(&mut bits, &mut shape_context)? {
            end_shape.push(record);
        }

        Ok(DefineMorphShape {
            id,
            version,
            flags,
            start: MorphShape {
                shape_bounds: start_shape_bounds,
                edge_bounds: start_edge_bounds,
                shape: start_shape,
                fill_styles: start_fill_styles,
                line_styles: start_line_styles,
            },
            end: MorphShape {
                shape_bounds: end_shape_bounds,
                edge_bounds: end_edge_bounds,
                shape: end_shape,
                fill_styles: end_fill_styles,
                line_styles: end_line_styles,
            },
        })
    }

    fn read_morph_line_style(&mut self, shape_version: u8) -> Result<(LineStyle, LineStyle)> {
        let start_width = Twips::new(self.read_u16()?.into());
        let end_width = Twips::new(self.read_u16()?.into());
        if shape_version < 2 {
            let start_color = self.read_rgba()?;
            let end_color = self.read_rgba()?;

            Ok((
                LineStyle::new()
                    .with_width(start_width)
                    .with_color(start_color),
                LineStyle::new().with_width(end_width).with_color(end_color),
            ))
        } else {
            // MorphLineStyle2 in DefineMorphShape2.
            let mut flags = LineStyleFlag::from_bits_retain(self.read_u16()?);
            // Verify valid cap and join styles.
            if flags.contains(LineStyleFlag::JOIN_STYLE) {
                log::warn!("Invalid line join style");
                flags -= LineStyleFlag::JOIN_STYLE;
            }
            if flags.contains(LineStyleFlag::START_CAP_STYLE) {
                log::warn!("Invalid line start cap style");
                flags -= LineStyleFlag::START_CAP_STYLE;
            }
            if flags.contains(LineStyleFlag::END_CAP_STYLE) {
                log::warn!("Invalid line end cap style");
                flags -= LineStyleFlag::END_CAP_STYLE;
            }
            let miter_limit = if flags & LineStyleFlag::JOIN_STYLE == LineStyleFlag::MITER {
                self.read_fixed8()?
            } else {
                Fixed8::ZERO
            };
            let (start_fill_style, end_fill_style) = if flags.contains(LineStyleFlag::HAS_FILL) {
                let (start, end) = self.read_morph_fill_style()?;
                (start, end)
            } else {
                (
                    FillStyle::Color(self.read_rgba()?),
                    FillStyle::Color(self.read_rgba()?),
                )
            };
            Ok((
                LineStyle {
                    width: start_width,
                    fill_style: start_fill_style,
                    flags,
                    miter_limit,
                },
                LineStyle {
                    width: end_width,
                    fill_style: end_fill_style,
                    flags,
                    miter_limit,
                },
            ))
        }
    }

    fn read_morph_fill_style(&mut self) -> Result<(FillStyle, FillStyle)> {
        let fill_style_type = self.read_u8()?;
        let fill_style = match fill_style_type {
            0x00 => {
                let start_color = self.read_rgba()?;
                let end_color = self.read_rgba()?;
                (FillStyle::Color(start_color), FillStyle::Color(end_color))
            }

            0x10 => {
                let (start_gradient, end_gradient) = self.read_morph_gradient()?;
                (
                    FillStyle::LinearGradient(start_gradient),
                    FillStyle::LinearGradient(end_gradient),
                )
            }

            0x12 => {
                let (start_gradient, end_gradient) = self.read_morph_gradient()?;
                (
                    FillStyle::RadialGradient(start_gradient),
                    FillStyle::RadialGradient(end_gradient),
                )
            }

            0x13 => {
                // SWF19 says focal gradients are only allowed in SWFv8+ and DefineMorphShapeShape2,
                // but it works even in earlier tags (#2730).
                // TODO(Herschel): How is focal_point stored?
                let (start_gradient, end_gradient) = self.read_morph_gradient()?;
                let start_focal_point = self.read_fixed8()?;
                let end_focal_point = self.read_fixed8()?;
                (
                    FillStyle::FocalGradient {
                        gradient: start_gradient,
                        focal_point: start_focal_point,
                    },
                    FillStyle::FocalGradient {
                        gradient: end_gradient,
                        focal_point: end_focal_point,
                    },
                )
            }

            0x40..=0x43 => {
                let id = self.read_character_id()?;
                (
                    FillStyle::Bitmap {
                        id,
                        matrix: self.read_matrix()?,
                        is_smoothed: (fill_style_type & 0b10) == 0,
                        is_repeating: (fill_style_type & 0b01) == 0,
                    },
                    FillStyle::Bitmap {
                        id,
                        matrix: self.read_matrix()?,
                        is_smoothed: (fill_style_type & 0b10) == 0,
                        is_repeating: (fill_style_type & 0b01) == 0,
                    },
                )
            }

            _ => return Err(Error::invalid_data("Invalid fill style.")),
        };
        Ok(fill_style)
    }

    fn read_morph_gradient(&mut self) -> Result<(Gradient, Gradient)> {
        let start_matrix = self.read_matrix()?;
        let end_matrix = self.read_matrix()?;
        let (num_records, spread, interpolation) = self.read_gradient_flags()?;
        let mut start_records = Vec::with_capacity(num_records);
        let mut end_records = Vec::with_capacity(num_records);
        for _ in 0..num_records {
            start_records.push(GradientRecord {
                ratio: self.read_u8()?,
                color: self.read_rgba()?,
            });
            end_records.push(GradientRecord {
                ratio: self.read_u8()?,
                color: self.read_rgba()?,
            });
        }
        Ok((
            Gradient {
                matrix: start_matrix,
                spread,
                interpolation,
                records: start_records,
            },
            Gradient {
                matrix: end_matrix,
                spread,
                interpolation,
                records: end_records,
            },
        ))
    }

    pub fn read_define_shape(&mut self, version: u8) -> Result<Shape> {
        let id = self.read_u16()?;
        let shape_bounds = self.read_rectangle()?;
        let edge_bounds;
        let flags;
        if version >= 4 {
            edge_bounds = self.read_rectangle()?;
            flags = ShapeFlag::from_bits_truncate(self.read_u8()?);
        } else {
            edge_bounds = shape_bounds.clone();
            flags = ShapeFlag::HAS_NON_SCALING_STROKES;
        }

        let (styles, num_fill_bits, num_line_bits) = self.read_shape_styles(version)?;
        let mut records = Vec::new();
        let mut shape_context = ShapeContext {
            swf_version: self.version,
            shape_version: version,
            num_fill_bits,
            num_line_bits,
        };
        let mut bits = self.bits();
        while let Some(record) = Self::read_shape_record(&mut bits, &mut shape_context)? {
            records.push(record);
        }

        Ok(Shape {
            version,
            id,
            shape_bounds,
            edge_bounds,
            flags,
            styles,
            shape: records,
        })
    }

    pub fn read_define_sound(&mut self) -> Result<Sound<'a>> {
        let id = self.read_u16()?;
        let format = self.read_sound_format()?;
        let num_samples = self.read_u32()?;
        let data = self.read_slice_to_end();
        Ok(Sound {
            id,
            format,
            num_samples,
            data,
        })
    }

    pub fn read_sound_stream_head(&mut self) -> Result<SoundStreamHead> {
        // TODO: Verify version requirements.
        let playback_format = self.read_sound_format()?;
        let stream_format = self.read_sound_format()?;
        let num_samples_per_block = self.read_u16()?;
        let latency_seek = if stream_format.compression == AudioCompression::Mp3 {
            // SWF19 says latency seek is i16, not u16. Is this wrong> How are negative values used?
            // Some software creates SWF files that incorrectly omit this value.
            // Fail silently if it's missing.
            // TODO: What is Flash's behavior in this case? Does it read the value from the following bytes?
            self.read_i16().unwrap_or(0)
        } else {
            0
        };
        Ok(SoundStreamHead {
            stream_format,
            playback_format,
            num_samples_per_block,
            latency_seek,
        })
    }

    fn read_shape_styles(&mut self, shape_version: u8) -> Result<(ShapeStyles, u8, u8)> {
        let num_fill_styles = match self.read_u8()? {
            0xff if shape_version >= 2 => self.read_u16()? as usize,
            n => n as usize,
        };
        let mut fill_styles = Vec::with_capacity(num_fill_styles);
        for _ in 0..num_fill_styles {
            fill_styles.push(self.read_fill_style(shape_version)?);
        }

        let num_line_styles = match self.read_u8()? {
            // TODO: is this true for linestyles too? SWF19 says not.
            0xff if shape_version >= 2 => self.read_u16()? as usize,
            n => n as usize,
        };
        let mut line_styles = Vec::with_capacity(num_line_styles);
        for _ in 0..num_line_styles {
            line_styles.push(self.read_line_style(shape_version)?);
        }

        let num_bits = self.read_u8()?;
        Ok((
            ShapeStyles {
                fill_styles,
                line_styles,
            },
            num_bits >> 4,
            num_bits & 0b1111,
        ))
    }

    fn read_fill_style(&mut self, shape_version: u8) -> Result<FillStyle> {
        let fill_style_type = self.read_u8()?;
        let fill_style = match fill_style_type {
            0x00 => {
                let color = if shape_version >= 3 {
                    self.read_rgba()?
                } else {
                    self.read_rgb()?
                };
                FillStyle::Color(color)
            }

            0x10 => {
                if let Some(gradient) = self.read_gradient(shape_version)? {
                    FillStyle::LinearGradient(gradient)
                } else {
                    FillStyle::Color(Color::BLACK)
                }
            }

            0x12 => {
                if let Some(gradient) = self.read_gradient(shape_version)? {
                    FillStyle::RadialGradient(gradient)
                } else {
                    FillStyle::Color(Color::BLACK)
                }
            }

            0x13 => {
                // SWF19 says focal gradients are only allowed in SWFv8+ and DefineShape4,
                // but it works even in earlier tags (#2730).
                let gradient = self.read_gradient(shape_version)?;
                let focal_point = self.read_fixed8()?;
                if let Some(gradient) = gradient {
                    FillStyle::FocalGradient {
                        gradient,
                        focal_point,
                    }
                } else {
                    FillStyle::Color(Color::BLACK)
                }
            }

            0x40..=0x43 => FillStyle::Bitmap {
                id: self.read_u16()?,
                matrix: self.read_matrix()?,
                // Bitmap smoothing only occurs in SWF version 8+.
                is_smoothed: self.version >= 8 && (fill_style_type & 0b10) == 0,
                is_repeating: (fill_style_type & 0b01) == 0,
            },

            _ => return Err(Error::invalid_data("Invalid fill style.")),
        };
        Ok(fill_style)
    }

    fn read_line_style(&mut self, shape_version: u8) -> Result<LineStyle> {
        let width = Twips::new(self.read_u16()?.into());
        if shape_version < 4 {
            // LineStyle1
            let color = if shape_version >= 3 {
                self.read_rgba()?
            } else {
                self.read_rgb()?
            };
            Ok(LineStyle::new().with_width(width).with_color(color))
        } else {
            // LineStyle2 in DefineShape4
            let mut flags = LineStyleFlag::from_bits_retain(self.read_u16()?);
            // Verify valid cap and join styles.
            if flags.contains(LineStyleFlag::JOIN_STYLE) {
                log::warn!("Invalid line join style");
                flags -= LineStyleFlag::JOIN_STYLE;
            }
            if flags.contains(LineStyleFlag::START_CAP_STYLE) {
                log::warn!("Invalid line start cap style");
                flags -= LineStyleFlag::START_CAP_STYLE;
            }
            if flags.contains(LineStyleFlag::END_CAP_STYLE) {
                log::warn!("Invalid line end cap style");
                flags -= LineStyleFlag::END_CAP_STYLE;
            }
            let miter_limit = if flags & LineStyleFlag::JOIN_STYLE == LineStyleFlag::MITER {
                self.read_fixed8()?
            } else {
                Fixed8::ZERO
            };

            let fill_style = if flags.contains(LineStyleFlag::HAS_FILL) {
                self.read_fill_style(shape_version)?
            } else {
                FillStyle::Color(self.read_rgba()?)
            };
            Ok(LineStyle {
                width,
                fill_style,
                flags,
                miter_limit,
            })
        }
    }

    fn read_gradient(&mut self, shape_version: u8) -> Result<Option<Gradient>> {
        let matrix = self.read_matrix()?;
        let (num_records, spread, interpolation) = self.read_gradient_flags()?;

        // this can happen in some malformed SWFs (#4499, #4414, #3365)
        if num_records == 0 {
            return Ok(None);
        }

        let mut records = Vec::with_capacity(num_records);
        for _ in 0..num_records {
            records.push(GradientRecord {
                ratio: self.read_u8()?,
                color: if shape_version >= 3 {
                    self.read_rgba()?
                } else {
                    self.read_rgb()?
                },
            });
        }
        Ok(Some(Gradient {
            matrix,
            spread,
            interpolation,
            records,
        }))
    }

    fn read_gradient_flags(&mut self) -> Result<(usize, GradientSpread, GradientInterpolation)> {
        let flags = self.read_u8()?;
        let spread = GradientSpread::from_u8((flags >> 6) & 0b11)
            .ok_or_else(|| Error::invalid_data("Invalid gradient spread mode"))?;
        let interpolation = GradientInterpolation::from_u8((flags >> 4) & 0b11)
            .ok_or_else(|| Error::invalid_data("Invalid gradient interpolation mode"))?;
        let num_records: usize = (flags & 0b1111).into();
        Ok((num_records, spread, interpolation))
    }

    fn read_shape_record(
        bits: &mut BitReader<'_, '_>,
        context: &mut ShapeContext,
    ) -> Result<Option<ShapeRecord>> {
        let is_edge_record = bits.read_bit()?;
        let shape_record = if is_edge_record {
            let is_straight_edge = bits.read_bit()?;
            let num_bits = bits.read_ubits(4)? + 2;
            if is_straight_edge {
                // StraightEdge
                let is_axis_aligned = !bits.read_bit()?;
                let is_vertical = is_axis_aligned && bits.read_bit()?;
                let delta_x = if !is_axis_aligned || !is_vertical {
                    bits.read_sbits_twips(num_bits)?
                } else {
                    Twips::ZERO
                };
                let delta_y = if !is_axis_aligned || is_vertical {
                    bits.read_sbits_twips(num_bits)?
                } else {
                    Twips::ZERO
                };
                Some(ShapeRecord::StraightEdge {
                    delta: PointDelta::new(delta_x, delta_y),
                })
            } else {
                // CurvedEdge
                let control_delta_x = bits.read_sbits_twips(num_bits)?;
                let control_delta_y = bits.read_sbits_twips(num_bits)?;
                let anchor_delta_x = bits.read_sbits_twips(num_bits)?;
                let anchor_delta_y = bits.read_sbits_twips(num_bits)?;
                Some(ShapeRecord::CurvedEdge {
                    control_delta: PointDelta::new(control_delta_x, control_delta_y),
                    anchor_delta: PointDelta::new(anchor_delta_x, anchor_delta_y),
                })
            }
        } else {
            let flags = ShapeRecordFlag::from_bits_truncate(bits.read_ubits(5)? as u8);
            if !flags.is_empty() {
                // StyleChange
                let num_fill_bits = context.num_fill_bits as u32;
                let num_line_bits = context.num_line_bits as u32;
                let move_to = if flags.contains(ShapeRecordFlag::MOVE_TO) {
                    let num_bits = bits.read_ubits(5)?;
                    let move_x = bits.read_sbits_twips(num_bits)?;
                    let move_y = bits.read_sbits_twips(num_bits)?;
                    Some(Point::new(move_x, move_y))
                } else {
                    None
                };
                let fill_style_0 = if flags.contains(ShapeRecordFlag::FILL_STYLE_0) {
                    Some(bits.read_ubits(num_fill_bits)?)
                } else {
                    None
                };
                let fill_style_1 = if flags.contains(ShapeRecordFlag::FILL_STYLE_1) {
                    Some(bits.read_ubits(num_fill_bits)?)
                } else {
                    None
                };
                let line_style = if flags.contains(ShapeRecordFlag::LINE_STYLE) {
                    Some(bits.read_ubits(num_line_bits)?)
                } else {
                    None
                };
                // The spec says that StyleChangeRecord can only occur in DefineShape2+,
                // but SWFs in the wild exist with them in DefineShape1 (generated by third party tools),
                // and these run correctly in the Flash Player.
                let new_styles = if flags.contains(ShapeRecordFlag::NEW_STYLES) {
                    let mut reader = Reader::new(bits.reader(), context.swf_version);
                    let (new_styles, num_fill_bits, num_line_bits) =
                        reader.read_shape_styles(context.shape_version)?;
                    *bits.reader() = reader.input;
                    context.num_fill_bits = num_fill_bits;
                    context.num_line_bits = num_line_bits;
                    Some(new_styles)
                } else {
                    None
                };
                Some(ShapeRecord::StyleChange(Box::new(StyleChangeData {
                    move_to,
                    fill_style_0,
                    fill_style_1,
                    line_style,
                    new_styles,
                })))
            } else {
                // EndShapeRecord
                None
            }
        };
        Ok(shape_record)
    }

    pub fn read_define_sprite(&mut self) -> Result<Sprite<'a>> {
        Ok(Sprite {
            id: self.read_u16()?,
            num_frames: self.read_u16()?,
            tags: self.read_tag_list()?,
        })
    }

    pub fn read_file_attributes(&mut self) -> Result<FileAttributes> {
        let flags = self.read_u32()?;
        Ok(FileAttributes::from_bits_truncate(flags as u8))
    }

    pub fn read_export_assets(&mut self) -> Result<ExportAssets<'a>> {
        let num_exports = self.read_u16()?;
        let mut exports = Vec::with_capacity(num_exports.into());
        for _ in 0..num_exports {
            exports.push(ExportedAsset {
                id: self.read_u16()?,
                name: self.read_str()?,
            });
        }
        Ok(exports)
    }

    pub fn read_import_assets(&mut self) -> Result<(&'a SwfStr, ExportAssets<'a>)> {
        let url = self.read_str()?;
        let num_imports = self.read_u16()?;
        let mut imports = Vec::with_capacity(num_imports as usize);
        for _ in 0..num_imports {
            imports.push(ExportedAsset {
                id: self.read_u16()?,
                name: self.read_str()?,
            });
        }

        Ok((url, imports))
    }

    pub fn read_import_assets_2(&mut self) -> Result<(&'a SwfStr, ExportAssets<'a>)> {
        let url = self.read_str()?;
        self.read_u8()?; // Reserved; must be 1
        self.read_u8()?; // Reserved; must be 0
        let num_imports = self.read_u16()?;
        let mut imports = Vec::with_capacity(num_imports as usize);

        for _ in 0..num_imports {
            imports.push(ExportedAsset {
                id: self.read_u16()?,
                name: self.read_str()?,
            });
        }

        Ok((url, imports))
    }

    pub fn read_place_object(&mut self) -> Result<PlaceObject<'a>> {
        Ok(PlaceObject {
            version: 1,
            action: PlaceObjectAction::Place(self.read_u16()?),
            depth: self.read_u16()?,
            matrix: Some(self.read_matrix()?),
            color_transform: if !self.get_ref().is_empty() {
                Some(self.read_color_transform(false)?)
            } else {
                None
            },
            ratio: None,
            name: None,
            clip_depth: None,
            class_name: None,
            filters: None,
            background_color: None,
            blend_mode: None,
            clip_actions: None,
            has_image: false,
            is_bitmap_cached: None,
            is_visible: None,
            amf_data: None,
        })
    }

    pub fn read_place_object_2_or_3(
        &mut self,
        place_object_version: u8,
    ) -> Result<PlaceObject<'a>> {
        let flags = if place_object_version >= 3 {
            PlaceFlag::from_bits_truncate(self.read_u16()?)
        } else {
            PlaceFlag::from_bits_truncate(self.read_u8()?.into())
        };

        let depth = self.read_u16()?;

        // PlaceObject3
        // SWF19 p.40 incorrectly says class name if (HasClassNameFlag || (HasImage && HasCharacterID))
        // I think this should be if (HasClassNameFlag || (HasImage && !HasCharacterID)),
        // you use the class name only if a character ID isn't present.
        // But what is the case where we'd have an image without either HasCharacterID or HasClassName set?
        let has_image = flags.contains(PlaceFlag::HAS_IMAGE);
        let has_character_id = flags.contains(PlaceFlag::HAS_CHARACTER);
        let has_class_name =
            flags.contains(PlaceFlag::HAS_CLASS_NAME) || (has_image && !has_character_id);
        let class_name = if has_class_name {
            Some(self.read_str()?)
        } else {
            None
        };

        let action = match (flags.contains(PlaceFlag::MOVE), has_character_id) {
            (true, false) => PlaceObjectAction::Modify,
            (false, true) => {
                let id = self.read_u16()?;
                PlaceObjectAction::Place(id)
            }
            (true, true) => {
                let id = self.read_u16()?;
                PlaceObjectAction::Replace(id)
            }
            _ => return Err(Error::invalid_data("Invalid PlaceObject type")),
        };
        let matrix = if flags.contains(PlaceFlag::HAS_MATRIX) {
            Some(self.read_matrix()?)
        } else {
            None
        };
        let color_transform = if flags.contains(PlaceFlag::HAS_COLOR_TRANSFORM) {
            Some(self.read_color_transform(true)?)
        } else {
            None
        };
        let ratio = if flags.contains(PlaceFlag::HAS_RATIO) {
            Some(self.read_u16()?)
        } else {
            None
        };
        let name = if flags.contains(PlaceFlag::HAS_NAME) {
            Some(self.read_str()?)
        } else {
            None
        };
        let clip_depth = if flags.contains(PlaceFlag::HAS_CLIP_DEPTH) {
            Some(self.read_u16()?)
        } else {
            None
        };

        // PlaceObject3
        let filters = if flags.contains(PlaceFlag::HAS_FILTER_LIST) {
            let num_filters = self.read_u8()?;
            let mut filters = Vec::with_capacity(num_filters as usize);
            for _ in 0..num_filters {
                filters.push(self.read_filter()?);
            }
            Some(filters)
        } else {
            None
        };
        let blend_mode = if flags.contains(PlaceFlag::HAS_BLEND_MODE) {
            Some(self.read_blend_mode()?)
        } else {
            None
        };
        let is_bitmap_cached = if flags.contains(PlaceFlag::HAS_CACHE_AS_BITMAP) {
            // Some incorrect SWFs appear to end the tag here, without
            // the expected 'u8'.
            if self.as_slice().is_empty() {
                Some(true)
            } else {
                Some(self.read_u8()? != 0)
            }
        } else {
            None
        };
        let is_visible = if flags.contains(PlaceFlag::HAS_VISIBLE) {
            Some(self.read_u8()? != 0)
        } else {
            None
        };
        let background_color = if flags.contains(PlaceFlag::OPAQUE_BACKGROUND) {
            Some(self.read_rgba()?)
        } else {
            None
        };
        let clip_actions = if flags.contains(PlaceFlag::HAS_CLIP_ACTIONS) {
            Some(self.read_clip_actions()?)
        } else {
            None
        };

        // PlaceObject4
        let amf_data = if place_object_version >= 4 {
            Some(self.read_slice_to_end())
        } else {
            None
        };

        Ok(PlaceObject {
            version: place_object_version,
            action,
            depth,
            matrix,
            color_transform,
            ratio,
            name,
            clip_depth,
            clip_actions,
            has_image,
            is_bitmap_cached,
            is_visible,
            class_name,
            filters,
            background_color,
            blend_mode,
            amf_data,
        })
    }

    pub fn read_remove_object_1(&mut self) -> Result<RemoveObject> {
        Ok(RemoveObject {
            character_id: Some(self.read_u16()?),
            depth: self.read_u16()?,
        })
    }

    pub fn read_remove_object_2(&mut self) -> Result<RemoveObject> {
        Ok(RemoveObject {
            depth: self.read_u16()?,
            character_id: None,
        })
    }

    pub fn read_blend_mode(&mut self) -> Result<BlendMode> {
        BlendMode::from_u8(self.read_u8()?).ok_or_else(|| Error::invalid_data("Invalid blend mode"))
    }

    fn read_clip_actions(&mut self) -> Result<Vec<ClipAction<'a>>> {
        self.read_u16()?; // Must be 0
        self.read_clip_event_flags(); // All event flags
        let mut clip_actions = vec![];
        while let Some(clip_action) = self.read_clip_action()? {
            clip_actions.push(clip_action);
        }
        Ok(clip_actions)
    }

    fn read_clip_action(&mut self) -> Result<Option<ClipAction<'a>>> {
        let events = self.read_clip_event_flags();
        if events.is_empty() {
            Ok(None)
        } else {
            let mut length = self.read_u32()?;
            let key_code = if events.contains(ClipEventFlag::KEY_PRESS) {
                // ActionData length includes the 1 byte key code.
                length -= 1;
                Some(self.read_u8()?)
            } else {
                None
            };
            let action_data = self.read_slice(length as usize)?;

            Ok(Some(ClipAction {
                events,
                key_code,
                action_data,
            }))
        }
    }

    fn read_clip_event_flags(&mut self) -> ClipEventFlag {
        // There are SWFs in the wild with malformed final ClipActions that is only 2 bytes
        // instead of 4 bytes (#2899). Handle this gracefully to allow the tag to run.
        // TODO: We may need a more general way to handle truncated tags, since this has
        // occurred in a few different places.
        let bits = if self.version >= 6 {
            self.read_u32().unwrap_or_default()
        } else {
            // SWF19 pp. 48-50: For SWFv5, ClipEventFlags only had 2 bytes of flags. This was
            // expanded to 4 bytes in SWFv6.
            // The 2nd byte is documented to be reserved (all 0), but it's not enforced by Flash.
            self.read_u16().unwrap_or_default().into()
        };

        ClipEventFlag::from_bits_truncate(bits)
    }

    fn read_drop_shadow_filter(&mut self) -> Result<DropShadowFilter> {
        Ok(DropShadowFilter {
            color: self.read_rgba()?,
            blur_x: self.read_fixed16()?,
            blur_y: self.read_fixed16()?,
            angle: self.read_fixed16()?,
            distance: self.read_fixed16()?,
            strength: self.read_fixed8()?,
            flags: DropShadowFilterFlags::from_bits_retain(self.read_u8()?),
        })
    }

    fn read_blur_filter(&mut self) -> Result<BlurFilter> {
        Ok(BlurFilter {
            blur_x: self.read_fixed16()?,
            blur_y: self.read_fixed16()?,
            flags: BlurFilterFlags::from_bits_retain(self.read_u8()?),
        })
    }

    fn read_glow_filter(&mut self) -> Result<GlowFilter> {
        Ok(GlowFilter {
            color: self.read_rgba()?,
            blur_x: self.read_fixed16()?,
            blur_y: self.read_fixed16()?,
            strength: self.read_fixed8()?,
            flags: GlowFilterFlags::from_bits_retain(self.read_u8()?),
        })
    }

    fn read_bevel_filter(&mut self) -> Result<BevelFilter> {
        Ok(BevelFilter {
            // Note that the color order is wrong in the spec, it's highlight then shadow.
            highlight_color: self.read_rgba()?,
            shadow_color: self.read_rgba()?,
            blur_x: self.read_fixed16()?,
            blur_y: self.read_fixed16()?,
            angle: self.read_fixed16()?,
            distance: self.read_fixed16()?,
            strength: self.read_fixed8()?,
            flags: BevelFilterFlags::from_bits_retain(self.read_u8()?),
        })
    }

    fn read_gradient_filter(&mut self) -> Result<GradientFilter> {
        let num_colors = self.read_u8()?;
        let mut colors = Vec::with_capacity(num_colors as usize);
        for _ in 0..num_colors {
            colors.push(self.read_rgba()?);
        }
        let mut gradient_records = Vec::with_capacity(num_colors as usize);
        for color in colors {
            gradient_records.push(GradientRecord {
                color,
                ratio: self.read_u8()?,
            });
        }
        Ok(GradientFilter {
            colors: gradient_records,
            blur_x: self.read_fixed16()?,
            blur_y: self.read_fixed16()?,
            angle: self.read_fixed16()?,
            distance: self.read_fixed16()?,
            strength: self.read_fixed8()?,
            flags: GradientFilterFlags::from_bits_retain(self.read_u8()?),
        })
    }

    fn read_convolution_filter(&mut self) -> Result<ConvolutionFilter> {
        let num_matrix_cols = self.read_u8()?;
        let num_matrix_rows = self.read_u8()?;
        let divisor = self.read_f32()?;
        let bias = self.read_f32()?;
        let num_entries = num_matrix_cols * num_matrix_rows;
        let mut matrix = Vec::with_capacity(num_entries as usize);
        for _ in 0..num_entries {
            matrix.push(self.read_f32()?);
        }
        Ok(ConvolutionFilter {
            num_matrix_cols,
            num_matrix_rows,
            divisor,
            bias,
            matrix,
            default_color: self.read_rgba()?,
            flags: ConvolutionFilterFlags::from_bits_truncate(self.read_u8()?),
        })
    }

    fn read_color_matrix_filter(&mut self) -> Result<ColorMatrixFilter> {
        let mut matrix = [0.0; 20];
        for m in &mut matrix {
            *m = self.read_f32()?;
        }
        Ok(ColorMatrixFilter { matrix })
    }

    pub fn read_filter(&mut self) -> Result<Filter> {
        let filter = match self.read_u8()? {
            0 => Filter::DropShadowFilter(Box::new(self.read_drop_shadow_filter()?)),
            1 => Filter::BlurFilter(Box::new(self.read_blur_filter()?)),
            2 => Filter::GlowFilter(Box::new(self.read_glow_filter()?)),
            3 => Filter::BevelFilter(Box::new(self.read_bevel_filter()?)),
            4 => Filter::GradientGlowFilter(Box::new(self.read_gradient_filter()?)),
            5 => Filter::ConvolutionFilter(Box::new(self.read_convolution_filter()?)),
            6 => Filter::ColorMatrixFilter(Box::new(self.read_color_matrix_filter()?)),
            7 => Filter::GradientBevelFilter(Box::new(self.read_gradient_filter()?)),
            _ => return Err(Error::invalid_data("Invalid filter type")),
        };
        Ok(filter)
    }

    pub fn read_sound_format(&mut self) -> Result<SoundFormat> {
        let flags = self.read_u8()?;
        let compression = AudioCompression::from_u8(flags >> 4)
            .ok_or_else(|| Error::invalid_data("Invalid audio format."))?;
        let sample_rate = match (flags & 0b11_00) >> 2 {
            0 => 5512,
            1 => 11025,
            2 => 22050,
            3 => 44100,
            _ => unreachable!(),
        };
        let is_16_bit = (flags & 0b10) != 0;
        let is_stereo = (flags & 0b1) != 0;
        Ok(SoundFormat {
            compression,
            sample_rate,
            is_stereo,
            is_16_bit,
        })
    }

    pub fn read_sound_info(&mut self) -> Result<SoundInfo> {
        let flags = self.read_u8()?;
        let event = SoundEvent::from_u8((flags >> 4) & 0b11).unwrap();
        let in_sample = if (flags & 0b1) != 0 {
            Some(self.read_u32()?)
        } else {
            None
        };
        let out_sample = if (flags & 0b10) != 0 {
            Some(self.read_u32()?)
        } else {
            None
        };
        let num_loops = if (flags & 0b100) != 0 {
            self.read_u16()?
        } else {
            1
        };
        let envelope = if (flags & 0b1000) != 0 {
            let num_points = self.read_u8()?;
            let mut envelope = SoundEnvelope::new();
            for _ in 0..num_points {
                envelope.push(SoundEnvelopePoint {
                    sample: self.read_u32()?,
                    left_volume: f32::from(self.read_u16()?) / 32768f32,
                    right_volume: f32::from(self.read_u16()?) / 32768f32,
                })
            }
            Some(envelope)
        } else {
            None
        };
        Ok(SoundInfo {
            event,
            in_sample,
            out_sample,
            num_loops,
            envelope,
        })
    }

    pub fn read_start_sound_1(&mut self) -> Result<StartSound> {
        Ok(StartSound {
            id: self.read_u16()?,
            sound_info: Box::new(self.read_sound_info()?),
        })
    }

    pub fn read_define_binary_data(&mut self) -> Result<DefineBinaryData<'a>> {
        let id = self.read_u16()?;
        self.read_u32()?; // Reserved
        let data = self.read_slice_to_end();
        Ok(DefineBinaryData { id, data })
    }

    pub fn read_define_text(&mut self, version: u8) -> Result<Text> {
        let id = self.read_character_id()?;
        let bounds = self.read_rectangle()?;
        let matrix = self.read_matrix()?;
        let num_glyph_bits = self.read_u8()?;
        let num_advance_bits = self.read_u8()?;

        let mut records = vec![];
        while let Some(record) = self.read_text_record(num_glyph_bits, num_advance_bits, version)? {
            records.push(record);
        }

        Ok(Text {
            id,
            bounds,
            matrix,
            records,
        })
    }

    fn read_text_record(
        &mut self,
        num_glyph_bits: u8,
        num_advance_bits: u8,
        version: u8,
    ) -> Result<Option<TextRecord>> {
        let flags = self.read_u8()?;

        if flags == 0 {
            // End of text records.
            return Ok(None);
        }

        let font_id = if flags & 0b1000 != 0 {
            Some(self.read_character_id()?)
        } else {
            None
        };
        let color = if flags & 0b100 != 0 {
            if version == 1 {
                Some(self.read_rgb()?)
            } else {
                Some(self.read_rgba()?)
            }
        } else {
            None
        };
        let x_offset = if flags & 0b1 != 0 {
            Some(Twips::new(self.read_i16()?.into()))
        } else {
            None
        };
        let y_offset = if flags & 0b10 != 0 {
            Some(Twips::new(self.read_i16()?.into()))
        } else {
            None
        };
        let height = if flags & 0b1000 != 0 {
            Some(Twips::new(self.read_u16()?.into()))
        } else {
            None
        };
        // TODO(Herschel): font_id and height are tied together. Merge them into a struct?
        let num_glyphs = self.read_u8()?;
        let mut glyphs = Vec::with_capacity(num_glyphs as usize);
        let mut bits = self.bits();
        for _ in 0..num_glyphs {
            glyphs.push(GlyphEntry {
                index: bits.read_ubits(num_glyph_bits.into())?,
                advance: bits.read_sbits(num_advance_bits.into())?,
            });
        }

        Ok(Some(TextRecord {
            font_id,
            color,
            x_offset,
            y_offset,
            height,
            glyphs,
        }))
    }

    pub fn read_define_edit_text(&mut self) -> Result<EditText<'a>> {
        let id = self.read_character_id()?;
        let bounds = self.read_rectangle()?;
        let flags = EditTextFlag::from_bits_truncate(self.read_u16()?);
        let font_id = if flags.contains(EditTextFlag::HAS_FONT) {
            self.read_character_id()?
        } else {
            Default::default()
        };
        let font_class = if flags.contains(EditTextFlag::HAS_FONT_CLASS) {
            self.read_str()?
        } else {
            Default::default()
        };
        let height = if flags.intersects(EditTextFlag::HAS_FONT | EditTextFlag::HAS_FONT_CLASS) {
            // SWF19 errata: The specs say this field is only present if the HasFont flag is set,
            // but it's also present when the HasFontClass flag is set.
            Twips::new(self.read_u16()?.into())
        } else {
            Twips::ZERO
        };
        let color = if flags.contains(EditTextFlag::HAS_TEXT_COLOR) {
            self.read_rgba()?
        } else {
            Color::BLACK
        };
        let max_length = if flags.contains(EditTextFlag::HAS_MAX_LENGTH) {
            self.read_u16()?
        } else {
            0
        };
        let layout = if flags.contains(EditTextFlag::HAS_LAYOUT) {
            TextLayout {
                align: TextAlign::from_u8(self.read_u8()?)
                    .ok_or_else(|| Error::invalid_data("Invalid edit text alignment"))?,
                left_margin: Twips::new(self.read_u16()?.into()),
                right_margin: Twips::new(self.read_u16()?.into()),
                indent: Twips::new(self.read_u16()?.into()),
                leading: Twips::new(self.read_i16()?.into()),
            }
        } else {
            TextLayout {
                align: TextAlign::Left,
                left_margin: Twips::ZERO,
                right_margin: Twips::ZERO,
                indent: Twips::ZERO,
                leading: Twips::ZERO,
            }
        };
        let variable_name = self.read_str()?;
        let initial_text = if flags.contains(EditTextFlag::HAS_TEXT) {
            self.read_str()?
        } else {
            Default::default()
        };
        Ok(EditText {
            id,
            bounds,
            font_id,
            font_class,
            height,
            color,
            max_length,
            layout,
            variable_name,
            initial_text,
            flags,
        })
    }

    pub fn read_define_video_stream(&mut self) -> Result<DefineVideoStream> {
        let id = self.read_character_id()?;
        let num_frames = self.read_u16()?;
        let width = self.read_u16()?;
        let height = self.read_u16()?;
        let flags = self.read_u8()?;
        // TODO(Herschel): Check SWF version.
        let codec = VideoCodec::from_u8(self.read_u8()?)
            .ok_or_else(|| Error::invalid_data("Invalid video codec."))?;
        Ok(DefineVideoStream {
            id,
            num_frames,
            width,
            height,
            is_smoothed: flags & 0b1 != 0,
            codec,
            deblocking: VideoDeblocking::from_u8((flags >> 1) & 0b111)
                .ok_or_else(|| Error::invalid_data("Invalid video deblocking value."))?,
        })
    }

    pub fn read_video_frame(&mut self) -> Result<VideoFrame<'a>> {
        let stream_id = self.read_character_id()?;
        let frame_num = self.read_u16()?;
        let data = self.read_slice_to_end();
        Ok(VideoFrame {
            stream_id,
            frame_num,
            data,
        })
    }

    fn read_define_bits_jpeg_3(&mut self, version: u8) -> Result<DefineBitsJpeg3<'a>> {
        let id = self.read_character_id()?;
        let data_size = self.read_u32()? as usize;
        let deblocking = if version >= 4 {
            self.read_fixed8()?
        } else {
            Fixed8::ZERO
        };
        let data = self.read_slice(data_size)?;
        let alpha_data = self.read_slice_to_end();
        Ok(DefineBitsJpeg3 {
            id,
            version,
            deblocking,
            data,
            alpha_data,
        })
    }

    pub fn read_define_bits_lossless(&mut self, version: u8) -> Result<DefineBitsLossless<'a>> {
        let id = self.read_character_id()?;
        let format = self.read_u8()?;
        let width = self.read_u16()?;
        let height = self.read_u16()?;
        let format = match format {
            3 => BitmapFormat::ColorMap8 {
                num_colors: self.read_u8()?,
            },
            4 if version == 1 => BitmapFormat::Rgb15,
            5 => BitmapFormat::Rgb32,
            _ => return Err(Error::invalid_data("Invalid bitmap format.")),
        };
        let data = self.read_slice_to_end();
        Ok(DefineBitsLossless {
            version,
            id,
            format,
            width,
            height,
            data: Cow::Borrowed(data),
        })
    }

    pub fn read_do_abc_2(&mut self) -> Result<DoAbc2<'a>> {
        let flags = DoAbc2Flag::from_bits_truncate(self.read_u32()?);
        let name = self.read_str()?;
        let data = self.read_slice_to_end();
        Ok(DoAbc2 { flags, name, data })
    }

    pub fn read_product_info(&mut self) -> Result<ProductInfo> {
        // Not documented in SWF19 reference.
        // See http://wahlers.com.br/claus/blog/undocumented-swf-tags-written-by-mxmlc/
        Ok(ProductInfo {
            product_id: self.read_u32()?,
            edition: self.read_u32()?,
            major_version: self.read_u8()?,
            minor_version: self.read_u8()?,
            build_number: self.read_u64()?,
            compilation_date: self.read_u64()?,
        })
    }

    pub fn read_debug_id(&mut self) -> Result<DebugId> {
        // Not documented in SWF19 reference.
        // See http://wahlers.com.br/claus/blog/undocumented-swf-tags-written-by-mxmlc/
        let mut debug_id = [0u8; 16];
        self.get_mut().read_exact(&mut debug_id)?;
        Ok(debug_id)
    }

    pub fn read_name_character(&mut self) -> Result<NameCharacter<'a>> {
        // Not documented in SWF19 reference, and seems to be ignored by the official Flash Player.
        // Not generated by any version of the Flash IDE, but some 3rd party tools contain it.
        // See https://www.m2osw.com/swf_tag_namecharacter
        Ok(NameCharacter {
            id: self.read_character_id()?,
            name: self.read_str()?,
        })
    }
}

pub fn read_compression_type<R: Read>(mut input: R) -> Result<Compression> {
    let mut signature = [0u8; 3];
    input.read_exact(&mut signature)?;
    let compression = match &signature {
        b"FWS" | b"GFX" => Compression::None,
        b"CWS" | b"CFX" => Compression::Zlib,
        b"ZWS" => Compression::Lzma,
        _ => return Err(Error::invalid_data("Invalid SWF")),
    };
    Ok(compression)
}

#[cfg(test)]
#[allow(clippy::unusual_byte_groupings)]
pub mod tests {
    use super::*;
    use crate::test_data;

    fn reader(data: &[u8]) -> Reader<'_> {
        let default_version = 13;
        Reader::new(data, default_version)
    }

    fn read_from_file(path: &str) -> SwfBuf {
        let data = std::fs::read(path).unwrap();
        decompress_swf(&data[..]).unwrap()
    }

    pub fn read_tag_bytes_from_file_with_index(
        path: &str,
        tag_code: TagCode,
        mut index: usize,
    ) -> Vec<u8> {
        let swf_buf = read_from_file(path);

        // Halfway parse the SWF file until we find the tag we're searching for.
        let mut reader = Reader::new(&swf_buf.data, swf_buf.header.version());
        loop {
            let tag_start = reader.get_ref();
            let (swf_tag_code, tag_len) = reader.read_tag_code_and_length().unwrap();
            let tag_end = &reader.get_ref()[tag_len..];

            let full_tag_len = tag_end.as_ptr() as usize - tag_start.as_ptr() as usize;
            let tag_data = &tag_start[..full_tag_len];

            // Skip tag data.
            *reader.get_mut() = tag_end;

            match TagCode::from_u16(swf_tag_code) {
                Some(swf_tag_code) if swf_tag_code == tag_code => {
                    if index > 0 {
                        index -= 1;
                        continue;
                    }

                    // Flash tends to export tags with the extended header even if the size
                    // would fit with the standard header.
                    // This screws up our tests, because swf-rs writes tags with the
                    // minimum header necessary.
                    // We want to easily write new tests by exporting SWFs from the Flash
                    // software, so rewrite with a standard header to match swf-rs output.
                    return if tag_len < 0b111111 && (tag_data[0] & 0b111111) == 0b111111 {
                        let mut rewritten_tag_data = Vec::with_capacity(tag_len + 2);
                        rewritten_tag_data.extend_from_slice(&tag_data[0..2]);
                        rewritten_tag_data.extend_from_slice(&tag_data[6..]);
                        rewritten_tag_data[0] = (tag_data[0] & !0b111111) | (tag_len as u8);
                        rewritten_tag_data
                    } else {
                        tag_data.to_vec()
                    };
                }
                Some(TagCode::End) => panic!("Tag not found"),
                _ => {}
            }
        }
    }

    pub fn read_tag_bytes_from_file(path: &str, tag_code: TagCode) -> Vec<u8> {
        read_tag_bytes_from_file_with_index(path, tag_code, 0)
    }

    #[test]
    fn read_swfs() {
        assert_eq!(
            read_from_file("tests/swfs/uncompressed.swf")
                .header
                .compression(),
            Compression::None
        );
        assert_eq!(
            read_from_file("tests/swfs/zlib.swf").header.compression(),
            Compression::Zlib
        );
        if cfg!(feature = "lzma") {
            assert_eq!(
                read_from_file("tests/swfs/lzma.swf").header.compression(),
                Compression::Lzma
            );
        }
    }

    #[test]
    fn read_invalid_swf() {
        let junk = [0u8; 128];
        let result = decompress_swf(&junk[..]);
        // TODO: Verify correct error.
        assert!(result.is_err());
    }

    #[test]
    fn read_compression_type() {
        assert_eq!(
            super::read_compression_type(&b"FWS"[..]).unwrap(),
            Compression::None
        );
        assert_eq!(
            super::read_compression_type(&b"CWS"[..]).unwrap(),
            Compression::Zlib
        );
        assert_eq!(
            super::read_compression_type(&b"ZWS"[..]).unwrap(),
            Compression::Lzma
        );
        super::read_compression_type(&b"ABC"[..]).unwrap_err();
    }

    #[test]
    fn read_bit() {
        let buf: &[u8] = &[0b01010101, 0b00100101];
        let mut reader = Reader::new(buf, 1);
        let mut bits = reader.bits();
        assert_eq!(
            (0..16)
                .map(|_| bits.read_bit().unwrap())
                .collect::<Vec<_>>(),
            [
                false, true, false, true, false, true, false, true, false, false, true, false,
                false, true, false, true
            ]
        );
    }

    #[test]
    fn read_ubits() {
        let buf: &[u8] = &[0b01010101, 0b00100101];
        let mut reader = Reader::new(buf, 1);
        let mut bits = reader.bits();
        assert_eq!(
            (0..8)
                .map(|_| bits.read_ubits(2).unwrap())
                .collect::<Vec<_>>(),
            [1, 1, 1, 1, 0, 2, 1, 1]
        );
    }

    #[test]
    fn read_sbits() {
        let buf: &[u8] = &[0b01010101, 0b00100101];
        let mut reader = Reader::new(buf, 1);
        let mut bits = reader.bits();
        assert_eq!(
            (0..8)
                .map(|_| bits.read_sbits(2).unwrap())
                .collect::<Vec<_>>(),
            [1, 1, 1, 1, 0, -2, 1, 1]
        );
    }

    #[test]
    fn read_fbits() {
        assert_eq!(
            Reader::new(&[0][..], 1).bits().read_fbits(5).unwrap(),
            Fixed16::ZERO
        );
        assert_eq!(
            Reader::new(&[0b01000000, 0b00000000, 0b0_0000000][..], 1)
                .bits()
                .read_fbits(17)
                .unwrap(),
            Fixed16::from_f32(0.5)
        );
        assert_eq!(
            Reader::new(&[0b10000000, 0b00000000][..], 1)
                .bits()
                .read_fbits(16)
                .unwrap(),
            Fixed16::from_f32(-0.5)
        );
    }

    #[test]
    fn read_fixed8() {
        let buf = [
            0b00000000, 0b00000000, 0b00000000, 0b00000001, 0b10000000, 0b00000110, 0b01000000,
            0b11101011,
        ];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_fixed8().unwrap(), Fixed8::from_f32(0.0));
        assert_eq!(reader.read_fixed8().unwrap(), Fixed8::from_f32(1.0));
        assert_eq!(reader.read_fixed8().unwrap(), Fixed8::from_f32(6.5));
        assert_eq!(reader.read_fixed8().unwrap(), Fixed8::from_f32(-20.75));
    }

    #[test]
    fn read_encoded_u32() {
        let read = |data: &[u8]| reader(data).read_encoded_u32().unwrap();
        assert_eq!(read(&[0]), 0);
        assert_eq!(read(&[2]), 2);
        assert_eq!(read(&[0b1_0000001, 0b0_0000001]), 129);
        assert_eq!(
            read(&[0b1_0000001, 0b1_0000001, 0b0_1100111]),
            0b1100111_0000001_0000001
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b0000_1111
            ]),
            0b1111_0000000_0000000_0000000_0000000
        );
        assert_eq!(
            read(&[
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1_0000000,
                0b1111_1111
            ]),
            0b1111_0000000_0000000_0000000_0000000
        );
    }

    #[test]
    fn read_rectangle_zero() {
        let buf = [0b00000_000];
        let mut reader = Reader::new(&buf[..], 1);
        let rectangle = reader.read_rectangle().unwrap();
        assert_eq!(
            rectangle,
            Rectangle {
                x_min: Twips::ZERO,
                y_min: Twips::ZERO,
                x_max: Twips::ZERO,
                y_max: Twips::ZERO,
            }
        );
    }

    #[test]
    fn read_rectangle_signed() {
        let buf = [0b00110_101, 0b100_01010, 0b0_101100_0, 0b10100_000];
        let mut reader = Reader::new(&buf[..], 1);
        let rectangle = reader.read_rectangle().unwrap();
        assert_eq!(
            rectangle,
            Rectangle {
                x_min: Twips::from_pixels(-1.0),
                y_min: Twips::from_pixels(-1.0),
                x_max: Twips::from_pixels(1.0),
                y_max: Twips::from_pixels(1.0),
            }
        );
    }

    #[test]
    fn read_matrix() {
        {
            let buf = [0b0_0_00001_0, 0b0_0000000];
            let mut reader = Reader::new(&buf[..], 1);
            let matrix = reader.read_matrix().unwrap();
            assert_eq!(
                matrix,
                Matrix {
                    tx: Twips::from_pixels(0.0),
                    ty: Twips::from_pixels(0.0),
                    a: Fixed16::ONE,
                    b: Fixed16::ZERO,
                    c: Fixed16::ZERO,
                    d: Fixed16::ONE,
                }
            );
        }
    }

    #[test]
    fn read_color() {
        {
            let buf = [1, 128, 255];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(
                reader.read_rgb().unwrap(),
                Color {
                    r: 1,
                    g: 128,
                    b: 255,
                    a: 255,
                }
            );
        }
        {
            let buf = [1, 128, 235, 44];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(
                reader.read_rgba().unwrap(),
                Color {
                    r: 1,
                    g: 128,
                    b: 235,
                    a: 44,
                }
            );
        }
    }

    #[test]
    fn read_string() {
        {
            let buf = b"Testing\0More testing\0\0Non-string data";
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_str().unwrap(), "Testing");
            assert_eq!(reader.read_str().unwrap(), "More testing");
            assert_eq!(reader.read_str().unwrap(), "");
            reader.read_str().unwrap_err();
        }
        {
            let mut reader = Reader::new(&[], 1);
            reader.read_str().unwrap_err();
        }
        {
            let buf = b"\0Testing";
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_str().unwrap(), "");
        }
        {
            let buf = "12🤖12\0";
            let mut reader = Reader::new(buf.as_bytes(), 1);
            assert_eq!(reader.read_str().unwrap(), "12🤖12");
        }
    }

    #[test]
    fn read_shape_styles() {}

    #[test]
    fn read_fill_style() {
        let read = |buf: &[u8], shape_version| reader(buf).read_fill_style(shape_version).unwrap();

        let fill_style = FillStyle::Color(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        });
        assert_eq!(read(&[0, 255, 0, 0], 1), fill_style);

        // DefineShape3 and 4 read RGBA colors.
        let fill_style = FillStyle::Color(Color {
            r: 255,
            g: 0,
            b: 0,
            a: 50,
        });
        assert_eq!(read(&[0, 255, 0, 0, 50], 3), fill_style);

        let fill_style = FillStyle::Bitmap {
            id: 20,
            matrix: Matrix::IDENTITY,
            is_smoothed: false,
            is_repeating: true,
        };
        assert_eq!(
            read(&[0x42, 20, 0, 0b00_00001_0, 0b0_0000000], 3),
            fill_style
        );

        let mut matrix = Matrix::IDENTITY;
        matrix.tx = Twips::from_pixels(1.0);
        let fill_style = FillStyle::Bitmap {
            id: 33,
            matrix,
            is_smoothed: false,
            is_repeating: false,
        };
        assert_eq!(
            read(&[0x43, 33, 0, 0b00_00110_0, 0b10100_000, 0b000_00000], 3),
            fill_style
        );
    }

    #[test]
    fn read_line_style() {
        // DefineShape1 and 2 read RGB colors.
        let line_style = LineStyle::new()
            .with_width(Twips::from_pixels(0.0))
            .with_color(Color::from_rgba(0xffff0000));
        assert_eq!(
            reader(&[0, 0, 255, 0, 0]).read_line_style(2).unwrap(),
            line_style
        );

        // DefineShape3 and 4 read RGBA colors.
        // let line_style = LineStyle { width: 3, color: Color { r: 1, g: 2, b: 3, a: 10 } };
        //assert_eq!(reader(&[3, 0, 1, 2, 3, 10]).read_line_style(3).unwrap(), line_style);

        // TODO: Read LineStyle2 from DefineShape4.
    }

    #[test]
    fn read_gradient() {
        // TODO
    }

    #[test]
    fn read_shape_record() {
        let read = |buf: &[u8]| {
            let mut reader = reader(buf);
            let mut context = ShapeContext {
                swf_version: reader.version,
                shape_version: 2,
                num_fill_bits: 1,
                num_line_bits: 1,
            };
            let mut bits = reader.bits();
            Reader::read_shape_record(&mut bits, &mut context)
                .unwrap()
                .unwrap()
        };

        let shape_record = ShapeRecord::StraightEdge {
            delta: PointDelta::from_pixels(1.0, 1.0),
        };
        assert_eq!(
            read(&[0b11_0100_1_0, 0b1010_0010, 0b100_00000]),
            shape_record
        );

        let shape_record = ShapeRecord::StraightEdge {
            delta: PointDelta::from_pixels(0.0, -1.0),
        };
        assert_eq!(read(&[0b11_0100_0_1, 0b101100_00]), shape_record);

        let shape_record = ShapeRecord::StraightEdge {
            delta: PointDelta::from_pixels(-1.5, 0.0),
        };
        assert_eq!(read(&[0b11_0100_0_0, 0b100010_00]), shape_record);
    }

    #[test]
    fn read_tags() {
        for (swf_version, expected_tag, tag_bytes) in test_data::tag_tests() {
            let mut reader = Reader::new(&tag_bytes[..], swf_version);
            let parsed_tag = match reader.read_tag() {
                Ok(tag) => tag,
                Err(e) => panic!("Error parsing tag: {e}"),
            };
            assert_eq!(
                parsed_tag, expected_tag,
                "Incorrectly parsed tag.\nRead:\n{parsed_tag:#?}\n\nExpected:\n{expected_tag:#?}",
            );
        }
    }

    #[test]
    fn read_tag_list() {
        {
            let buf = [0, 0];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_tag_list().unwrap(), []);
        }

        {
            let buf = [0b01_000000, 0b00000000, 0, 0];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_tag_list().unwrap(), [Tag::ShowFrame]);
        }
    }

    /// Ensure that we return an error on invalid data.
    #[test]
    fn read_invalid_tag() {
        let tag_bytes = [0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let mut reader = Reader::new(&tag_bytes[..], 5);
        match reader.read_tag() {
            Err(crate::error::Error::SwfParseError { .. }) => (),
            result => {
                panic!("Expected SwfParseError, got {result:?}");
            }
        }
    }

    /// Ensure that we can read a PlaceObject3 tag that
    /// inccorrectly omits the 'is_bitmap_cached' u8
    /// Extracted from #7098
    #[test]
    fn read_incorrect_place_object_3() {
        let place_object = Tag::PlaceObject(Box::new(PlaceObject {
            version: 3,
            action: PlaceObjectAction::Place(161),
            depth: 1,
            matrix: Some(Matrix {
                a: Fixed16::ONE,
                b: Fixed16::ZERO,
                c: Fixed16::ZERO,
                d: Fixed16::ONE,
                tx: Twips::from_pixels(0.0),
                ty: Twips::from_pixels(0.0),
            }),
            color_transform: None,
            ratio: None,
            name: None,
            clip_depth: None,
            class_name: None,
            filters: None,
            background_color: None,
            blend_mode: None,
            clip_actions: None,
            has_image: false,
            is_bitmap_cached: Some(true),
            is_visible: None,
            amf_data: None,
        }));
        assert_eq!(
            reader(&[0x88, 0x11, 0x06, 0x4, 0x01, 0x00, 0xA1, 0x00, 0x02, 0x00])
                .read_tag()
                .unwrap(),
            place_object
        );
    }
}
