use avm1;
use byteorder::{LittleEndian, WriteBytesExt};
use flate2::Compression as ZlibCompression;
use flate2::write::ZlibEncoder;
use std::cmp::max;
use std::collections::HashSet;
use std::io::{Error, ErrorKind, Result, Write};
use tag_codes::TagCode;
use types::*;
use xz2::write::XzEncoder;

pub fn write_swf<W: Write>(swf: &Swf, mut output: W) -> Result<()> {
    let signature = match swf.compression {
        Compression::None => b"FWS",
        Compression::Zlib => b"CWS",
        Compression::Lzma => b"ZWS",
    };
    try!(output.write_all(&signature[..]));
    try!(output.write_u8(swf.version));

    // Write SWF body.
    let mut swf_body = Vec::new();
    {
        let mut writer = Writer::new(&mut swf_body, swf.version);

        try!(writer.write_rectangle(&swf.stage_size));
        try!(writer.write_fixed8(swf.frame_rate));
        try!(writer.write_u16(swf.num_frames));

        // Write main timeline tag list.
        try!(writer.write_tag_list(&swf.tags));
    }

    // Write SWF header.
    // Uncompressed SWF length.
    try!(output.write_u32::<LittleEndian>(swf_body.len() as u32 + 8));

    // Compress SWF body.
    match swf.compression {
        Compression::None => {
            try!(output.write_all(&swf_body));
        },

        Compression::Zlib => {
            let mut encoder = ZlibEncoder::new(&mut output, ZlibCompression::Best);
            try!(encoder.write_all(&swf_body));
        }

        Compression::Lzma => {
            // LZMA header.
            // SWF format has a mangled LZMA header, so we have to do some magic to conver the
            // standard LZMA header to SWF format.
            // https://helpx.adobe.com/flash-player/kb/exception-thrown-you-decompress-lzma-compressed.html
            use xz2::stream::{Action, LzmaOptions, Stream};
            let mut stream = try!(Stream::new_lzma_encoder(&try!(LzmaOptions::new_preset(9))));
            let mut lzma_header = [0; 13];
            try!(stream.process(&[], &mut lzma_header, Action::Run));
            // Compressed length. We just write out a dummy value.
            try!(output.write_u32::<LittleEndian>(0xffffffff));
            try!(output.write_all(&lzma_header[0..5])); // LZMA property bytes.
            let mut encoder = XzEncoder::new_stream(&mut output, stream);
            try!(encoder.write_all(&swf_body));
        }
    };

    Ok(())
}

pub trait SwfWrite<W: Write> {
    fn get_inner(&mut self) -> &mut W;

    fn write_u8(&mut self, n: u8) -> Result<()> {
        self.get_inner().write_u8(n)
    }

    fn write_u16(&mut self, n: u16) -> Result<()> {
        self.get_inner().write_u16::<LittleEndian>(n)
    }

    fn write_u32(&mut self, n: u32) -> Result<()> {
        self.get_inner().write_u32::<LittleEndian>(n)
    }

    fn write_i8(&mut self, n: i8) -> Result<()> {
        self.get_inner().write_i8(n)
    }

    fn write_i16(&mut self, n: i16) -> Result<()> {
        self.get_inner().write_i16::<LittleEndian>(n)
    }

    fn write_i32(&mut self, n: i32) -> Result<()> {
        self.get_inner().write_i32::<LittleEndian>(n)
    }

    fn write_fixed8(&mut self, n: f32) -> Result<()> {
        self.write_i16((n * 256f32) as i16)
    }

    fn write_fixed16(&mut self, n: f64) -> Result<()> {
        self.write_i32((n * 65536f64) as i32)
    }

    fn write_f32(&mut self, n: f32) -> Result<()> {
        self.get_inner().write_f32::<LittleEndian>(n)
    }

    fn write_f64(&mut self, n: f64) -> Result<()> {
        // Flash weirdly stores f64 as two LE 32-bit chunks.
        // First word is the hi-word, second word is the lo-word.
        let mut num = [0u8; 8];
        try!((&mut num[..]).write_f64::<LittleEndian>(n));
        num.swap(0, 4);
        num.swap(1, 5);
        num.swap(2, 6);
        num.swap(3, 7);
        self.get_inner().write_all(&num)
    }

    fn write_c_string(&mut self, s: &str) -> Result<()> {
        try!(self.get_inner().write_all(s.as_bytes()));
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

    fn write_u8(&mut self, n: u8) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_u8(n)
    }

    fn write_u16(&mut self, n: u16) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_u16::<LittleEndian>(n)
    }

    fn write_u32(&mut self, n: u32) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_u32::<LittleEndian>(n)
    }

    fn write_i8(&mut self, n: i8) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_i8(n)
    }

    fn write_i16(&mut self, n: i16) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_i16::<LittleEndian>(n)
    }

    fn write_i32(&mut self, n: i32) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_i32::<LittleEndian>(n)
    }

    fn write_f32(&mut self, n: f32) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_f32::<LittleEndian>(n)
    }

    fn write_f64(&mut self, n: f64) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_f64::<LittleEndian>(n)
    }

    fn write_c_string(&mut self, s: &str) -> Result<()> {
        try!(self.flush_bits());
        try!(self.get_inner().write_all(s.as_bytes()));
        self.write_u8(0)
    }
}

impl<W: Write> Writer<W> {
    fn new(output: W, version: u8) -> Writer<W> {
        Writer {
            output: output,
            version: version,
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
            try!(self.flush_bits());
        }
        Ok(())
    }

    fn flush_bits(&mut self) -> Result<()> {
        if self.bit_index != 8 {
            try!(self.output.write_u8(self.byte));
            self.bit_index = 8;
            self.byte = 0;
        }
        Ok(())
    }


    fn write_ubits(&mut self, num_bits: u8, n: u32) -> Result<()> {
        for i in 0..num_bits {
            try!(self.write_bit(n & (1 << ((num_bits - i - 1) as u32)) != 0));
        }
        Ok(())
    }

    fn write_sbits(&mut self, num_bits: u8, n: i32) -> Result<()> {
        self.write_ubits(num_bits, n as u32)
    }

    fn write_fbits(&mut self, num_bits: u8, n: f32) -> Result<()> {
        self.write_ubits(num_bits, (n * 65536f32) as u32)
    }

    fn write_encoded_u32(&mut self, mut n: u32) -> Result<()> {
        loop {
            let mut byte = (n & 0b01111111) as u8;
            n >>= 7;
            if n != 0 {
                byte |= 0b10000000;
            }
            try!(self.write_u8(byte));
            if n == 0 {
                break;
            }
        }
        Ok(())
    }

    fn write_rectangle(&mut self, rectangle: &Rectangle) -> Result<()> {
        try!(self.flush_bits());
        let num_bits: u8 = [rectangle.x_min, rectangle.x_max, rectangle.y_min, rectangle.y_max]
            .iter()
            .map(|x| count_sbits((*x * 20f32) as i32))
            .max()
            .unwrap();
        try!(self.write_ubits(5, num_bits as u32));
        try!(self.write_sbits(num_bits, (rectangle.x_min * 20f32) as i32));
        try!(self.write_sbits(num_bits, (rectangle.x_max * 20f32) as i32));
        try!(self.write_sbits(num_bits, (rectangle.y_min * 20f32) as i32));
        try!(self.write_sbits(num_bits, (rectangle.y_max * 20f32) as i32));
        Ok(())
    }

    fn write_character_id(&mut self, id: CharacterId) -> Result<()> {
        self.write_u16(id)
    }

    fn write_rgb(&mut self, color: &Color) -> Result<()> {
        try!(self.write_u8(color.r));
        try!(self.write_u8(color.g));
        try!(self.write_u8(color.b));
        Ok(())
    }

    fn write_rgba(&mut self, color: &Color) -> Result<()> {
        try!(self.write_u8(color.r));
        try!(self.write_u8(color.g));
        try!(self.write_u8(color.b));
        try!(self.write_u8(color.a));
        Ok(())
    }

