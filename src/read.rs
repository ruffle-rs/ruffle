use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use num::FromPrimitive;
use std::collections::HashSet;
use std::io::{Error, ErrorKind, Read, Result};
use types::*;
use xz2::read::XzDecoder;

/// Reads SWF data from a stream.
pub fn read_swf<R: Read>(input: R) -> Result<Swf> {
    let (mut swf, mut reader) = try!(read_swf_header(input));
    swf.tags = try!(reader.read_tag_list());
    Ok(swf)
}

fn read_swf_header<'a, R: Read + 'a>(mut input: R) -> Result<(Swf, Reader<Box<Read + 'a>>)>
{
    // Read SWF header.
    let compression = try!(Reader::read_compression_type(&mut input));
    let version = try!(input.read_u8());
    let uncompressed_length = try!(input.read_u32::<LittleEndian>());

    // Now the SWF switches to a compressed stream.
    let decompressed_input: Box<Read> = match compression {
        Compression::None => Box::new(input),
        Compression::Zlib => Box::new(ZlibDecoder::new(input)),
        Compression::Lzma => {
            // Flash uses a mangled LZMA header, so we have to massage it into the normal
            // format.
            use std::io::{Cursor, Write};
            use xz2::stream::{Action, Stream};
            use byteorder::WriteBytesExt;
            try!(input.read_u32::<LittleEndian>()); // Compressed length
            let mut lzma_properties = [0u8; 5];
            try!(input.read_exact(&mut lzma_properties));
            let mut lzma_header = Cursor::new(Vec::with_capacity(13));
            try!(lzma_header.write_all(&lzma_properties));
            try!(lzma_header.write_u64::<LittleEndian>(uncompressed_length as u64));
            let mut lzma_stream = try!(Stream::new_lzma_decoder(u64::max_value()));
            try!(lzma_stream.process(&lzma_header.into_inner(), &mut [0u8; 1], Action::Run));
            Box::new(XzDecoder::new_stream(input, lzma_stream))
        }
    };

    let mut reader = Reader::new(decompressed_input, version);
    let stage_size = try!(reader.read_rectangle());
    let frame_rate = try!(reader.read_fixed8());
    let num_frames = try!(reader.read_u16());
    let swf = Swf {
        version: version,
        compression: compression,
        stage_size: stage_size,
        frame_rate: frame_rate,
        num_frames: num_frames,
        tags: vec![],
    };
    Ok((swf, reader))
}

pub struct Reader<R: Read> {
    input: R,
    version: u8,

    byte: u8,
    bit_index: u8,

    num_fill_bits: u8,
    num_line_bits: u8,
}

impl<R: Read> Reader<R> {
    pub fn new(input: R, version: u8) -> Reader<R> {
        Reader {
            input: input,
            version: version,
            byte: 0,
            bit_index: 0,
            num_fill_bits: 0,
            num_line_bits: 0,
        }
    }

    fn read_compression_type(mut input: R) -> Result<Compression> {
        let mut signature = [0u8; 3];
        try!(input.read_exact(&mut signature));
        let compression = match &signature {
            b"FWS" => Compression::None,
            b"CWS" => Compression::Zlib,
            b"ZWS" => Compression::Lzma,
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid SWF")),
        };
        Ok(compression)
    }

    fn read_rectangle(&mut self) -> Result<Rectangle> {
        self.byte_align();
        let num_bits = try!(self.read_ubits(5)) as usize;
        Ok(Rectangle {
            x_min: try!(self.read_sbits(num_bits)) as f32 / 20f32,
            x_max: try!(self.read_sbits(num_bits)) as f32 / 20f32,
            y_min: try!(self.read_sbits(num_bits)) as f32 / 20f32,
            y_max: try!(self.read_sbits(num_bits)) as f32 / 20f32,
        })
    }

    fn read_u8(&mut self) -> Result<u8> {
        self.byte_align();
        self.input.read_u8()
    }

    fn read_u16(&mut self) -> Result<u16> {
        self.byte_align();
        self.input.read_u16::<LittleEndian>()
    }

    fn read_u32(&mut self) -> Result<u32> {
        self.byte_align();
        self.input.read_u32::<LittleEndian>()
    }

    #[allow(dead_code)]
    fn read_i16(&mut self) -> Result<i16> {
        self.byte_align();
        self.input.read_i16::<LittleEndian>()
    }

    fn read_bit(&mut self) -> Result<bool> {
        if self.bit_index == 0 {
            self.byte = try!(self.input.read_u8());
            self.bit_index = 8;
        }
        self.bit_index -= 1;
        let val = self.byte & (1 << self.bit_index) != 0;
        Ok(val)
    }

    fn byte_align(&mut self) {
        self.bit_index = 0;
    }

    fn read_ubits(&mut self, num_bits: usize) -> Result<u32> {
        let mut val = 0u32;
        for _ in 0..num_bits {
            val <<= 1;
            val |= if try!(self.read_bit()) { 1 } else { 0 };
        }
        Ok(val)
    }

    fn read_sbits(&mut self, num_bits: usize) -> Result<i32> {
        if num_bits > 0 {
            self.read_ubits(num_bits).map(|n| (n as i32) << (32 - num_bits) >> (32 - num_bits))
        } else {
            Ok(0)
        }
    }

    fn read_fbits(&mut self, num_bits: usize) -> Result<f32> {
        self.read_sbits(num_bits).map(|n| (n as f32) / 65536f32)
    }

    fn read_fixed8(&mut self) -> Result<f32> {
        self.byte_align();
        self.input.read_i16::<LittleEndian>().map(|n| n as f32 / 256f32)
    }

    fn read_fixed16(&mut self) -> Result<f64> {
        self.byte_align();
        self.input.read_i32::<LittleEndian>().map(|n| n as f64 / 65536f64)
    }

    fn read_encoded_u32(&mut self) -> Result<u32> {
        let mut val = 0u32;
        for i in 0..5 {
            let byte = try!(self.read_u8());
            val |= ((byte & 0b01111111) as u32) << (i * 7);
            if byte & 0b10000000 == 0 {
                break;
            }
        }
        Ok(val)
    }

    fn read_c_string(&mut self) -> Result<String> {
        let mut bytes = Vec::new();
        loop {
            let byte = try!(self.read_u8());
            if byte == 0 {
                break;
            }
            bytes.push(byte)
        }
        // TODO: There is probably a better way to do this.
        // TODO: Verify ANSI for SWF 5 and earlier.
        String::from_utf8(bytes)
            .map_err(|_| Error::new(ErrorKind::InvalidData, "Invalid string data"))
    }

    fn read_rgb(&mut self) -> Result<Color> {
        let r = try!(self.read_u8());
        let g = try!(self.read_u8());
        let b = try!(self.read_u8());
        Ok(Color {
            r: r,
            g: g,
            b: b,
            a: 255,
        })
    }

    fn read_rgba(&mut self) -> Result<Color> {
        let r = try!(self.read_u8());
        let g = try!(self.read_u8());
        let b = try!(self.read_u8());
        let a = try!(self.read_u8());
        Ok(Color {
            r: r,
            g: g,
            b: b,
            a: a,
        })
    }

    fn read_color_transform_no_alpha(&mut self) -> Result<ColorTransform> {
        self.byte_align();
        let has_add = try!(self.read_bit());
        let has_mult = try!(self.read_bit());
        let num_bits = try!(self.read_ubits(4)) as usize;
        let mut color_transform = ColorTransform {
            r_multiply: 1f32,
            g_multiply: 1f32,
            b_multiply: 1f32,
            a_multiply: 1f32,
            r_add: 0i16,
            g_add: 0i16,
            b_add: 0i16,
            a_add: 0i16,
        };
        if has_mult {
            color_transform.r_multiply = try!(self.read_sbits(num_bits)) as f32 / 256f32;
            color_transform.g_multiply = try!(self.read_sbits(num_bits)) as f32 / 256f32;
            color_transform.b_multiply = try!(self.read_sbits(num_bits)) as f32 / 256f32;
        }
        if has_add {
            color_transform.r_add = try!(self.read_sbits(num_bits)) as i16;
            color_transform.g_add = try!(self.read_sbits(num_bits)) as i16;
            color_transform.b_add = try!(self.read_sbits(num_bits)) as i16;
        }
        Ok(color_transform)
    }

    fn read_color_transform(&mut self) -> Result<ColorTransform> {
        self.byte_align();
        let has_add = try!(self.read_bit());
        let has_mult = try!(self.read_bit());
        let num_bits = try!(self.read_ubits(4)) as usize;
        let mut color_transform = ColorTransform {
            r_multiply: 1f32,
            g_multiply: 1f32,
            b_multiply: 1f32,
            a_multiply: 1f32,
            r_add: 0i16,
            g_add: 0i16,
            b_add: 0i16,
            a_add: 0i16,
        };
        if has_mult {
            color_transform.r_multiply = try!(self.read_sbits(num_bits)) as f32 / 256f32;
            color_transform.g_multiply = try!(self.read_sbits(num_bits)) as f32 / 256f32;
            color_transform.b_multiply = try!(self.read_sbits(num_bits)) as f32 / 256f32;
            color_transform.a_multiply = try!(self.read_sbits(num_bits)) as f32 / 256f32;
        }
        if has_add {
            color_transform.r_add = try!(self.read_sbits(num_bits)) as i16;
            color_transform.g_add = try!(self.read_sbits(num_bits)) as i16;
            color_transform.b_add = try!(self.read_sbits(num_bits)) as i16;
            color_transform.a_add = try!(self.read_sbits(num_bits)) as i16;
        }
        Ok(color_transform)
    }

    fn read_matrix(&mut self) -> Result<Matrix> {
        self.byte_align();
        let mut m = Matrix::new();
        // Scale
        if try!(self.read_bit()) {
            let num_bits = try!(self.read_ubits(5)) as usize;
            m.scale_x = try!(self.read_fbits(num_bits));
            m.scale_y = try!(self.read_fbits(num_bits));
        }
        // Rotate/Skew
        if try!(self.read_bit()) {
            let num_bits = try!(self.read_ubits(5)) as usize;
            m.rotate_skew_0 = try!(self.read_fbits(num_bits));
            m.rotate_skew_1 = try!(self.read_fbits(num_bits));
        }
        // Translate (always present)
        let num_bits = try!(self.read_ubits(5)) as usize;
        m.translate_x = (try!(self.read_sbits(num_bits)) as f32) / 20f32;
        m.translate_y = (try!(self.read_sbits(num_bits)) as f32) / 20f32;
        Ok(m)
    }

    fn read_tag_list(&mut self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();
        loop {
            match self.read_tag() {
                Ok(Some(tag)) => tags.push(tag),
                Ok(None) => break,
                Err(err) => {
                    // We screwed up reading this tag in some way.
                    // TODO: We could recover more gracefully in some way.
                    // Simply throw away this tag, but keep going.
                    if cfg!(debug_assertions) {
                        panic!("Error reading tag");
                    }
                    use std::convert::From;
                    return Err(From::from(err));
                }
            };
        }
        Ok(tags)
    }

    fn read_tag(&mut self) -> Result<Option<Tag>> {
        let (tag_code, length) = try!(self.read_tag_code_and_length());

        let mut tag_reader = Reader::new(self.input.by_ref().take(length as u64), self.version);
        //let mut tag_reader = Reader::new(Box::new(self.input.by_ref().take(length as u64)) as Box<Read>, self.version);
        use tag_codes::TagCode;
        let tag = match TagCode::from_u16(tag_code) {
            Some(TagCode::End) => return Ok(None),
            Some(TagCode::ShowFrame) => Tag::ShowFrame,
            Some(TagCode::DefineBinaryData) => {
                let id = try!(tag_reader.read_u16());
                try!(tag_reader.read_u32()); // Reserved
                let mut data = Vec::with_capacity(length - 6);
                try!(tag_reader.input.read_to_end(&mut data));
                Tag::DefineBinaryData {
                    id: id,
                    data: data,
                }
            },
            Some(TagCode::DefineShape) => try!(tag_reader.read_define_shape(1)),
            Some(TagCode::DefineShape2) => try!(tag_reader.read_define_shape(2)),
            Some(TagCode::DefineShape3) => try!(tag_reader.read_define_shape(3)),
            Some(TagCode::DefineShape4) => try!(tag_reader.read_define_shape(4)),
            Some(TagCode::DefineSound) => try!(tag_reader.read_define_sound()),
            Some(TagCode::EnableTelemetry) => {
                try!(tag_reader.read_u16()); // Reserved
                let password_hash = if length > 2 {
                    let mut data = Vec::with_capacity(32);
                    data.resize(32, 0);
                    try!(tag_reader.input.read_exact(&mut data));
                    data
                } else {
                    vec![]
                };
                Tag::EnableTelemetry { password_hash: password_hash } 
            },
            Some(TagCode::ImportAssets) => {
                let url = try!(tag_reader.read_c_string());
                let num_imports = try!(tag_reader.read_u16());
                let mut imports = Vec::with_capacity(num_imports as usize);
                for _ in 0..num_imports {
                    imports.push(ExportedAsset {
                        id: try!(tag_reader.read_u16()),
                        name: try!(tag_reader.read_c_string()),
                    });
                }
                Tag::ImportAssets { url: url, imports: imports }
            },
            Some(TagCode::ImportAssets2) => {
                let url = try!(tag_reader.read_c_string());
                try!(tag_reader.read_u8()); // Reserved; must be 1
                try!(tag_reader.read_u8()); // Reserved; must be 0
                let num_imports = try!(tag_reader.read_u16());
                let mut imports = Vec::with_capacity(num_imports as usize);
                for _ in 0..num_imports {
                    imports.push(ExportedAsset {
                        id: try!(tag_reader.read_u16()),
                        name: try!(tag_reader.read_c_string()),
                    });
                }
                Tag::ImportAssets { url: url, imports: imports }
            },

            Some(TagCode::Metadata) => Tag::Metadata(try!(tag_reader.read_c_string())),

            Some(TagCode::SetBackgroundColor) => {
                Tag::SetBackgroundColor(try!(tag_reader.read_rgb()))
            },

            Some(TagCode::StartSound) => Tag::StartSound {
                id: try!(tag_reader.read_u16()),
                sound_info: Box::new(try!(tag_reader.read_sound_info())),
            },

            Some(TagCode::StartSound2) => Tag::StartSound2 {
                class_name: try!(tag_reader.read_c_string()),
                sound_info: Box::new(try!(tag_reader.read_sound_info())),
            },

            Some(TagCode::DefineScalingGrid) => Tag::DefineScalingGrid {
                id: try!(tag_reader.read_u16()),
                splitter_rect: try!(tag_reader.read_rectangle()),
            },

            Some(TagCode::DoAbc) => {
                let mut action_data = Vec::with_capacity(length);
                try!(tag_reader.input.read_to_end(&mut action_data));
                Tag::DoAbc(action_data)
            },

            Some(TagCode::DoAction) => {
                let mut action_data = Vec::with_capacity(length);
                try!(tag_reader.input.read_to_end(&mut action_data));
                Tag::DoAction(action_data)
            },

            Some(TagCode::DoInitAction) => {
                let id = try!(tag_reader.read_u16());
                let mut action_data = Vec::with_capacity(length);
                try!(tag_reader.input.read_to_end(&mut action_data));
                Tag::DoInitAction { id: id, action_data: action_data }
            },

            Some(TagCode::EnableDebugger) => Tag::EnableDebugger(try!(tag_reader.read_c_string())),
            Some(TagCode::EnableDebugger2) => {
                try!(tag_reader.read_u16()); // Reserved
                Tag::EnableDebugger(try!(tag_reader.read_c_string()))
            },

            Some(TagCode::ScriptLimits) => Tag::ScriptLimits {
                max_recursion_depth: try!(tag_reader.read_u16()),
                timeout_in_seconds: try!(tag_reader.read_u16()),
            },

            Some(TagCode::SetTabIndex) => Tag::SetTabIndex {
                depth: try!(tag_reader.read_i16()),
                tab_index: try!(tag_reader.read_u16()),
            },

            Some(TagCode::SymbolClass) => {
                let num_symbols = try!(tag_reader.read_u16());
                let mut symbols = Vec::with_capacity(num_symbols as usize);
                for _ in 0..num_symbols {
                    symbols.push(SymbolClassLink {
                        id: try!(tag_reader.read_u16()),
                        class_name: try!(tag_reader.read_c_string()),
                    });
                }
                Tag::SymbolClass(symbols)
            },

            Some(TagCode::ExportAssets) => {
                let num_exports = try!(tag_reader.read_u16());
                let mut exports = Vec::with_capacity(num_exports as usize);
                for _ in 0..num_exports {
                    exports.push(ExportedAsset {
                        id: try!(tag_reader.read_u16()),
                        name: try!(tag_reader.read_c_string()),
                    });
                }
                Tag::ExportAssets(exports)
            },

            Some(TagCode::FileAttributes) => {
                let flags = try!(tag_reader.read_u32());
                Tag::FileAttributes(FileAttributes {
                    use_direct_blit: (flags & 0b01000000) != 0,
                    use_gpu: (flags & 0b00100000) != 0,
                    has_metadata: (flags & 0b00010000) != 0,
                    is_action_script_3: (flags & 0b00001000) != 0,
                    use_network_sandbox: (flags & 0b00000001) != 0,
                })
            },

            Some(TagCode::Protect) => {
                try!(tag_reader.read_u16());  // Two null bytes? Not specified in SWF19.
                Tag::Protect(
                    if length > 0 { Some(try!(tag_reader.read_c_string())) } else { None }
                )
            },

            Some(TagCode::DefineSceneAndFrameLabelData) => {
                try!(tag_reader.read_define_scene_and_frame_label_data())
            },

            Some(TagCode::FrameLabel) => {
                let label = try!(tag_reader.read_c_string());
                Tag::FrameLabel {
                    is_anchor: tag_reader.version >= 6 && length > label.len() + 1 &&
                                try!(tag_reader.read_u8()) != 0,
                    label: label,
                }
            },

            Some(TagCode::DefineSprite) => {
                // TODO: There's probably a better way to prevent the infinite type recursion.
                // Tags can only be nested one level deep, so perhaps I can implement
                // read_tag_list for Reader<Take<R>> to enforce this.
                let mut sprite_reader = Reader::new(
                    &mut tag_reader.input as &mut Read,
                    self.version
                );
                try!(sprite_reader.read_define_sprite())
            },

            Some(TagCode::PlaceObject) => try!(tag_reader.read_place_object()),
            Some(TagCode::PlaceObject2) => try!(tag_reader.read_place_object_2_or_3(2)),
            Some(TagCode::PlaceObject3)  => try!(tag_reader.read_place_object_2_or_3(3)),

            Some(TagCode::RemoveObject) => {
                Tag::RemoveObject {
                    character_id: Some(try!(tag_reader.read_u16())),
                    depth: try!(tag_reader.read_i16()),
                }
            },

            Some(TagCode::RemoveObject2) => {
                Tag::RemoveObject {
                    depth: try!(tag_reader.read_i16()),
                    character_id: None,
                }
            },

            _ => {
                let size = length as usize;
                let mut data = Vec::with_capacity(size);
                data.resize(size, 0);
                try!(tag_reader.input.read_exact(&mut data[..]));
                Tag::Unknown {
                    tag_code: tag_code,
                    data: data,
                }
            }
        };

        if cfg!(debug_assertions) {
            // There should be no data remaining in the tag if we read it correctly.
            // If there is data remaining, we probably screwed up, so panic in debug builds.
            if let Ok(_) = tag_reader.read_u8() {
                panic!("Error reading tag {:?}", tag_code);
            }
        }

        Ok(Some(tag))
    }

    fn read_tag_code_and_length(&mut self) -> Result<(u16, usize)> {
        let tag_code_and_length = try!(self.read_u16());
        let tag_code = tag_code_and_length >> 6;
        let mut length = (tag_code_and_length & 0b111111) as usize;
        if length == 0b111111 {
            // Extended tag.
            length = try!(self.read_u32()) as usize;
        }
        Ok((tag_code, length))
    }

    fn read_define_scene_and_frame_label_data(&mut self) -> Result<Tag> {
        let num_scenes = try!(self.read_encoded_u32()) as usize;
        let mut scenes = Vec::with_capacity(num_scenes);
        for _ in 0..num_scenes {
            scenes.push(FrameLabel {
                frame_num: try!(self.read_encoded_u32()),
                label: try!(self.read_c_string()),
            });
        }

        let num_frame_labels = try!(self.read_encoded_u32()) as usize;
        let mut frame_labels = Vec::with_capacity(num_frame_labels);
        for _ in 0..num_frame_labels {
            frame_labels.push(FrameLabel {
                frame_num: try!(self.read_encoded_u32()),
                label: try!(self.read_c_string()),
            });
        }

        Ok(Tag::DefineSceneAndFrameLabelData {
            scenes: scenes,
            frame_labels: frame_labels,
        })
    }

    fn read_define_shape(&mut self, version: u8) -> Result<Tag> {
        let id = try!(self.read_u16());
        let shape_bounds = try!(self.read_rectangle());
        let (edge_bounds, has_fill_winding_rule,
            has_non_scaling_strokes, has_scaling_strokes) =
        if version >= 4 {
            let edge_bounds = try!(self.read_rectangle());
            let flags = try!(self.read_u8());
            (
                edge_bounds,
                (flags & 0b100) != 0,
                (flags & 0b10) != 0,
                (flags & 0b1) != 0
            )
        } else {
            (shape_bounds.clone(), false, true, false)
        };
        let styles = try!(self.read_shape_styles(version));
        let mut records = Vec::new();
        while let Some(record) = try!(self.read_shape_record(version)) {
            records.push(record);
        }
        Ok(Tag::DefineShape(Shape {
            version: version,
            id: id,
            shape_bounds: shape_bounds,
            edge_bounds: edge_bounds,
            has_fill_winding_rule: has_fill_winding_rule,
            has_non_scaling_strokes: has_non_scaling_strokes,
            has_scaling_strokes: has_scaling_strokes,
            styles: styles,
            shape: records,
        }))
    }

    fn read_define_sound(&mut self) -> Result<Tag> {
        let id = try!(self.read_u16());
        let flags = try!(self.read_u8());
        let compression = match flags >> 4 {
            0 => AudioCompression::UncompressedUnknownEndian,
            1 => AudioCompression::Adpcm,
            2 => AudioCompression::Mp3,
            3 => AudioCompression::Uncompressed,
            4 => AudioCompression::Nellymoser16Khz,
            5 => AudioCompression::Nellymoser8Khz,
            6 => AudioCompression::Nellymoser,
            11 => AudioCompression::Speex,
            _ => return Err(Error::new(ErrorKind::InvalidData,
                        "Invalid audio format.")),
        };
        let sample_rate = match (flags & 0b11_00) >> 2 {
            0 => 5512,
            1 => 11025,
            2 => 22050,
            3 => 44100,
            _ => unreachable!(),
        };
        let is_16_bit = (flags & 0b10) != 0;
        let is_stereo = (flags & 0b1) != 0;
        let num_samples = try!(self.read_u32());
        let mut data = Vec::new();
        try!(self.input.read_to_end(&mut data));
        Ok(Tag::DefineSound(Box::new(Sound {
            id: id,
            compression: compression,
            sample_rate: sample_rate,
            is_16_bit: is_16_bit,
            is_stereo: is_stereo,
            num_samples: num_samples,
            data: data
        })))
    }

    fn read_shape_styles(&mut self, shape_version: u8) -> Result<ShapeStyles> {
        let num_fill_styles = match try!(self.read_u8()) {
            0xff if shape_version >= 2 => try!(self.read_u16()) as usize,
            n => n as usize,
        };
        let mut fill_styles = Vec::with_capacity(num_fill_styles);
        for _ in 0..num_fill_styles {
            fill_styles.push(try!(self.read_fill_style(shape_version)));
        }

        let num_line_styles = match try!(self.read_u8()) {
            // TODO: is this true for linestyles too? SWF19 says not.
            0xff if shape_version >= 2 => try!(self.read_u16()) as usize,
            n => n as usize,
        };
        let mut line_styles = Vec::with_capacity(num_line_styles);
        for _ in 0..num_line_styles {
            line_styles.push(try!(self.read_line_style(shape_version)));
        }

        self.num_fill_bits = try!(self.read_ubits(4)) as u8;
        self.num_line_bits = try!(self.read_ubits(4)) as u8;
        Ok(ShapeStyles {
            fill_styles: fill_styles,
            line_styles: line_styles,
        })
    }

    fn read_fill_style(&mut self, shape_version: u8) -> Result<FillStyle> {
        let fill_style_type = try!(self.read_u8());
        let fill_style = match fill_style_type {
            0x00 => {
                let color = if shape_version >= 3 {
                    try!(self.read_rgba())
                } else {
                    try!(self.read_rgb())
                };
                FillStyle::Color(color)
            }

            0x10 => FillStyle::LinearGradient(try!(self.read_gradient(shape_version))),

            0x12 => FillStyle::RadialGradient(try!(self.read_gradient(shape_version))),

            0x13 => {
                if self.version < 8 || shape_version < 4 {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Focal gradients are only supported in SWF version 8 \
                                           or higher."));
                }
                FillStyle::FocalGradient {
                    gradient: try!(self.read_gradient(shape_version)),
                    focal_point: try!(self.read_fixed8()),
                }
            }

            0x40...0x43 => {
                FillStyle::Bitmap {
                    id: try!(self.read_u16()),
                    matrix: try!(self.read_matrix()),
                    is_smoothed: (fill_style_type & 0b10) == 0,
                    is_repeating: (fill_style_type & 0b01) == 0,
                }
            }

            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid fill style.")),
        };
        Ok(fill_style)
    }

    fn read_line_style(&mut self, shape_version: u8) -> Result<LineStyle> {
        if shape_version < 4 {
            // LineStyle1
            Ok(LineStyle::new_v1(
                try!(self.read_u16()), // TODO: Twips
                if shape_version >= 3 {
                    try!(self.read_rgba())
                } else {
                    try!(self.read_rgb())
                }
            ))
        } else {
            // LineStyle2 in DefineShape4
            let width = try!(self.read_u16());
            let start_cap = match try!(self.read_ubits(2)) {
                0 => LineCapStyle::Round,
                1 => LineCapStyle::None,
                2 => LineCapStyle::Square,
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid line cap type.")),
            };
            let join_style_id = try!(self.read_ubits(2));
            let has_fill = try!(self.read_bit());
            let allow_scale_x = !try!(self.read_bit());
            let allow_scale_y = !try!(self.read_bit());
            let is_pixel_hinted = try!(self.read_bit());
            try!(self.read_ubits(5));
            let allow_close = !try!(self.read_bit());
            let end_cap = match try!(self.read_ubits(2)) {
                0 => LineCapStyle::Round,
                1 => LineCapStyle::None,
                2 => LineCapStyle::Square,
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid line cap type.")),
            };
            let join_style = match join_style_id {
                0 => LineJoinStyle::Round,
                1 => LineJoinStyle::Bevel,
                2 => LineJoinStyle::Miter(try!(self.read_fixed8())),
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid line cap type.")),
            };
            let color = if !has_fill {
                try!(self.read_rgba())
            } else {
                Color { r: 0, g: 0, b: 0, a: 0 }
            };
            let fill_style = if has_fill {
                Some(try!(self.read_fill_style(shape_version)))
            } else {
                None
            };
            Ok( LineStyle {
                width: width,
                color: color,
                start_cap: start_cap,
                end_cap: end_cap,
                join_style: join_style,
                allow_scale_x: allow_scale_x,
                allow_scale_y: allow_scale_y,
                is_pixel_hinted: is_pixel_hinted,
                allow_close: allow_close,
                fill_style: fill_style,
            })
        }
    }

    fn read_gradient(&mut self, shape_version: u8) -> Result<Gradient> {
        let matrix = try!(self.read_matrix());
        self.byte_align();
        let spread = match try!(self.read_ubits(2)) {
            0 => GradientSpread::Pad,
            1 => GradientSpread::Reflect,
            2 => GradientSpread::Repeat,
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid gradient spread mode.")),
        };
        let interpolation = match try!(self.read_ubits(2)) {
            0 => GradientInterpolation::RGB,
            1 => GradientInterpolation::LinearRGB,
            _ => {
                return Err(Error::new(ErrorKind::InvalidData,
                                      "Invalid gradient interpolation mode."))
            }
        };
        let num_records = try!(self.read_ubits(4)) as usize;
        let mut records = Vec::with_capacity(num_records);
        for _ in 0..num_records {
            records.push(GradientRecord {
                ratio: try!(self.read_u8()),
                color: if shape_version >= 3 {
                    try!(self.read_rgba())
                } else {
                    try!(self.read_rgb())
                },
            });
        }
        Ok(Gradient {
            matrix: matrix,
            spread: spread,
            interpolation: interpolation,
            records: records,
        })
    }

    fn read_shape_record(&mut self, shape_version: u8) -> Result<Option<ShapeRecord>> {
        // TODO: Twips
        let is_edge_record = try!(self.read_bit());
        let shape_record = if is_edge_record {
            let is_straight_edge = try!(self.read_bit());
            if is_straight_edge {
                // StraightEdge
                let num_bits = try!(self.read_ubits(4)) as usize + 2;
                let is_axis_aligned = !try!(self.read_bit());
                let is_vertical = is_axis_aligned && try!(self.read_bit());
                let delta_x = if !is_axis_aligned || !is_vertical {
                    try!(self.read_sbits(num_bits))
                } else {
                    0
                };
                let delta_y = if !is_axis_aligned || is_vertical {
                    try!(self.read_sbits(num_bits))
                } else {
                    0
                };
                Some(ShapeRecord::StraightEdge {
                    delta_x: (delta_x as f32) / 20f32,
                    delta_y: (delta_y as f32) / 20f32,
                })
            } else {
                // CurvedEdge
                let num_bits = try!(self.read_ubits(4)) as usize + 2;
                Some(ShapeRecord::CurvedEdge {
                    control_delta_x: (try!(self.read_sbits(num_bits)) as f32) / 20f32,
                    control_delta_y: (try!(self.read_sbits(num_bits)) as f32) / 20f32,
                    anchor_delta_x: (try!(self.read_sbits(num_bits)) as f32) / 20f32,
                    anchor_delta_y: (try!(self.read_sbits(num_bits)) as f32) / 20f32,
                })
            }
        } else {
            let flags = try!(self.read_ubits(5));
            if flags != 0 {
                // StyleChange
                let num_fill_bits = self.num_fill_bits as usize;
                let num_line_bits = self.num_line_bits as usize;
                let mut new_style = StyleChangeData {
                    move_delta_x: 0f32,
                    move_delta_y: 0f32,
                    fill_style_0: None,
                    fill_style_1: None,
                    line_style: None,
                    new_styles: None,
                };
                if (flags & 0b1) != 0 {
                    // move
                    let num_bits = try!(self.read_ubits(5)) as usize;
                    new_style.move_delta_x = (try!(self.read_sbits(num_bits)) as f32) / 20f32;
                    new_style.move_delta_y = (try!(self.read_sbits(num_bits)) as f32) / 20f32;
                }
                if (flags & 0b10) != 0 {
                    new_style.fill_style_0 = Some(try!(self.read_ubits(num_fill_bits)));
                }
                if (flags & 0b100) != 0 {
                    new_style.fill_style_1 = Some(try!(self.read_ubits(num_fill_bits)));
                }
                if (flags & 0b1000) != 0 {
                    new_style.line_style = Some(try!(self.read_ubits(num_line_bits)));
                }
                if shape_version >= 2 && (flags & 0b10000) != 0 {
                    let new_styles = try!(self.read_shape_styles(shape_version));
                    new_style.new_styles = Some(new_styles);
                }
                Some(ShapeRecord::StyleChange(new_style))
            } else {
                None
            }
        };
        Ok(shape_record)
    }

    fn read_define_sprite(&mut self) -> Result<Tag> {
        Ok(Tag::DefineSprite(Sprite {
            id: try!(self.read_u16()),
            num_frames: try!(self.read_u16()),
            tags: try!(self.read_tag_list()),
        }))
    }

    fn read_place_object(&mut self) -> Result<Tag> {
        // TODO: What's a best way to know if the tag has a color transform?
        Ok(Tag::PlaceObject(Box::new(PlaceObject {
            version: 1,
            action: PlaceObjectAction::Place(try!(self.read_u16())),
            depth: try!(self.read_i16()),
            matrix: Some(try!(self.read_matrix())),
            color_transform: self.read_color_transform_no_alpha().ok(),
            ratio: None,
            name: None,
            clip_depth: None,
            class_name: None,
            filters: vec![],
            background_color: None,
            blend_mode: BlendMode::Normal,
            clip_actions: vec![],
            is_image: false,
            is_bitmap_cached: false,
            is_visible: true,
        })))
    }

    fn read_place_object_2_or_3(&mut self, place_object_version: u8) -> Result<Tag> {
        let flags = if place_object_version >= 3 {
            try!(self.read_u16())
        } else {
            try!(self.read_u8()) as u16
        };

        let depth = try!(self.read_i16());

        // PlaceObject3
        let is_image = (flags & 0b10000_00000000) != 0;
        let has_class_name = (flags & 0b1000_00000000) != 0 || (is_image && (flags & 0b10) != 0);
        let class_name = if has_class_name { Some(try!(self.read_c_string())) } else { None };

        let action = match flags & 0b11 {
            0b01 => PlaceObjectAction::Modify,
            0b10 => PlaceObjectAction::Place(try!(self.read_u16())),
            0b11 => PlaceObjectAction::Replace(try!(self.read_u16())),
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid PlaceObject type")),
        };
        let matrix = if (flags & 0b100) != 0 {
            Some(try!(self.read_matrix()))
        } else { None };
        let color_transform = if (flags & 0b1000) != 0 {
            Some(try!(self.read_color_transform()))
        } else { None};
        let ratio = if (flags & 0b1_0000) != 0 {
            Some(try!(self.read_u16()))
        } else { None };
        let name = if (flags & 0b10_0000) != 0 {
            Some(try!(self.read_c_string()))
        } else { None };
        let clip_depth = if (flags & 0b100_0000) != 0 {
            Some(try!(self.read_i16()))
        } else { None };

        // PlaceObject3
        let mut filters = vec![];
        if (flags & 0b1_00000000) != 0 {
            let num_filters = try!(self.read_u8());
            for _ in 0..num_filters {
                filters.push(try!(self.read_filter()));
            }
        } 
        let blend_mode = if (flags & 0b10_00000000) != 0 {
            match try!(self.read_u8()) {
                0 | 1 => BlendMode::Normal,
                2 => BlendMode::Layer,
                3 => BlendMode::Multiply,
                4 => BlendMode::Screen,
                5 => BlendMode::Lighten,
                6 => BlendMode::Darken,
                7 => BlendMode::Difference,
                8 => BlendMode::Add,
                9 => BlendMode::Subtract,
                10 => BlendMode::Invert,
                11 => BlendMode::Alpha,
                12 => BlendMode::Erase,
                13 => BlendMode::Overlay,
                14 => BlendMode::HardLight,
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid blend mode")),
            }
        } else { BlendMode::Normal };
        let is_bitmap_cached = (flags & 0b100_00000000) != 0 && try!(self.read_u8()) != 0;
        let is_visible = (flags & 0b100000_00000000) == 0 || try!(self.read_u8()) != 0;
        let background_color = if (flags & 0b1000000_00000000) != 0 {
            Some(try!(self.read_rgba()))
        } else { None };

        let clip_actions = if (flags & 0b1000_0000) != 0 {
            try!(self.read_clip_actions())
        } else { vec![] };
        Ok(Tag::PlaceObject(Box::new(PlaceObject {
            version: place_object_version,
            action: action,
            depth: depth,
            matrix: matrix,
            color_transform: color_transform,
            ratio: ratio,
            name: name,
            clip_depth: clip_depth,
            clip_actions: clip_actions,
            is_image: is_image,
            is_bitmap_cached: is_bitmap_cached,
            is_visible: is_visible,
            class_name: class_name,
            filters: filters,
            background_color: background_color,
            blend_mode: blend_mode,
        })))
    }

    fn read_clip_actions(&mut self) -> Result<Vec<ClipAction>> {
        try!(self.read_u16()); // Must be 0
        try!(self.read_clip_event_flags()); // All event flags
        let mut clip_actions = vec![];
        while let Some(clip_action) = try!(self.read_clip_action()) {
            clip_actions.push(clip_action);
        }
        Ok(clip_actions)
    }

    fn read_clip_action(&mut self) -> Result<Option<ClipAction>> {
        let events = try!(self.read_clip_event_flags());
        if events.is_empty() {
            Ok(None)
        } else {
            let mut length = try!(self.read_u32());
            let key_code = if events.contains(&ClipEvent::KeyPress) {
                // ActionData length includes the 1 byte key code.
                length -= 1;
                Some(try!(self.read_u8()))
            } else {
                None
            };

            let mut action_data = Vec::with_capacity(length as usize);
            action_data.resize(length as usize, 0);
            try!(self.input.read_exact(&mut action_data));

            Ok(Some(ClipAction {
                events: events,
                key_code: key_code,
                action_data: action_data,
            }))
        }
    }

    fn read_clip_event_flags(&mut self) -> Result<HashSet<ClipEvent>> {
        // TODO: Switch to a bitset.
        let mut event_list = HashSet::with_capacity(32);
        if try!(self.read_bit()) { event_list.insert(ClipEvent::KeyUp); }
        if try!(self.read_bit()) { event_list.insert(ClipEvent::KeyDown); }
        if try!(self.read_bit()) { event_list.insert(ClipEvent::MouseUp); }
        if try!(self.read_bit()) { event_list.insert(ClipEvent::MouseDown); }
        if try!(self.read_bit()) { event_list.insert(ClipEvent::MouseMove); }
        if try!(self.read_bit()) { event_list.insert(ClipEvent::Unload); }
        if try!(self.read_bit()) { event_list.insert(ClipEvent::EnterFrame); }
        if try!(self.read_bit()) { event_list.insert(ClipEvent::Load); }
        if self.version < 6 {
            try!(self.read_u16());
            try!(self.read_u8());
        } else {
            if try!(self.read_bit()) { event_list.insert(ClipEvent::DragOver); }
            if try!(self.read_bit()) { event_list.insert(ClipEvent::RollOut); }
            if try!(self.read_bit()) { event_list.insert(ClipEvent::RollOver); }
            if try!(self.read_bit()) { event_list.insert(ClipEvent::ReleaseOutside); }
            if try!(self.read_bit()) { event_list.insert(ClipEvent::Release); }
            if try!(self.read_bit()) { event_list.insert(ClipEvent::Press); }
            if try!(self.read_bit()) { event_list.insert(ClipEvent::Initialize); }
            if try!(self.read_bit()) { event_list.insert(ClipEvent::Data); }
            if self.version < 6 {
                try!(self.read_u16());
            } else {
                try!(self.read_ubits(5));
                if try!(self.read_bit()) && self.version >= 7 {
                    event_list.insert(ClipEvent::Construct);
                }
                if try!(self.read_bit()) { event_list.insert(ClipEvent::KeyPress); }
                if try!(self.read_bit()) { event_list.insert(ClipEvent::DragOut); }
                try!(self.read_u8());
            }
        }
        Ok(event_list)
    }

    fn read_filter(&mut self) -> Result<Filter> {
        self.byte_align();
        let filter = match try!(self.read_u8()) {
            0 => Filter::DropShadowFilter(Box::new(DropShadowFilter {
                color: try!(self.read_rgba()),
                blur_x: try!(self.read_fixed16()),
                blur_y: try!(self.read_fixed16()),
                angle: try!(self.read_fixed16()),
                distance: try!(self.read_fixed16()),
                strength: try!(self.read_fixed8()),
                is_inner: try!(self.read_bit()),
                is_knockout: try!(self.read_bit()),
                num_passes: try!(self.read_ubits(6)) as u8 & 0b011111,
            })),
            1 => Filter::BlurFilter(Box::new(BlurFilter {
                blur_x: try!(self.read_fixed16()),
                blur_y: try!(self.read_fixed16()),
                num_passes: try!(self.read_ubits(5)) as u8,
            })),
            2 => Filter::GlowFilter(Box::new(GlowFilter {
                color: try!(self.read_rgba()),
                blur_x: try!(self.read_fixed16()),
                blur_y: try!(self.read_fixed16()),
                strength: try!(self.read_fixed8()),
                is_inner: try!(self.read_bit()),
                is_knockout: try!(self.read_bit()),
                num_passes: try!(self.read_ubits(6)) as u8 & 0b011111,
            })),
            3 => Filter::BevelFilter(Box::new(BevelFilter {
                shadow_color: try!(self.read_rgba()),
                highlight_color: try!(self.read_rgba()),
                blur_x: try!(self.read_fixed16()),
                blur_y: try!(self.read_fixed16()),
                angle: try!(self.read_fixed16()),
                distance: try!(self.read_fixed16()),
                strength: try!(self.read_fixed8()),
                is_inner: try!(self.read_bit()),
                is_knockout: try!(self.read_bit()),
                is_on_top: (try!(self.read_ubits(2)) & 0b1) != 0,
                num_passes: try!(self.read_ubits(4)) as u8 & 0b011111,
            })),
            4 => {
                let num_colors = try!(self.read_u8());
                let mut colors = Vec::with_capacity(num_colors as usize);
                for _ in 0..num_colors {
                    colors.push(try!(self.read_rgba()));
                }
                let mut gradient_records = Vec::with_capacity(num_colors as usize);
                for color in colors {
                    gradient_records.push(
                        GradientRecord { color: color, ratio: try!(self.read_u8()) }
                    );
                }
                Filter::GradientGlowFilter(Box::new(GradientGlowFilter {
                    colors: gradient_records,
                    blur_x: try!(self.read_fixed16()),
                    blur_y: try!(self.read_fixed16()),
                    angle: try!(self.read_fixed16()),
                    distance: try!(self.read_fixed16()),
                    strength: try!(self.read_fixed8()),
                    is_inner: try!(self.read_bit()),
                    is_knockout: try!(self.read_bit()),
                    is_on_top: (try!(self.read_ubits(2)) & 0b1) != 0,
                    num_passes: try!(self.read_ubits(4)) as u8,
                }))
            },
            5 => {
                let num_matrix_cols = try!(self.read_u8());
                let num_matrix_rows = try!(self.read_u8());
                let divisor = try!(self.read_fixed16());
                let bias = try!(self.read_fixed16());
                let num_entries = num_matrix_cols * num_matrix_rows;
                let mut matrix = Vec::with_capacity(num_entries as usize);
                for _ in 0..num_entries {
                    matrix.push(try!(self.read_fixed16()));
                }
                let default_color = try!(self.read_rgba());
                let flags = try!(self.read_u8());
                Filter::ConvolutionFilter(Box::new(ConvolutionFilter {
                    num_matrix_cols: num_matrix_cols,
                    num_matrix_rows: num_matrix_rows,
                    divisor: divisor,
                    bias: bias,
                    matrix: matrix,
                    default_color: default_color,
                    is_clamped: (flags & 0b10) != 0,
                    is_preserve_alpha: (flags & 0b1) != 0,
                }))
            },
            6 => {
                let mut matrix = [0f64; 20];
                for i in 0..20 {
                    matrix[i] = try!(self.read_fixed16());
                }
                Filter::ColorMatrixFilter(Box::new(ColorMatrixFilter {
                    matrix: matrix
                }))
            },
            7 => {
                let num_colors = try!(self.read_u8());
                let mut colors = Vec::with_capacity(num_colors as usize);
                for _ in 0..num_colors {
                    colors.push(try!(self.read_rgba()));
                }
                let mut gradient_records = Vec::with_capacity(num_colors as usize);
                for color in colors {
                    gradient_records.push(
                        GradientRecord { color: color, ratio: try!(self.read_u8()) }
                    );
                }
                Filter::GradientBevelFilter(Box::new(GradientBevelFilter {
                    colors: gradient_records,
                    blur_x: try!(self.read_fixed16()),
                    blur_y: try!(self.read_fixed16()),
                    angle: try!(self.read_fixed16()),
                    distance: try!(self.read_fixed16()),
                    strength: try!(self.read_fixed8()),
                    is_inner: try!(self.read_bit()),
                    is_knockout: try!(self.read_bit()),
                    is_on_top: (try!(self.read_ubits(2)) & 0b1) != 0,
                    num_passes: try!(self.read_ubits(4)) as u8 & 0b011111,
                }))
            },
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid filter type")),
        };
        self.byte_align();
        Ok(filter)
    }

    fn read_sound_info(&mut self) -> Result<SoundInfo> {
        let flags = try!(self.read_u8());
        let event = match (flags >> 4) & 0b11 {
            0b10 | 0b11 => SoundEvent::Stop,
            0b00 => SoundEvent::Event,
            0b01 => SoundEvent::Start,
            _ => unreachable!(),
        };
        let in_sample = if (flags & 0b1) != 0 {
            Some(try!(self.read_u32()))
        } else {
            None
        };
        let out_sample = if (flags & 0b10) != 0 {
            Some(try!(self.read_u32()))
        } else {
            None
        };
        let num_loops = if (flags & 0b100) != 0 {
            try!(self.read_u16())
        } else {
            1
        };
        let envelope = if (flags & 0b1000) != 0 {
            let num_points = try!(self.read_u8());
            let mut envelope = SoundEnvelope::new();
            for _ in 0..num_points {
                envelope.push(SoundEnvelopePoint {
                    sample: try!(self.read_u32()),
                    left_volume: try!(self.read_u16()) as f32 / 32768f32,
                    right_volume: try!(self.read_u16()) as f32 / 32768f32,
                })
            }
            Some(envelope)
        } else {
            None
        };
        Ok(SoundInfo {
            event: event,
            in_sample: in_sample,
            out_sample: out_sample,
            num_loops: num_loops,
            envelope: envelope,
        })
    }
}

#[cfg(test)]
pub mod tests {
    use std::fs::File;
    use std::io::{Cursor, Read};
    use std::vec::Vec;
    use super::*;
    use test_data;
    use types::*;
    use tag_codes::TagCode;

    fn reader(data: &[u8]) -> Reader<&[u8]> {
        let default_version = 13;
        Reader::new(data, default_version)
    }

    fn read_from_file(path: &str) -> Swf {
        let mut file = File::open(path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        read_swf(&data[..]).unwrap()
    }

    pub fn read_tag_bytes_from_file_with_index(path: &str, tag_code: TagCode, mut index: usize) -> Vec<u8> {
        let mut file = File::open(path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();

        // Halfway parse the SWF file until we find the tag we're searching for.
        let (swf, mut reader) = super::read_swf_header(&data[..]).unwrap();

        let mut data = Vec::new();
        reader.input.read_to_end(&mut data).unwrap();
        let mut cursor = Cursor::new(data);
        loop {
            let pos = cursor.position();
            let (swf_tag_code, length) =  {
                let mut tag_reader = Reader::new(&mut cursor, swf.version);
                tag_reader.read_tag_code_and_length().unwrap()
            };
            let tag_header_length = cursor.position() - pos;
            let mut data = Vec::new();
            data.resize(length + tag_header_length as usize, 0);
            cursor.set_position(pos);
            cursor.read_exact(&mut data[..]).unwrap();
            if swf_tag_code == 0 {
                panic!("Tag not found");
            } else {
                if swf_tag_code == tag_code as u16 {
                    if index == 0 {
                        // Flash tends to export tags with the extended header even if the size
                        // would fit with the standard header.
                        // This screws up our tests, because swf-rs writes tags with the
                        // minimum header necessary.
                        // We want to easily write new tests by exporting SWFs from the Flash
                        // software, so rewrite with a standard header to match swf-rs output.
                        if length < 0b111111 && (data[0] & 0b111111) == 0b111111 {
                            let mut tag_data = Vec::with_capacity(length + 2);
                            tag_data.extend_from_slice(&data[0..2]);
                            tag_data.extend_from_slice(&data[6..]);
                            tag_data[0] = (data[0] & !0b111111) | (length as u8);
                            data = tag_data;
                        }
                        return data;
                    } else {
                        index -= 1;
                    }
                }
            }
        }
    }

    pub fn read_tag_bytes_from_file(path: &str, tag_code: TagCode) -> Vec<u8> {
        read_tag_bytes_from_file_with_index(path, tag_code, 0)
    }

    #[test]
    fn read_swfs() {
        assert_eq!(read_from_file("tests/swfs/uncompressed.swf").compression,
                   Compression::None);
        assert_eq!(read_from_file("tests/swfs/zlib.swf").compression,
                   Compression::Zlib);
        assert_eq!(read_from_file("tests/swfs/lzma.swf").compression,
                   Compression::Lzma);
    }

    #[test]
    fn read_invalid_swf() {
        let junk = [0u8; 128];
        let result = read_swf(&junk[..]);
        // TODO: Verify correct error.
        assert!(result.is_err());
    }

    #[test]
    fn read_compression_type() {
        assert_eq!(Reader::read_compression_type(&b"FWS"[..]).unwrap(),
                   Compression::None);
        assert_eq!(Reader::read_compression_type(&b"CWS"[..]).unwrap(),
                   Compression::Zlib);
        assert_eq!(Reader::read_compression_type(&b"ZWS"[..]).unwrap(),
                   Compression::Lzma);
        assert!(Reader::read_compression_type(&b"ABC"[..]).is_err());
    }

    #[test]
    fn read_bit() {
        let mut buf: &[u8] = &[0b01010101, 0b00100101];
        let mut reader = Reader::new(&mut buf, 1);
        assert_eq!((0..16).map(|_| reader.read_bit().unwrap()).collect::<Vec<_>>(),
                   [false, true, false, true, false, true, false, true, false, false, true,
                    false, false, true, false, true]);
    }

    #[test]
    fn read_ubits() {
        let mut buf: &[u8] = &[0b01010101, 0b00100101];
        let mut reader = Reader::new(&mut buf, 1);
        assert_eq!((0..8).map(|_| reader.read_ubits(2).unwrap()).collect::<Vec<_>>(),
                   [1, 1, 1, 1, 0, 2, 1, 1]);
    }

    #[test]
    fn read_sbits() {
        let mut buf: &[u8] = &[0b01010101, 0b00100101];
        let mut reader = Reader::new(&mut buf, 1);
        assert_eq!((0..8).map(|_| reader.read_sbits(2).unwrap()).collect::<Vec<_>>(),
                   [1, 1, 1, 1, 0, -2, 1, 1]);
    }

    #[test]
    fn read_fbits() {
        assert_eq!(Reader::new(&[0][..], 1).read_fbits(5).unwrap(), 0f32);
        assert_eq!(Reader::new(&[0b01000000, 0b00000000, 0b0_0000000][..], 1)
                       .read_fbits(17)
                       .unwrap(),
                   0.5f32);
        assert_eq!(Reader::new(&[0b10000000, 0b00000000][..], 1).read_fbits(16).unwrap(),
                   -0.5f32);
    }

    #[test]
    fn read_fixed8() {
        let buf = [0b00000000, 0b00000000, 0b00000000, 0b00000001, 0b10000000, 0b00000110,
                   0b01000000, 0b11101011];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_fixed8().unwrap(), 0f32);
        assert_eq!(reader.read_fixed8().unwrap(), 1f32);
        assert_eq!(reader.read_fixed8().unwrap(), 6.5f32);
        assert_eq!(reader.read_fixed8().unwrap(), -20.75f32);
    }

    #[test]
    fn read_encoded_u32() {
        let read = |data: &[u8]| reader(data).read_encoded_u32().unwrap();
        assert_eq!(read(&[0]), 0);
        assert_eq!(read(&[2]), 2);
        assert_eq!(read(&[0b1_0000001, 0b0_0000001]), 129);
        assert_eq!(read(&[0b1_0000001, 0b1_0000001, 0b0_1100111]),
                   0b1100111_0000001_0000001);
        assert_eq!(read(&[0b1_0000000, 0b1_0000000, 0b1_0000000, 0b1_0000000, 0b0000_1111]),
                   0b1111_0000000_0000000_0000000_0000000);
        assert_eq!(read(&[0b1_0000000, 0b1_0000000, 0b1_0000000, 0b1_0000000, 0b1111_1111]),
                   0b1111_0000000_0000000_0000000_0000000);
    }

    #[test]
    fn read_rectangle_zero() {
        let buf = [0b00000_000];
        let mut reader = Reader::new(&buf[..], 1);
        let rectangle = reader.read_rectangle().unwrap();
        assert_eq!(rectangle,
                   Rectangle {
                       x_min: 0f32,
                       x_max: 0f32,
                       y_min: 0f32,
                       y_max: 0f32,
                   });
    }

    #[test]
    fn read_rectangle_signed() {
        let buf = [0b00110_101, 0b100_01010, 0b0_101100_0, 0b10100_000];
        let mut reader = Reader::new(&buf[..], 1);
        let rectangle = reader.read_rectangle().unwrap();
        assert_eq!(rectangle,
                   Rectangle {
                       x_min: -1f32,
                       x_max: 1f32,
                       y_min: -1f32,
                       y_max: 1f32,
                   });
    }

    #[test]
    fn read_matrix() {
        {
            let buf = [0b0_0_00001_0, 0b0_0000000];
            let mut reader = Reader::new(&buf[..], 1);
            let matrix = reader.read_matrix().unwrap();
            assert_eq!(matrix,
                       Matrix {
                           translate_x: 0f32,
                           translate_y: 0f32,
                           scale_x: 1f32,
                           scale_y: 1f32,
                           rotate_skew_0: 0f32,
                           rotate_skew_1: 0f32,
                       });
        }
    }

    #[test]
    fn read_color() {
        {
            let buf = [1, 128, 255];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_rgb().unwrap(),
                       Color {
                           r: 1,
                           g: 128,
                           b: 255,
                           a: 255,
                       });
        }
        {
            let buf = [1, 128, 235, 44];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_rgba().unwrap(),
                       Color {
                           r: 1,
                           g: 128,
                           b: 235,
                           a: 44,
                       });
        }
    }

    #[test]
    fn read_c_string() {
        {
            let buf = "Testing\0".as_bytes();
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_c_string().unwrap(), "Testing");
        }
        {
            let buf = "12🤖12\0".as_bytes();
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_c_string().unwrap(), "12🤖12");
        }
    }

    #[test]
    fn read_end_tag() {
        let buf = [0, 0];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_tag().unwrap(), None);
    }

    #[test]
    fn read_shape_styles() {
    }

    #[test]
    fn read_fill_style() {
        let read = |buf: &[u8], shape_version| reader(buf).read_fill_style(shape_version).unwrap();

        let fill_style = FillStyle::Color(Color { r: 255, g: 0, b: 0, a: 255 });
        assert_eq!(read(&[0, 255, 0, 0], 1), fill_style);

        // DefineShape3 and 4 read RGBA colors.
        let fill_style = FillStyle::Color(Color { r: 255, g: 0, b: 0, a: 50 });
        assert_eq!(read(&[0, 255, 0, 0, 50], 3), fill_style);

        let fill_style = FillStyle::Bitmap {
            id: 20, matrix: Matrix::new(), is_smoothed: false, is_repeating: true
        };
        assert_eq!(read(&[0x42, 20, 0, 0b00_00001_0, 0b0_0000000], 3), fill_style);

        let mut matrix = Matrix::new();
        matrix.translate_x = 1f32;
        let fill_style = FillStyle::Bitmap {
            id: 33, matrix: matrix, is_smoothed: false, is_repeating: false
        };
        assert_eq!(read(&[0x43, 33, 0, 0b00_00110_0, 0b10100_000, 0b000_00000], 3), fill_style);
    }

    #[test]
    fn read_line_style() {
        // DefineShape1 and 2 read RGB colors.
        let line_style = LineStyle::new_v1( 0, Color { r: 255, g: 0, b: 0, a: 255 } );
        assert_eq!(reader(&[0, 0, 255, 0, 0]).read_line_style(2).unwrap(), line_style);

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
        let read = |buf: &[u8]| reader(buf).read_shape_record(2).unwrap().unwrap();

        let shape_record = ShapeRecord::StraightEdge {
            delta_x: 1f32,
            delta_y: 1f32,
        };
        assert_eq!(read(&[0b11_0100_1_0, 0b1010_0010, 0b100_00000]), shape_record);

        let shape_record = ShapeRecord::StraightEdge {
            delta_x: 0f32,
            delta_y: -1f32,
        };
        assert_eq!(read(&[0b11_0100_0_1, 0b101100_00]), shape_record);

        let shape_record = ShapeRecord::StraightEdge {
            delta_x: -1.5f32,
            delta_y: 0f32,
        };
        assert_eq!(read(&[0b11_0100_0_0, 0b100010_00]), shape_record);
    }

    #[test]
    fn read_tags() {
        for (swf_version, expected_tag, tag_bytes) in test_data::tag_tests() {
            let mut reader = Reader::new(&tag_bytes[..], swf_version);
            let parsed_tag = reader.read_tag().unwrap().unwrap();
            if parsed_tag != expected_tag {
                // Failed, result doesn't match.
                panic!(
                    "Incorrectly parsed tag.\nRead:\n{:?}\n\nExpected:\n{:?}",
                    parsed_tag,
                    expected_tag
                );
            }
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
}
