use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;
use num::FromPrimitive;
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
    let frame_rate = try!(reader.read_fixed88());
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

    fn read_fixed88(&mut self) -> Result<f32> {
        self.input.read_i16::<LittleEndian>().map(|n| n as f32 / 256f32)
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
        while let Some(tag) = try!(self.read_tag()) {
            tags.push(tag);
        }
        // TODO: Verify that we read at least one tag?
        // Are zero-length tag lists allowed?
        Ok(tags)
    }

    fn read_tag(&mut self) -> Result<Option<Tag>> {
        let (tag_code, length) = try!(self.read_tag_code_and_length());

        let mut tag_reader = Reader::new(self.input.by_ref().take(length as u64), self.version);
        use tag_codes::TagCode;
        let tag = match TagCode::from_u16(tag_code) {
            Some(TagCode::End) => return Ok(None),
            Some(TagCode::ShowFrame) => Tag::ShowFrame,
            Some(TagCode::DefineShape) => try!(tag_reader.read_define_shape(1)),

            Some(TagCode::SetBackgroundColor) => {
                Tag::SetBackgroundColor(try!(tag_reader.read_rgb()))
            }

            Some(TagCode::FileAttributes) => {
                let flags = try!(tag_reader.read_u32());
                Tag::FileAttributes(FileAttributes {
                    use_direct_blit: (flags & 0b01000000) != 0,
                    use_gpu: (flags & 0b00100000) != 0,
                    has_metadata: (flags & 0b00010000) != 0,
                    is_action_script_3: (flags & 0b00001000) != 0,
                    use_network_sandbox: (flags & 0b00000001) != 0,
                })
            }

            Some(TagCode::DefineSceneAndFrameLabelData) => {
                try!(tag_reader.read_define_scene_and_frame_label_data())
            }

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
        let styles = try!(self.read_shape_styles(version));
        let mut records = Vec::new();
        while let Some(record) = try!(self.read_shape_record(version)) {
            records.push(record);
        }
        Ok(Tag::DefineShape(Shape {
            version: version,
            id: id,
            shape_bounds: shape_bounds.clone(),
            edge_bounds: shape_bounds,
            styles: styles,
            shape: records,
        }))
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
                if self.version < 8 {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Focal gradients are only supported in SWF version 8 \
                                           or higher."));
                }
                FillStyle::FocalGradient {
                    gradient: try!(self.read_gradient(shape_version)),
                    focal_point: try!(self.read_fixed88()),
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
        Ok(LineStyle {
            width: try!(self.read_u16()), // TODO: Twips
            color: if shape_version >= 3 {
                try!(self.read_rgba())
            } else {
                try!(self.read_rgb())
            },
        })
    }

    fn read_gradient(&mut self, shape_version: u8) -> Result<Gradient> {
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

    pub fn read_tag_bytes_from_file(path: &str, tag_code: TagCode) -> Vec<u8> {
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
                    return data;
                }
            }
        }
    }

    fn read_tag_from_file(path: &str, version: u8) -> Tag {
        let mut file = File::open(path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        Reader::new(&data[..], version).read_tag().unwrap().unwrap()
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
    fn read_fixed88() {
        let buf = [0b00000000, 0b00000000, 0b00000000, 0b00000001, 0b10000000, 0b00000110,
                   0b01000000, 0b11101011];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_fixed88().unwrap(), 0f32);
        assert_eq!(reader.read_fixed88().unwrap(), 1f32);
        assert_eq!(reader.read_fixed88().unwrap(), 6.5f32);
        assert_eq!(reader.read_fixed88().unwrap(), -20.75f32);
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

    // TAGS
    #[test]
    fn read_unknown_tag() {
        {
            let buf = &[0b00_000000, 0b10000000];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_tag().unwrap().unwrap(),
                       Tag::Unknown {
                           tag_code: 512,
                           data: [].to_vec(),
                       });
        }

        {
            let buf = &[0b01_000010, 0b10000000, 1, 2];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_tag().unwrap().unwrap(),
                       Tag::Unknown {
                           tag_code: 513,
                           data: [1, 2].to_vec(),
                       });
        }

        {
            let buf = &[0b01_111111, 0b10000000, 3, 0, 0, 0, 1, 2, 3];
            let mut reader = Reader::new(&buf[..], 1);
            assert_eq!(reader.read_tag().unwrap().unwrap(),
                       Tag::Unknown {
                           tag_code: 513,
                           data: [1, 2, 3].to_vec(),
                       });
        }
    }

    #[test]
    fn read_end_tag() {
        let buf = [0, 0];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_tag().unwrap(), None);
    }

    #[test]
    fn read_show_frame() {
        let buf = [0b01_000000, 0];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_tag().unwrap().unwrap(), Tag::ShowFrame);
    }

    #[test]
    fn read_define_shape() {
        let (tag, tag_bytes) = test_data::define_shape();
        assert_eq!(reader(&tag_bytes).read_tag().unwrap().unwrap(), tag);
    }

    #[test]
    fn read_shape_styles() {
        /*
        let shape_styles = ShapeStyles {
            fill_styles: vec![],
            line_styles: vec![],
        };
        assert_eq!(reader(&[0, 0, 0]).read_shape_styles(1).unwrap(), shape_styles);
        */

        let shape_styles = ShapeStyles {
            fill_styles: vec![
                FillStyle::Color(Color { r: 255, g: 0, b: 0, a: 255 })
            ],
            line_styles: vec![],
        };
        //assert_eq!(reader(&[1, , 00, 0]).read_shape_styles(1).unwrap(), shape_styles);
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
        let line_style = LineStyle { width: 0, color: Color { r: 255, g: 0, b: 0, a: 255 } };
        assert_eq!(reader(&[0, 0, 255, 0, 0]).read_line_style(2).unwrap(), line_style);

        // DefineShape3 and 4 read RGBA colors.
        let line_style = LineStyle { width: 3, color: Color { r: 1, g: 2, b: 3, a: 10 } };
        //assert_eq!(reader(&[3, 0, 1, 2, 3, 10]).read_line_style(3).unwrap(), line_style);

        // TODO: Read LineStyle2 from DefineShape4.
    }

    #[test]
    fn read_gradient() {
        let gradient = Gradient {
            spread: GradientSpread::Reflect,
            interpolation: GradientInterpolation::RGB,
            records: vec![
                //GradientRecord { ratio: 0, color: }
            ]
        };
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
    fn read_file_attributes() {
        let file_attributes = FileAttributes {
            use_direct_blit: false,
            use_gpu: true,
            has_metadata: false,
            is_action_script_3: true,
            use_network_sandbox: false,
        };
        let buf = [0b01_000100, 0b00010001, 0b00101000, 0, 0, 0];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_tag().unwrap().unwrap(),
                   Tag::FileAttributes(file_attributes));
    }

    #[test]
    fn read_set_background_color() {
        let buf = [0b01_000011, 0b00000010, 64, 150, 255];
        let mut reader = Reader::new(&buf[..], 1);
        assert_eq!(reader.read_tag().unwrap().unwrap(),
                   Tag::SetBackgroundColor(Color {
                       r: 64,
                       g: 150,
                       b: 255,
                       a: 255,
                   }));
    }

    #[test]
    fn read_define_scene_and_frame_label_data() {
        let (tag, tag_bytes) = test_data::define_scene_and_frame_label_data();
        assert_eq!(reader(&tag_bytes).read_tag().unwrap().unwrap(), tag);
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