    fn write_color_transform_no_alpha(&mut self, color_transform: &ColorTransform) -> Result<()> {
        // TODO: Assert that alpha is 1.0?
        try!(self.flush_bits());
        let has_mult = color_transform.r_multiply != 1f32 || color_transform.g_multiply != 1f32 ||
            color_transform.b_multiply != 1f32;
        let has_add = color_transform.r_add != 0 || color_transform.g_add != 0 ||
            color_transform.b_add != 0;
        let multiply = [color_transform.r_multiply, color_transform.g_multiply, color_transform.b_multiply];
        let add = [color_transform.a_add, color_transform.g_add, color_transform.b_add, color_transform.a_add];
        try!(self.write_bit(has_mult));
        try!(self.write_bit(has_add));
        let mut num_bits = 0u8;
        if has_mult {
            num_bits = multiply.iter().map(|n| count_sbits((*n * 256f32) as i32)).max().unwrap();
        }
        if has_add {
            num_bits = max(num_bits, add.iter().map(|n| count_sbits(*n as i32)).max().unwrap());
        }
        try!(self.write_ubits(4, num_bits as u32));
        if has_mult {
            try!(self.write_sbits(num_bits, (color_transform.r_multiply * 256f32) as i32));
            try!(self.write_sbits(num_bits, (color_transform.g_multiply * 256f32) as i32));
            try!(self.write_sbits(num_bits, (color_transform.b_multiply * 256f32) as i32));
        }
        if has_add {
            try!(self.write_sbits(num_bits, color_transform.r_add as i32));
            try!(self.write_sbits(num_bits, color_transform.g_add as i32));
            try!(self.write_sbits(num_bits, color_transform.b_add as i32));
        }
        Ok(())
    }

    fn write_color_transform(&mut self, color_transform: &ColorTransform) -> Result<()> {
        try!(self.flush_bits());
        let has_mult = color_transform.r_multiply != 1f32 || color_transform.g_multiply != 1f32 ||
            color_transform.b_multiply != 1f32 || color_transform.a_multiply != 1f32;
        let has_add = color_transform.r_add != 0 || color_transform.g_add != 0 ||
            color_transform.b_add != 0 || color_transform.a_add != 0;
        let multiply = [color_transform.r_multiply, color_transform.g_multiply,
            color_transform.b_multiply, color_transform.a_multiply];
        let add = [color_transform.r_add, color_transform.g_add,
            color_transform.b_add, color_transform.a_add];
        try!(self.write_bit(has_add));
        try!(self.write_bit(has_mult));
        let mut num_bits = 0u8;
        if has_mult {
            num_bits = multiply.iter().map(|n| count_sbits((*n * 256f32) as i32)).max().unwrap();
        }
        if has_add {
            num_bits = max(num_bits, add.iter().map(|n| count_sbits(*n as i32)).max().unwrap());
        }
        try!(self.write_ubits(4, num_bits as u32));
        if has_mult {
            try!(self.write_sbits(num_bits, (color_transform.r_multiply * 256f32) as i32));
            try!(self.write_sbits(num_bits, (color_transform.g_multiply * 256f32) as i32));
            try!(self.write_sbits(num_bits, (color_transform.b_multiply * 256f32) as i32));
            try!(self.write_sbits(num_bits, (color_transform.a_multiply * 256f32) as i32));
        }
        if has_add {
            try!(self.write_sbits(num_bits, color_transform.r_add as i32));
            try!(self.write_sbits(num_bits, color_transform.g_add as i32));
            try!(self.write_sbits(num_bits, color_transform.b_add as i32));
            try!(self.write_sbits(num_bits, color_transform.a_add as i32));
        }
        Ok(())
    }


    fn write_matrix(&mut self, m: &Matrix) -> Result<()> {
        try!(self.flush_bits());
        // Scale
        let has_scale = m.scale_x != 1f32 || m.scale_y != 1f32;
        try!(self.write_bit(has_scale));
        if has_scale {
            let num_bits = max(count_fbits(m.scale_x), count_fbits(m.scale_y));
            try!(self.write_ubits(5, num_bits as u32));
            try!(self.write_fbits(num_bits, m.scale_x));
            try!(self.write_fbits(num_bits, m.scale_y));
        }
        // Rotate/Skew
        let has_rotate_skew = m.rotate_skew_0 != 0f32 || m.rotate_skew_1 != 0f32;
        try!(self.write_bit(has_rotate_skew));
        if has_rotate_skew {
            let num_bits = max(count_fbits(m.rotate_skew_0), count_fbits(m.rotate_skew_1));
            try!(self.write_ubits(5, num_bits as u32));
            try!(self.write_fbits(num_bits, m.rotate_skew_0));
            try!(self.write_fbits(num_bits, m.rotate_skew_1));
        }
        // Translate (always written)
        let translate_x_twips = (m.translate_x * 20f32) as i32;
        let translate_y_twips = (m.translate_y * 20f32) as i32;
        let num_bits = max(count_sbits(translate_x_twips), count_sbits(translate_y_twips));
        try!(self.write_ubits(5, num_bits as u32));
        try!(self.write_sbits(num_bits, translate_x_twips));
        try!(self.write_sbits(num_bits, translate_y_twips));
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
        })
    }

    fn write_tag(&mut self, tag: &Tag) -> Result<()> {
        match tag {
            &Tag::ShowFrame => try!(self.write_tag_header(TagCode::ShowFrame, 0)),

            &Tag::ExportAssets(ref exports) => {
                let len = exports.iter().map(|e| e.name.len() as u32 + 1).sum::<u32>()
                            + exports.len() as u32 * 2
                            + 2;
                try!(self.write_tag_header(TagCode::ExportAssets, len));
                try!(self.write_u16(exports.len() as u16));
                for &ExportedAsset {id, ref name} in exports {
                    try!(self.write_u16(id));
                    try!(self.write_c_string(name));
                }
            },

            &Tag::Protect(ref password) => {
                if let &Some(ref password_md5) = password {
                    try!(self.write_tag_header(TagCode::Protect, password_md5.len() as u32 + 3));
                    try!(self.write_u16(0)); // Two null bytes? Not specified in SWF19.
                    try!(self.write_c_string(password_md5));
                } else {
                    try!(self.write_tag_header(TagCode::Protect, 0));
                }
            },

            &Tag::DefineBinaryData { id, ref data } => {
                try!(self.write_tag_header(TagCode::DefineBinaryData, data.len() as u32 + 6));
                try!(self.write_u16(id));
                try!(self.write_u32(0)); // Reserved
                try!(self.output.write_all(&data));
            },

            &Tag::DefineBits { id, ref jpeg_data } => {
                self.write_tag_header(TagCode::DefineBits, jpeg_data.len() as u32 + 2)?;
                self.write_u16(id)?;
                self.output.write_all(jpeg_data)?;
            },

            &Tag::DefineBitsJpeg2 { id, ref jpeg_data } => {
                self.write_tag_header(TagCode::DefineBitsJpeg2, jpeg_data.len() as u32 + 2)?;
                self.write_u16(id)?;
                self.output.write_all(jpeg_data)?;
            },

            &Tag::DefineBitsLossless(ref tag) => {
                let mut length = 7 + tag.data.len();
                if tag.format == BitmapFormat::ColorMap8 {
                    length += 1;
                }
                self.write_tag_header(TagCode::DefineBitsLossless, length as u32)?;
                self.write_character_id(tag.id)?;
                let format_id = match tag.format {
                    BitmapFormat::ColorMap8 => 3,
                    BitmapFormat::Rgb15 => 4,
                    BitmapFormat::Rgb24 => 5,
                };
                self.write_u8(format_id)?;
                self.write_u16(tag.width)?;
                self.write_u16(tag.height)?;
                if tag.format == BitmapFormat::ColorMap8 {
                    self.write_u8(tag.num_colors)?;
                }
                self.output.write_all(&tag.data)?;
            },

            &Tag::DefineButton(ref button) => {
                try!(self.write_define_button(button))
            },

            &Tag::DefineButton2(ref button) => {
                try!(self.write_define_button_2(button))
            },

            &Tag::DefineButtonColorTransform { id, ref color_transforms } => {
                let mut buf = Vec::new();
                {
                    let mut writer = Writer::new(&mut buf, self.version);
                    try!(writer.write_u16(id));
                    for color_transform in color_transforms {
                        try!(writer.write_color_transform_no_alpha(color_transform));
                        try!(writer.flush_bits());
                    }
                }
                try!(self.write_tag_header(TagCode::DefineButtonCxform, buf.len() as u32));
                try!(self.output.write_all(&buf));
            },

            &Tag::DefineButtonSound(ref button_sounds) => {
                let mut buf = Vec::new();
                {
                    let mut writer = Writer::new(&mut buf, self.version);
                    try!(writer.write_u16(button_sounds.id));
                    if let Some(ref sound) = button_sounds.over_to_up_sound {
                        try!(writer.write_u16(sound.0));
                        try!(writer.write_sound_info(&sound.1));
                    } else { try!(writer.write_u16(0)) };
                    if let Some(ref sound) = button_sounds.up_to_over_sound {
                        try!(writer.write_u16(sound.0));
                        try!(writer.write_sound_info(&sound.1));
                    } else { try!(writer.write_u16(0)) };
                    if let Some(ref sound) = button_sounds.over_to_down_sound {
                        try!(writer.write_u16(sound.0));
                        try!(writer.write_sound_info(&sound.1));
                    } else { try!(writer.write_u16(0)) };
                    if let Some(ref sound) = button_sounds.down_to_over_sound {
                        try!(writer.write_u16(sound.0));
                        try!(writer.write_sound_info(&sound.1));
                    } else { try!(writer.write_u16(0)) };
                }
                try!(self.write_tag_header(TagCode::DefineButtonSound, buf.len() as u32));
                try!(self.output.write_all(&buf));
            },

            &Tag::DefineEditText(ref edit_text) => self.write_define_edit_text(edit_text)?,

            &Tag::DefineFont(ref font) => {
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
                            try!(writer.write_shape_record(shape_record, 1));
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
            },

            &Tag::DefineFontInfo(ref font_info) => {
                let use_wide_codes = self.version >= 6 || font_info.version >= 2;

                let len = font_info.name.len() +
                    if use_wide_codes { 2 } else { 1 } * font_info.code_table.len() +
                    if font_info.version >= 2 { 1 } else { 0 } +
                    4;

                let tag_id = if font_info.version == 1 { TagCode::DefineFontInfo }
                    else { TagCode::DefineFontInfo2 };
                self.write_tag_header(tag_id, len as u32)?;
                self.write_u16(font_info.id)?;

                // SWF19 has ANSI and Shift-JIS backwards?
                self.write_u8(font_info.name.len() as u8)?;
                self.output.write_all(font_info.name.as_bytes())?;
                self.write_u8(
                    if font_info.is_small_text { 0b100000 } else { 0 } |
                    if font_info.is_ansi { 0b10000 } else { 0 } |
                    if font_info.is_shift_jis { 0b1000 } else { 0 } |
                    if font_info.is_italic { 0b100 } else { 0 } |
                    if font_info.is_bold { 0b10 } else { 0 } |
                    if use_wide_codes { 0b1 } else { 0 }
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
            },

            &Tag::DefineScalingGrid { id, ref splitter_rect } => {
                let mut buf = Vec::new();
                {
                    let mut writer = Writer::new(&mut buf, self.version);
                    try!(writer.write_u16(id));
                    try!(writer.write_rectangle(splitter_rect));
                    try!(writer.flush_bits());
                }
                try!(self.write_tag_header(TagCode::DefineScalingGrid, buf.len() as u32));
                try!(self.output.write_all(&buf));
            },

            &Tag::DefineShape(ref shape) => try!(self.write_define_shape(shape)),
            &Tag::DefineSound(ref sound) => try!(self.write_define_sound(sound)),
            &Tag::DefineSprite(ref sprite) => try!(self.write_define_sprite(sprite)),
            &Tag::DefineText(ref text) => self.write_define_text(text)?,
            &Tag::DoAbc(ref action_data) => {
                try!(self.write_tag_header(TagCode::DoAbc, action_data.len() as u32));
                try!(self.output.write_all(action_data));
            },
            &Tag::DoAction(ref actions) => {
                let mut buf = Vec::new();
                {
                    let mut action_writer = avm1::write::Writer::new(&mut buf, self.version);
                    try!(action_writer.write_action_list(&actions));
                }
                try!(self.write_tag_header(TagCode::DoAction, buf.len() as u32));
                try!(self.output.write_all(&buf));
            },
            &Tag::DoInitAction { id, ref action_data } => {
                try!(self.write_tag_header(TagCode::DoInitAction, action_data.len() as u32 + 2));
                try!(self.write_u16(id));
                try!(self.output.write_all(action_data));
            },

            &Tag::EnableDebugger(ref password_md5) => {
                let len = password_md5.len() as u32 + 1;
                if self.version >= 6 {
                    // SWF v6+ uses EnableDebugger2 tag.
                    try!(self.write_tag_header(TagCode::EnableDebugger2, len + 2));
                    try!(self.write_u16(0)); // Reserved
                } else {
                    try!(self.write_tag_header(TagCode::EnableDebugger, len));
                }
                
                try!(self.write_c_string(password_md5));
            },

            &Tag::EnableTelemetry { ref password_hash } => {
                if password_hash.len() > 0 {
                    try!(self.write_tag_header(TagCode::EnableTelemetry, 34));
                    try!(self.write_u16(0));
                    try!(self.output.write_all(&password_hash[0..32]));
                } else {
                    try!(self.write_tag_header(TagCode::EnableTelemetry, 2));
                    try!(self.write_u16(0));
                }
            },

            &Tag::ImportAssets { ref url, ref imports } => {
                let len = imports.iter().map(|e| e.name.len() as u32 + 3).sum::<u32>()
                            + url.len() as u32 + 1
                            + 2;
                // SWF v8 and later use ImportAssets2 tag.
                if self.version >= 8 {
                    try!(self.write_tag_header(TagCode::ImportAssets2, len + 2));
                    try!(self.write_c_string(url));
                    try!(self.write_u8(1));
                    try!(self.write_u8(0));
                } else {
                    try!(self.write_tag_header(TagCode::ImportAssets, len));
                    try!(self.write_c_string(url));
                }
                try!(self.write_u16(imports.len() as u16));
                for &ExportedAsset {id, ref name} in imports {
                    try!(self.write_u16(id));
                    try!(self.write_c_string(name));
                }
            },

            &Tag::JpegTables(ref data) => {
                self.write_tag_header(TagCode::JpegTables, data.len() as u32)?;
                self.output.write_all(data)?;
            },

            &Tag::Metadata(ref metadata) => {
                try!(self.write_tag_header(TagCode::Metadata, metadata.len() as u32 + 1));
                try!(self.write_c_string(metadata));
            },

            // TODO: Allow clone of color.
            &Tag::SetBackgroundColor(ref color) => {
                try!(self.write_tag_header(TagCode::SetBackgroundColor, 3));
                try!(self.write_rgb(color));
            },

            &Tag::ScriptLimits { max_recursion_depth, timeout_in_seconds } => {
                try!(self.write_tag_header(TagCode::ScriptLimits, 4));
                try!(self.write_u16(max_recursion_depth));
                try!(self.write_u16(timeout_in_seconds));
            },

            &Tag::SetTabIndex { depth, tab_index } => {
                try!(self.write_tag_header(TagCode::SetTabIndex, 4));
                try!(self.write_i16(depth));
                try!(self.write_u16(tab_index));
            },

            &Tag::PlaceObject(ref place_object) => match (*place_object).version {
                1 => try!(self.write_place_object(place_object)),
                2 => try!(self.write_place_object_2_or_3(place_object, 2)),
                3 => try!(self.write_place_object_2_or_3(place_object, 3)),
                _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid PlaceObject version.")),
            },

            &Tag::RemoveObject { depth, character_id } => {
                if let Some(id) = character_id {
                    try!(self.write_tag_header(TagCode::RemoveObject, 4));
                    try!(self.write_u16(id));
                } else {
                    try!(self.write_tag_header(TagCode::RemoveObject2, 2));
                }
                try!(self.write_i16(depth));
            },

            &Tag::SoundStreamBlock(ref data) => {
                try!(self.write_tag_header(TagCode::SoundStreamBlock, data.len() as u32));
                try!(self.output.write_all(data));
            }

            &Tag::SoundStreamHead(ref sound_stream_info) => {
                try!(self.write_sound_stream_head(sound_stream_info, 1));
            }

            &Tag::SoundStreamHead2(ref sound_stream_info) => {
                try!(self.write_sound_stream_head(sound_stream_info, 2));
            }

            &Tag::StartSound { id, ref sound_info } => {
                let length = 3
                    + if let Some(_) = sound_info.in_sample { 4 } else { 0 }
                    + if let Some(_) = sound_info.out_sample { 4 } else { 0 }
                    + if sound_info.num_loops > 1 { 2 } else { 0 }
                    + if let Some(ref e) = sound_info.envelope { e.len() as u32 * 8 + 1 } else { 0 };
                try!(self.write_tag_header(TagCode::StartSound, length));
                try!(self.write_u16(id));
                try!(self.write_sound_info(sound_info));
            },

            &Tag::StartSound2 { ref class_name, ref sound_info } => {
                let length = class_name.len() as u32 + 2
                    + if let Some(_) = sound_info.in_sample { 4 } else { 0 }
                    + if let Some(_) = sound_info.out_sample { 4 } else { 0 }
                    + if sound_info.num_loops > 1 { 2 } else { 0 }
                    + if let Some(ref e) = sound_info.envelope { e.len() as u32 * 8 + 1 } else { 0 };
                try!(self.write_tag_header(TagCode::StartSound2, length));
                try!(self.write_c_string(class_name));
                try!(self.write_sound_info(sound_info));
            },

            &Tag::SymbolClass(ref symbols) => {
                let len = symbols.iter().map(|e| e.class_name.len() as u32 + 3).sum::<u32>()
                            + 2;
                try!(self.write_tag_header(TagCode::SymbolClass, len));
                try!(self.write_u16(symbols.len() as u16));
                for &SymbolClassLink {id, ref class_name} in symbols {
                    try!(self.write_u16(id));
                    try!(self.write_c_string(class_name));
                }
            },

            &Tag::FileAttributes(ref attributes) => {
                try!(self.write_tag_header(TagCode::FileAttributes, 4));
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
                try!(self.write_u32(flags));
            }

            &Tag::FrameLabel { ref label, is_anchor } => {
                // TODO: Assert proper version
                let is_anchor = is_anchor && self.version >= 6;
                let length = label.len() as u32 + if is_anchor { 2 } else { 1 };
                try!(self.write_tag_header(TagCode::FrameLabel, length));
                try!(self.write_c_string(label));
                if is_anchor {
                    try!(self.write_u8(1));
                }
            },

            &Tag::DefineSceneAndFrameLabelData { ref scenes, ref frame_labels } => {
                try!(self.write_define_scene_and_frame_label_data(scenes, frame_labels))
            }

            &Tag::Unknown { tag_code, ref data } => {
                try!(self.write_tag_code_and_length(tag_code, data.len() as u32));
                try!(self.output.write_all(data));
            }
        }
        Ok(())
    }

    fn write_define_button(&mut self, button: &Button) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            try!(writer.write_u16(button.id));
            for record in &button.records {
                try!(writer.write_button_record(record, 1));
            }
            try!(writer.write_u8(0)); // End button records
            // TODO: Assert we have some action.
            try!(writer.output.write_all(&button.actions[0].action_data));
        }
        try!(self.write_tag_header(TagCode::DefineButton, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_define_button_2(&mut self, button: &Button) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            try!(writer.write_u16(button.id));
            let flags = if button.is_track_as_menu { 1 } else { 0 };
            try!(writer.write_u8(flags));

            let mut record_data = Vec::new();
            {
                let mut writer_2 = Writer::new(&mut record_data, self.version);
                for record in &button.records {
                    try!(writer_2.write_button_record(record, 2));
                }
                try!(writer_2.write_u8(0)); // End button records
            }
            try!(writer.write_u16(record_data.len() as u16 + 2));
            try!(writer.output.write_all(&record_data));

            let mut iter = button.actions.iter().peekable();
            while let Some(action) = iter.next() {
                if let Some(_) = iter.peek() {
                    let length = action.action_data.len() as u16 + 4;
                    try!(writer.write_u16(length));
                } else {
                    try!(writer.write_u16(0));
                }
                try!(writer.write_u8(
                    if action.conditions.contains(&ButtonActionCondition::IdleToOverDown) { 0b1000_0000 } else { 0 } |
                    if action.conditions.contains(&ButtonActionCondition::OutDownToIdle) { 0b100_0000 } else { 0 } |
                    if action.conditions.contains(&ButtonActionCondition::OutDownToOverDown) { 0b10_0000 } else { 0 } |
                    if action.conditions.contains(&ButtonActionCondition::OverDownToOutDown) { 0b1_0000 } else { 0 } |
                    if action.conditions.contains(&ButtonActionCondition::OverDownToOverUp) { 0b1000 } else { 0 } |
                    if action.conditions.contains(&ButtonActionCondition::OverUpToOverDown) { 0b100 } else { 0 } |
                    if action.conditions.contains(&ButtonActionCondition::OverUpToIdle) { 0b10 } else { 0 } |
                    if action.conditions.contains(&ButtonActionCondition::IdleToOverUp) { 0b1 } else { 0 }
                ));
                let mut flags = if action.conditions.contains(&ButtonActionCondition::OverDownToIdle) { 0b1 } else { 0 };
                if action.conditions.contains(&ButtonActionCondition::KeyPress) {
                    if let Some(key_code) = action.key_code {
                        flags |= key_code << 1;
                    }
                }
                try!(writer.write_u8(flags));
                try!(writer.output.write_all(&action.action_data));
            }
        }
        try!(self.write_tag_header(TagCode::DefineButton2, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_define_scene_and_frame_label_data(&mut self,
                                               scenes: &Vec<FrameLabel>,
                                               frame_labels: &Vec<FrameLabel>)
                                               -> Result<()> {

        let mut buf = Vec::with_capacity((scenes.len() + frame_labels.len()) * 4);
        {
            let mut writer = Writer::new(&mut buf, self.version);
            try!(writer.write_encoded_u32(scenes.len() as u32));
            for scene in scenes {
                try!(writer.write_encoded_u32(scene.frame_num));
                try!(writer.write_c_string(&scene.label));
            }
            try!(writer.write_encoded_u32(frame_labels.len() as u32));
            for frame_label in frame_labels {
                try!(writer.write_encoded_u32(frame_label.frame_num));
                try!(writer.write_c_string(&frame_label.label));
            }
        }
        try!(self.write_tag_header(TagCode::DefineSceneAndFrameLabelData, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_define_shape(&mut self, shape: &Shape) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            try!(writer.write_u16(shape.id));
            try!(writer.write_rectangle(&shape.shape_bounds));
            if shape.version >= 4 {
                try!(writer.write_rectangle(&shape.edge_bounds));
                try!(writer.flush_bits());
                try!(writer.write_u8(
                    if shape.has_fill_winding_rule { 0b100 } else { 0 } |
                    if shape.has_non_scaling_strokes { 0b10 } else { 0 } |
                    if shape.has_scaling_strokes { 0b1 } else { 0 }
                ));
            }

            try!(writer.write_shape_styles(&shape.styles, shape.version));

            for shape_record in &shape.shape {
                try!(writer.write_shape_record(shape_record, shape.version));
            }
            // End shape record.
            try!(writer.write_ubits(6, 0));
            try!(writer.flush_bits());
        }

        let tag_code = match shape.version {
            1 => TagCode::DefineShape,
            2 => TagCode::DefineShape2,
            3 => TagCode::DefineShape3,
            4 => TagCode::DefineShape4,
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid DefineShape version.")),
        };
        try!(self.write_tag_header(tag_code, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_define_sound(&mut self, sound: &Sound) -> Result<()> {
        try!(self.write_tag_header(
            TagCode::DefineSound,
            7 + sound.data.len() as u32
        ));
        try!(self.write_u16(sound.id));
        try!(self.write_sound_format(&sound.format));
        try!(self.write_u32(sound.num_samples));
        try!(self.output.write_all(&sound.data));
        Ok(())
    }

    fn write_define_sprite(&mut self, sprite: &Sprite) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            try!(writer.write_u16(sprite.id));
            try!(writer.write_u16(sprite.num_frames));
            try!(writer.write_tag_list(&sprite.tags));
        };
        try!(self.write_tag_header(TagCode::DefineSprite, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_button_record(&mut self, record: &ButtonRecord, tag_version: u8) -> Result<()> {
        // TODO: Validate version
        let flags =
            if record.blend_mode != BlendMode::Normal { 0b10_0000 } else { 0 } |
            if !record.filters.is_empty() { 0b1_0000 } else { 0 } |
            if record.states.contains(&ButtonState::HitTest) { 0b1000 } else { 0 } |
            if record.states.contains(&ButtonState::Down) { 0b100 } else { 0 } |
            if record.states.contains(&ButtonState::Over) { 0b10 } else { 0 } |
            if record.states.contains(&ButtonState::Up) { 0b1 } else { 0 };
        try!(self.write_u8(flags));
        try!(self.write_u16(record.id));
        try!(self.write_i16(record.depth));
        try!(self.write_matrix(&record.matrix));
        if tag_version >= 2 {
            try!(self.write_color_transform(&record.color_transform));
            if !record.filters.is_empty() {
                try!(self.write_u8(record.filters.len() as u8));
                for filter in &record.filters {
                    try!(self.write_filter(filter));
                }
            }
            if record.blend_mode != BlendMode::Normal {
                try!(self.write_blend_mode(record.blend_mode));
            }
        }
        Ok(())
    }

    fn write_blend_mode(&mut self, blend_mode: BlendMode) -> Result<()> {
        self.write_u8(
            match blend_mode {
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
            }
        )
    }

    fn write_shape_styles(&mut self, styles: &ShapeStyles, shape_version: u8) -> Result<()> {
        // TODO: Check shape_version.
        if styles.fill_styles.len() >= 0xff {
            try!(self.write_u8(0xff));
            try!(self.write_u16(styles.fill_styles.len() as u16));
        } else {
            try!(self.write_u8(styles.fill_styles.len() as u8));
        }
        for fill_style in &styles.fill_styles {
            try!(self.write_fill_style(fill_style, shape_version));
        }

        if styles.line_styles.len() >= 0xff {
            try!(self.write_u8(0xff));
            try!(self.write_u16(styles.line_styles.len() as u16));
        } else {
            try!(self.write_u8(styles.line_styles.len() as u8));
        }
        for line_style in &styles.line_styles {
            try!(self.write_line_style(line_style, shape_version));
        }

        let num_fill_bits = count_ubits(styles.fill_styles.len() as u32);
        let num_line_bits = count_ubits(styles.line_styles.len() as u32);
        try!(self.write_ubits(4, num_fill_bits as u32));
        try!(self.write_ubits(4, num_line_bits as u32));
        self.num_fill_bits = num_fill_bits;
        self.num_line_bits = num_line_bits;
        Ok(())
    }

    fn write_shape_record(&mut self, record: &ShapeRecord, shape_version: u8) -> Result<()> {
        match record {
            &ShapeRecord::StraightEdge { delta_x, delta_y } => {
                try!(self.write_ubits(2, 0b11)); // Straight edge
                let delta_x_twips = (delta_x * 20f32) as i32;
                let delta_y_twips = (delta_y * 20f32) as i32;
                // TODO: Check underflow?
                let mut num_bits = max(count_sbits(delta_x_twips), count_sbits(delta_y_twips));
                num_bits = max(2, num_bits);
                let is_axis_aligned = delta_x_twips == 0 || delta_y_twips == 0;
                try!(self.write_ubits(4, num_bits as u32 - 2));
                try!(self.write_bit(!is_axis_aligned));
                if is_axis_aligned {
                    try!(self.write_bit(delta_x_twips == 0));
                }
                if delta_x_twips != 0 {
                    try!(self.write_sbits(num_bits, delta_x_twips));
                }
                if delta_y_twips != 0 {
                    try!(self.write_sbits(num_bits, delta_y_twips));
                }
            }
            &ShapeRecord::CurvedEdge { control_delta_x,
                                       control_delta_y,
                                       anchor_delta_x,
                                       anchor_delta_y } => {
                try!(self.write_ubits(2, 0b10)); // Curved edge
                let control_twips_x = (control_delta_x * 20f32) as i32;
                let control_twips_y = (control_delta_y * 20f32) as i32;
                let anchor_twips_x = (anchor_delta_x * 20f32) as i32;
                let anchor_twips_y = (anchor_delta_y * 20f32) as i32;
                let num_bits = [control_twips_x, control_twips_y, anchor_twips_x, anchor_twips_y]
                    .iter()
                    .map(|x| count_sbits(*x))
                    .max()
                    .unwrap();
                try!(self.write_ubits(4, num_bits as u32 - 2));
                try!(self.write_sbits(num_bits, control_twips_x));
                try!(self.write_sbits(num_bits, control_twips_y));
                try!(self.write_sbits(num_bits, anchor_twips_x));
                try!(self.write_sbits(num_bits, anchor_twips_y));
            }
            &ShapeRecord::StyleChange(ref style_change) => {
                try!(self.write_bit(false));  // Style change
                let num_fill_bits = self.num_fill_bits;
                let num_line_bits = self.num_line_bits;
                try!(self.write_bit(style_change.new_styles.is_some()));
                try!(self.write_bit(style_change.line_style.is_some()));
                try!(self.write_bit(style_change.fill_style_1.is_some()));
                try!(self.write_bit(style_change.fill_style_0.is_some()));
                try!(self.write_bit(style_change.move_to.is_some()));
                if let Some((move_x, move_y)) = style_change.move_to {
                    let move_twips_x = (move_x * 20f32) as i32;
                    let move_twips_y = (move_y * 20f32) as i32;
                    let num_bits = max(count_sbits(move_twips_x), count_sbits(move_twips_y));
                    try!(self.write_ubits(5, num_bits as u32));
                    try!(self.write_sbits(num_bits, move_twips_x));
                    try!(self.write_sbits(num_bits, move_twips_y));
                }
                if let Some(fill_style_index) = style_change.fill_style_0 {
                    try!(self.write_ubits(num_fill_bits, fill_style_index));
                }
                if let Some(fill_style_index) = style_change.fill_style_1 {
                    try!(self.write_ubits(num_fill_bits, fill_style_index));
                }
                if let Some(line_style_index) = style_change.line_style {
                    try!(self.write_ubits(num_line_bits, line_style_index));
                }
                if let Some(ref new_styles) = style_change.new_styles {
                    if shape_version < 2 {
                        return Err(Error::new(ErrorKind::InvalidData,
                                              "Only DefineShape2 and higher may change styles."));
                    }
                    try!(self.write_shape_styles(new_styles, shape_version));
                }
            }
        }
        Ok(())
    }

    fn write_fill_style(&mut self, fill_style: &FillStyle, shape_version: u8) -> Result<()> {
        match fill_style {
            &FillStyle::Color(ref color) => {
                try!(self.write_u8(0x00)); // Solid color.
                if shape_version >= 3 {
                    try!(self.write_rgba(color))
                } else {
                    try!(self.write_rgb(color));
                }
            }

            &FillStyle::LinearGradient(ref gradient) => {
                try!(self.write_u8(0x10)); // Linear gradient.
                try!(self.write_gradient(gradient, shape_version));
            }

            &FillStyle::RadialGradient(ref gradient) => {
                try!(self.write_u8(0x12)); // Linear gradient.
                try!(self.write_gradient(gradient, shape_version));
            }

            &FillStyle::FocalGradient { ref gradient, focal_point } => {
                if self.version < 8 {
                    return Err(Error::new(ErrorKind::InvalidData,
                                          "Focal gradients are only support in SWF version 8 \
                                           and higher."));
                }

                try!(self.write_u8(0x13)); // Focal gradient.
                try!(self.write_gradient(gradient, shape_version));
                try!(self.write_fixed8(focal_point));
            }

            &FillStyle::Bitmap { id, ref matrix, is_smoothed, is_repeating } => {
                let fill_style_type = match (is_smoothed, is_repeating) {
                    (true, true) => 0x40,
                    (true, false) => 0x41,
                    (false, true) => 0x42,
                    (false, false) => 0x43,
                };
                try!(self.write_u8(fill_style_type));
                try!(self.write_u16(id));
                try!(self.write_matrix(matrix));
            }
        }
        Ok(())
    }

    fn write_line_style(&mut self, line_style: &LineStyle, shape_version: u8) -> Result<()> {
        try!(self.write_u16(line_style.width));
        if shape_version >= 4 {
            // LineStyle2
            try!(self.write_ubits(2, match line_style.start_cap {
                LineCapStyle::Round => 0,
                LineCapStyle::None => 1,
                LineCapStyle::Square => 2,
            }));
            try!(self.write_ubits(2, match line_style.join_style {
                LineJoinStyle::Round => 0,
                LineJoinStyle::Bevel => 1,
                LineJoinStyle::Miter(_) => 2,
            }));
            try!(self.write_bit(
                if let Some(_) = line_style.fill_style { true } else { false }
            ));
            try!(self.write_bit(!line_style.allow_scale_x));
            try!(self.write_bit(!line_style.allow_scale_y));
            try!(self.write_bit(line_style.is_pixel_hinted));
            try!(self.write_ubits(5, 0));
            try!(self.write_bit(!line_style.allow_close));
            try!(self.write_ubits(2, match line_style.end_cap {
                LineCapStyle::Round => 0,
                LineCapStyle::None => 1,
                LineCapStyle::Square => 2,
            }));
            if let LineJoinStyle::Miter(miter_factor) = line_style.join_style {
                try!(self.write_fixed8(miter_factor));
            }
            match line_style.fill_style {
                None => try!(self.write_rgba(&line_style.color)),
                Some(ref fill) => try!(self.write_fill_style(fill, shape_version)),
            }
        } else if shape_version >= 3 {
            // LineStyle1 with RGBA
            try!(self.write_rgba(&line_style.color));
        } else {
            // LineStyle1 with RGB
            try!(self.write_rgb(&line_style.color));
        }
        Ok(())
    }

    fn write_gradient(&mut self, gradient: &Gradient, shape_version: u8) -> Result<()> {
        try!(self.write_matrix(&gradient.matrix));
        try!(self.flush_bits());
        let spread_bits = match gradient.spread {
            GradientSpread::Pad => 0,
            GradientSpread::Reflect => 1,
            GradientSpread::Repeat => 2,
        };
        try!(self.write_ubits(2, spread_bits));
        let interpolation_bits = match gradient.interpolation {
            GradientInterpolation::RGB => 0,
            GradientInterpolation::LinearRGB => 1,
        };
        try!(self.write_ubits(2, interpolation_bits));
        // TODO: Check overflow.
        try!(self.write_ubits(4, gradient.records.len() as u32));
        for record in &gradient.records {
            try!(self.write_u8(record.ratio));
            if shape_version >= 3 {
                try!(self.write_rgba(&record.color));
            } else {
                try!(self.write_rgb(&record.color));
            }
        }
        Ok(())
    }

    fn write_place_object(&mut self, place_object: &PlaceObject) -> Result<()> {
        // TODO: Assert that the extraneous fields are the defaults.
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            if let PlaceObjectAction::Place(character_id) = place_object.action {
                try!(writer.write_u16(character_id));
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "PlaceObject version 1 can only use a Place action."
                ));
            }
            try!(writer.write_i16(place_object.depth));
            if let Some(ref matrix) = place_object.matrix {
                try!(writer.write_matrix(&matrix));
            } else {
                try!(writer.write_matrix(&Matrix::new()));
            }
            if let Some(ref color_transform) = place_object.color_transform {
                try!(writer.write_color_transform_no_alpha(color_transform));
            }
        }
        try!(self.write_tag_header(TagCode::PlaceObject, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_place_object_2_or_3(&mut self, place_object: &PlaceObject, place_object_version: u8) -> Result<()> {
        let mut buf = Vec::new();
        {
            // TODO: Assert version.
            let mut writer = Writer::new(&mut buf, self.version);
            try!(writer.write_u8(
                if !place_object.clip_actions.is_empty() { 0b1000_0000 } else { 0 } |
                if place_object.clip_depth.is_some() { 0b0100_0000 } else { 0 } |
                if place_object.name.is_some() { 0b0010_0000 } else { 0 } |
                if place_object.ratio.is_some() { 0b0001_0000 } else { 0 } |
                if place_object.color_transform.is_some() { 0b0000_1000 } else { 0 } |
                if place_object.matrix.is_some() { 0b0000_0100 } else { 0 } |
                match place_object.action {
                    PlaceObjectAction::Place(_) => 0b10,
                    PlaceObjectAction::Modify => 0b01,
                    PlaceObjectAction::Replace(_) => 0b11,
                }
            ));
            if place_object_version >= 3 {
                try!(writer.write_u8(
                    if place_object.background_color.is_some() { 0b100_0000 } else { 0 } |
                    if !place_object.is_visible { 0b10_0000 } else { 0 } |
                    if place_object.is_image { 0b1_0000 } else { 0 } |
                    if place_object.class_name.is_some() { 0b1000 } else { 0 } |
                    if place_object.is_bitmap_cached { 0b100 } else { 0 } |
                    if place_object.blend_mode != BlendMode::Normal { 0b10 } else { 0 } |
                    if !place_object.filters.is_empty() { 0b1 } else { 0 }
                ));
            }
            try!(writer.write_i16(place_object.depth));

            if place_object_version >= 3 {
                if let Some(ref class_name) = place_object.class_name {
                    try!(writer.write_c_string(class_name));
                }
            }

            match place_object.action {
                PlaceObjectAction::Place(character_id) |
                PlaceObjectAction::Replace(character_id) => 
                    try!(writer.write_u16(character_id)),
                PlaceObjectAction::Modify => (),
            }
            if let Some(ref matrix) = place_object.matrix {
                try!(writer.write_matrix(matrix));
            };
            if let Some(ref color_transform) = place_object.color_transform {
                try!(writer.write_color_transform(color_transform));
            };
            if let Some(ratio) = place_object.ratio { 
                try!(writer.write_u16(ratio));
            }
            if let Some(ref name) = place_object.name {
                try!(writer.write_c_string(name));
            };
            if let Some(clip_depth) = place_object.clip_depth {
                try!(writer.write_i16(clip_depth));
            }

            if place_object_version >= 3 {
                if !place_object.filters.is_empty() {
                    try!(writer.write_u8(place_object.filters.len() as u8));
                    for filter in &place_object.filters {
                        try!(writer.write_filter(filter));
                    }
                }

                if place_object.blend_mode != BlendMode::Normal {
                    try!(writer.write_blend_mode(place_object.blend_mode));
                }

                if place_object.is_bitmap_cached {
                    try!(writer.write_u8(1));
                }

                if !place_object.is_visible {
                    try!(writer.write_u8(0));
                }

                if let Some(ref background_color) = place_object.background_color {
                    try!(writer.write_rgba(background_color));
                }
            }

            if !place_object.clip_actions.is_empty() {
                try!(writer.write_clip_actions(&place_object.clip_actions));
            }
            try!(writer.flush_bits());
        }
        let tag_code = if place_object_version == 2 {
            TagCode::PlaceObject2
        } else {
            TagCode::PlaceObject3
        };
        try!(self.write_tag_header(tag_code, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_filter(&mut self, filter: &Filter) -> Result<()> {
        match filter {
            &Filter::DropShadowFilter(ref drop_shadow) => {
                try!(self.write_u8(0));
                try!(self.write_rgba(&drop_shadow.color));
                try!(self.write_fixed16(drop_shadow.blur_x));
                try!(self.write_fixed16(drop_shadow.blur_y));
                try!(self.write_fixed16(drop_shadow.angle));
                try!(self.write_fixed16(drop_shadow.distance));
                try!(self.write_fixed8(drop_shadow.strength));
                try!(self.write_bit(drop_shadow.is_inner));
                try!(self.write_bit(drop_shadow.is_knockout));
                try!(self.write_bit(true));
                try!(self.write_ubits(5, drop_shadow.num_passes as u32));
            },

            &Filter::BlurFilter(ref blur) => {
                try!(self.write_u8(1));
                try!(self.write_fixed16(blur.blur_x));
                try!(self.write_fixed16(blur.blur_y));
                try!(self.write_u8(blur.num_passes << 3));
            },

            &Filter::GlowFilter(ref glow) => {
                try!(self.write_u8(2));
                try!(self.write_rgba(&glow.color));
                try!(self.write_fixed16(glow.blur_x));
                try!(self.write_fixed16(glow.blur_y));
                try!(self.write_fixed8(glow.strength));
                try!(self.write_bit(glow.is_inner));
                try!(self.write_bit(glow.is_knockout));
                try!(self.write_bit(true));
                try!(self.write_ubits(5, glow.num_passes as u32));
            },

            &Filter::BevelFilter(ref bevel) => {
                try!(self.write_u8(3));
                try!(self.write_rgba(&bevel.shadow_color));
                try!(self.write_rgba(&bevel.highlight_color));
                try!(self.write_fixed16(bevel.blur_x));
                try!(self.write_fixed16(bevel.blur_y));
                try!(self.write_fixed16(bevel.angle));
                try!(self.write_fixed16(bevel.distance));
                try!(self.write_fixed8(bevel.strength));
                try!(self.write_bit(bevel.is_inner));
                try!(self.write_bit(bevel.is_knockout));
                try!(self.write_bit(true));
                try!(self.write_bit(bevel.is_on_top));
                try!(self.write_ubits(4, bevel.num_passes as u32));
            },

            &Filter::GradientGlowFilter(ref glow) => {
                try!(self.write_u8(4));
                try!(self.write_u8(glow.colors.len() as u8));
                for gradient_record in &glow.colors {
                    try!(self.write_rgba(&gradient_record.color));
                }
                for gradient_record in &glow.colors {
                    try!(self.write_u8(gradient_record.ratio));
                }
                try!(self.write_fixed16(glow.blur_x));
                try!(self.write_fixed16(glow.blur_y));
                try!(self.write_fixed16(glow.angle));
                try!(self.write_fixed16(glow.distance));
                try!(self.write_fixed8(glow.strength));
                try!(self.write_bit(glow.is_inner));
                try!(self.write_bit(glow.is_knockout));
                try!(self.write_bit(true));
                try!(self.write_bit(glow.is_on_top));
                try!(self.write_ubits(4, glow.num_passes as u32));
            },

            &Filter::ConvolutionFilter(ref convolve) => {
                try!(self.write_u8(5));
                try!(self.write_u8(convolve.num_matrix_cols));
                try!(self.write_u8(convolve.num_matrix_rows));
                try!(self.write_fixed16(convolve.divisor));
                try!(self.write_fixed16(convolve.bias));
                for val in &convolve.matrix {
                    try!(self.write_fixed16(*val));
                }
                try!(self.write_rgba(&convolve.default_color));
                try!(self.write_u8(
                    if convolve.is_clamped { 0b10 } else { 0 } |
                    if convolve.is_preserve_alpha { 0b1 } else { 0 }
                ));
            },

            &Filter::ColorMatrixFilter(ref color_matrix) => {
                try!(self.write_u8(6));
                for i in 0..20 {
                    try!(self.write_fixed16(color_matrix.matrix[i]));
                }
            },

            &Filter::GradientBevelFilter(ref bevel) => {
                try!(self.write_u8(7));
                try!(self.write_u8(bevel.colors.len() as u8));
                for gradient_record in &bevel.colors {
                    try!(self.write_rgba(&gradient_record.color));
                }
                for gradient_record in &bevel.colors {
                    try!(self.write_u8(gradient_record.ratio));
                }
                try!(self.write_fixed16(bevel.blur_x));
                try!(self.write_fixed16(bevel.blur_y));
                try!(self.write_fixed16(bevel.angle));
                try!(self.write_fixed16(bevel.distance));
                try!(self.write_fixed8(bevel.strength));
                try!(self.write_bit(bevel.is_inner));
                try!(self.write_bit(bevel.is_knockout));
                try!(self.write_bit(true));
                try!(self.write_bit(bevel.is_on_top));
                try!(self.write_ubits(4, bevel.num_passes as u32));
            },
        }
        try!(self.flush_bits());
        Ok(())
    }

    fn write_clip_actions(&mut self, clip_actions: &Vec<ClipAction>) -> Result<()> {
        try!(self.write_u16(0)); // Reserved
        {
            let mut all_events = HashSet::with_capacity(32);
            for action in clip_actions {
                all_events = &all_events | &action.events;
            }
            try!(self.write_clip_event_flags(&all_events));
        }
        for action in clip_actions {
            try!(self.write_clip_event_flags(&action.events));
            let action_length = action.action_data.len() as u32
                                + if action.key_code.is_some() { 1 } else { 0 };
            try!(self.write_u32(action_length));
            if let Some(k) = action.key_code {
                try!(self.write_u8(k));
            }
            try!(self.output.write_all(&action.action_data));
        }
        if self.version <= 5 {
            try!(self.write_u16(0));
        } else {
            try!(self.write_u32(0));
        }
        Ok(())
    }

    fn write_clip_event_flags(&mut self, clip_events: &HashSet<ClipEvent>) -> Result<()> {
        // TODO: Assert proper version.
        try!(self.write_bit(clip_events.contains(&ClipEvent::KeyUp)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::KeyDown)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::MouseUp)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::MouseDown)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::MouseMove)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::Unload)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::EnterFrame)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::Load)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::DragOver)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::RollOut)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::RollOver)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::ReleaseOutside)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::Release)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::Press)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::Initialize)));
        try!(self.write_bit(clip_events.contains(&ClipEvent::Data)));
        if self.version >= 6 {
            try!(self.write_ubits(5, 0));
            let has_construct = self.version >= 7 && clip_events.contains(&ClipEvent::Construct);
            try!(self.write_bit(has_construct));
            try!(self.write_bit(clip_events.contains(&ClipEvent::KeyPress)));
            try!(self.write_bit(clip_events.contains(&ClipEvent::DragOut)));
            try!(self.write_u8(0));
        }
        try!(self.flush_bits());
        Ok(())
    }

    fn write_sound_stream_head(&mut self, stream_info: &SoundStreamInfo, version: u8) -> Result<()> {
        let tag_code = if version >= 2 {
            TagCode::SoundStreamHead2
        } else {
            TagCode::SoundStreamHead
        };
        // MP3 compression has added latency seek field.
        let length = if stream_info.stream_format.compression == AudioCompression::Mp3 {
            6
        } else {
            4
        };
        try!(self.write_tag_header(tag_code, length));
        try!(self.write_sound_format(&stream_info.playback_format));
        try!(self.write_sound_format(&stream_info.stream_format));
        try!(self.write_u16(stream_info.num_samples_per_block));
        if stream_info.stream_format.compression  == AudioCompression::Mp3 {
            try!(self.write_i16(stream_info.latency_seek));
        }
        Ok(())
    }

    fn write_sound_format(&mut self, sound_format: &SoundFormat) -> Result<()> {
        try!(self.write_ubits(4, match sound_format.compression {
            AudioCompression::UncompressedUnknownEndian => 0,
            AudioCompression::Adpcm => 1,
            AudioCompression::Mp3 => 2,
            AudioCompression::Uncompressed => 3,
            AudioCompression::Nellymoser16Khz => 4,
            AudioCompression::Nellymoser8Khz => 5,
            AudioCompression::Nellymoser => 6,
            AudioCompression::Speex => 11,
        }));
        try!(self.write_ubits(2, match sound_format.sample_rate {
            5512 => 0,
            11025 => 1,
            22050 => 2,
            44100 => 3,
            _ => return Err(Error::new(ErrorKind::InvalidData, "Invalid sample rate.")),
        }));
        try!(self.write_bit(sound_format.is_16_bit));
        try!(self.write_bit(sound_format.is_stereo));
        try!(self.flush_bits());
        Ok(())
    }

    fn write_sound_info(&mut self, sound_info: &SoundInfo) -> Result<()> {
        let flags =
            match sound_info.event {
                SoundEvent::Event => 0b00_0000u8,
                SoundEvent::Start => 0b01_0000u8,
                SoundEvent::Stop => 0b10_0000u8,
            }
            | if let Some(_) = sound_info.in_sample { 0b1 } else { 0 }
            | if let Some(_) = sound_info.out_sample { 0b10 } else { 0 }
            | if sound_info.num_loops > 1 { 0b100 } else { 0 }
            | if let Some(_) = sound_info.envelope { 0b1000 } else { 0 };
        try!(self.write_u8(flags));
        if let Some(n) = sound_info.in_sample {
            try!(self.write_u32(n));
        }
        if let Some(n) = sound_info.out_sample {
            try!(self.write_u32(n));
        }
        if sound_info.num_loops > 1 {
            try!(self.write_u16(sound_info.num_loops));
        }
        if let Some(ref envelope) = sound_info.envelope {
            try!(self.write_u8(envelope.len() as u8));
            for point in envelope {
                try!(self.write_u32(point.sample));
                try!(self.write_u16((point.left_volume * 32768f32) as u16));
                try!(self.write_u16((point.right_volume * 32768f32) as u16));
            }
        }
        Ok(())
    }

    fn write_define_text(&mut self, text: &Text) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(text.id)?;
            writer.write_rectangle(&text.bounds)?;
            writer.write_matrix(&text.matrix)?;
            let num_glyph_bits = text.records
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| count_ubits(g.index)))
                .max()
                .unwrap_or(0);
            let num_advance_bits = text.records
                .iter()
                .flat_map(|r| r.glyphs.iter().map(|g| count_sbits(g.advance)))
                .max()
                .unwrap_or(0);
            writer.write_u8(num_glyph_bits)?;
            writer.write_u8(num_advance_bits)?;

            for record in &text.records {
                let flags =
                    0b10000000 |
                    if record.font_id.is_some() { 0b1000 } else { 0 } |
                    if record.color.is_some() { 0b100 } else { 0 } |
                    if record.y_offset.is_some() { 0b10 } else { 0 } |
                    if record.x_offset.is_some() { 0b1 } else { 0 };
                writer.write_u8(flags)?;
                if let Some(id) = record.font_id {
                    writer.write_character_id(id)?;
                }
                if let Some(ref color) = record.color {
                    writer.write_rgb(color)?;
                }
                if let Some(x) = record.x_offset {
                    writer.write_i16((x * 20.0) as i16)?;
                }
                if let Some(y) = record.y_offset {
                    writer.write_i16((y * 20.0) as i16)?;
                }
                if let Some(height) = record.height {
                    writer.write_u16(height)?;
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
            let flags =
                if edit_text.initial_text.is_some() { 0b10000000 } else { 0 } |
                if edit_text.is_word_wrap { 0b1000000 } else { 0 } |
                if edit_text.is_multiline { 0b100000 } else { 0 } |
                if edit_text.is_password { 0b10000 } else { 0 } |
                if edit_text.is_read_only { 0b1000 } else { 0 } |
                if edit_text.color.is_some() { 0b100 } else { 0 } |
                if edit_text.max_length.is_some() { 0b10 } else { 0 } |
                if edit_text.font_id.is_some() { 0b1 } else { 0 };
            let flags2 =
                if edit_text.font_class_name.is_some() { 0b10000000 } else { 0 } |
                if edit_text.is_auto_size { 0b1000000 } else { 0 } |
                if edit_text.layout.is_some() { 0b100000 } else { 0 } |
                if !edit_text.is_selectable { 0b10000 } else { 0 } |
                if edit_text.has_border { 0b1000 } else { 0 } |
                if edit_text.was_static { 0b100 } else { 0 } |
                if edit_text.is_html { 0b10 } else { 0 } |
                if !edit_text.is_device_font { 0b1 } else { 0 };

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
                writer.write_u16(height)?
            }

            if let Some(ref color) = edit_text.color {
                writer.write_rgba(&color)?
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
                writer.write_u16((layout.left_margin * 20.0) as u16)?;
                writer.write_u16((layout.right_margin * 20.0) as u16)?;
                writer.write_u16((layout.indent * 20.0) as u16)?;
                writer.write_i16((layout.leading * 20.0) as i16)?;
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

    fn write_tag_header(&mut self, tag_code: TagCode, length: u32) -> Result<()> {
        self.write_tag_code_and_length(tag_code as u16, length)
    }

    fn write_tag_code_and_length(&mut self, tag_code: u16, length: u32) -> Result<()> {
        // TODO: Test for tag code/length overflow.
        let mut tag_code_and_length: u16 = tag_code << 6;
        if length < 0b111111 {
            tag_code_and_length |= length as u16;
            self.write_u16(tag_code_and_length)
        } else {
            tag_code_and_length |= 0b111111;
            try!(self.write_u16(tag_code_and_length));
            self.write_u32(length)
        }
    }

    fn write_tag_list(&mut self, tags: &Vec<Tag>) -> Result<()> {
        // TODO: Better error handling. Can skip errored tags, unless EOF.
        for tag in tags {
            try!(self.write_tag(tag));
        }
        // Write End tag.
        self.write_u16(0)
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

fn count_fbits(n: f32) -> u8 {
    count_sbits((n * 65536f32) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Writer;
    use std::io::Result;
    use test_data;
    use types::*;

    fn new_swf() -> Swf {
        Swf {
            version: 13,
            compression: Compression::Zlib,
            stage_size: Rectangle {
                x_min: 0f32,
                x_max: 640f32,
                y_min: 0f32,
                y_max: 480f32,
            },
            frame_rate: 60.0,
            num_frames: 1,
            tags: vec![],
        }
    }

    #[test]
    fn write_swfs() {
        fn write_dummy_swf(compression: Compression) -> Result<()> {
            let mut buf = Vec::new();
            let mut swf = new_swf();
            swf.compression = compression;
            write_swf(&swf, &mut buf)
        }
        assert!(write_dummy_swf(Compression::None).is_ok(),
                "Failed to write uncompressed SWF.");
        assert!(write_dummy_swf(Compression::Zlib).is_ok(),
                "Failed to write zlib SWF.");
        assert!(write_dummy_swf(Compression::Lzma).is_ok(),
                "Failed to write LZMA SWF.");
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
        assert_eq!(buf,
                   [0b00000000, 0b00000000, 0b00000000, 0b00000001, 0b10000000, 0b00000110,
                    0b01000000, 0b11101011]);
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
        assert_eq!(write_to_buf(0b1100111_0000001_0000001),
                   [0b1_0000001, 0b1_0000001, 0b0_1100111]);
        assert_eq!(write_to_buf(0b1111_0000000_0000000_0000000_0000000u32),
                   [0b1_0000000, 0b1_0000000, 0b1_0000000, 0b1_0000000, 0b0000_1111]);
    }

    #[test]
    fn write_bit() {
        let bits = [false, true, false, true, false, true, false, true, false, false, true, false,
                    false, true, false, true];
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            for b in bits.iter() {
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
            for n in nums.iter() {
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
            for n in nums.iter() {
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
            for n in nums.iter() {
                writer.write_fbits(num_bits, *n).unwrap();
            }
            writer.flush_bits().unwrap();
        }
        assert_eq!(buf,
                   [0b01_000000, 0b00000000, 0b00_11_0000, 0b00000000, 0b0000_0000]);
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
            assert_eq!(buf, "Hello!\0".bytes().into_iter().collect::<Vec<_>>());
        }

        {
            let mut buf = Vec::new();
            {
                // TODO: What if I use a cursor instead of buf ?
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_c_string("😀😂!🐼").unwrap();
            }
            assert_eq!(buf,
                       "😀😂!🐼\0".bytes().into_iter().collect::<Vec<_>>());
        }
    }

    #[test]
    fn write_rectangle_zero() {
        let rect = Rectangle {
            x_min: 0f32,
            x_max: 0f32,
            y_min: 0f32,
            y_max: 0f32,
        };
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
            x_min: -1f32,
            x_max: 1f32,
            y_min: -1f32,
            y_max: 1f32,
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
            Writer::new(&mut written_tag_bytes, swf_version).write_tag(&tag).unwrap();
            if written_tag_bytes != expected_tag_bytes {
                panic!(
                    "Error reading tag.\nTag:\n{:?}\n\nWrote:\n{:?}\n\nExpected:\n{:?}",
                    tag,
                    written_tag_bytes,
                    expected_tag_bytes
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
                writer.write_tag_list(&vec![]).unwrap();
            }
            assert_eq!(buf, [0, 0]);
        }
        {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_tag_list(&vec![Tag::ShowFrame]).unwrap();
            }
            assert_eq!(buf, [0b01_000000, 0b00000000, 0, 0]);
        }
        {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_tag_list(&vec![Tag::Unknown {
                                              tag_code: 512,
                                              data: vec![0; 100],
                                          },
                                          Tag::ShowFrame])
                    .unwrap();
            }
            let mut expected = vec![0b00_111111, 0b10000000, 100, 0, 0, 0];
            expected.extend_from_slice(&[0; 100]);
            expected.extend_from_slice(&[0b01_000000, 0b00000000, 0, 0]);
            assert_eq!(buf, expected);
        }
    }
}
