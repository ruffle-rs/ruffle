use byteorder::{LittleEndian, WriteBytesExt};
use flate2::Compression as ZlibCompression;
use flate2::write::ZlibEncoder;
use std::cmp::max;
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
        try!(writer.write_fixed88(swf.frame_rate));
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

struct Writer<W: Write> {
    pub output: W,
    pub version: u8,
    pub byte: u8,
    pub bit_index: u8,
    pub num_fill_bits: u8,
    pub num_line_bits: u8,
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

    fn into_inner(self) -> W {
        self.output
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

    #[allow(dead_code)]
    fn write_i16(&mut self, n: i16) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_i16::<LittleEndian>(n)
    }

    fn write_fixed88(&mut self, n: f32) -> Result<()> {
        try!(self.flush_bits());
        self.output.write_i16::<LittleEndian>((n * 256f32) as i16)
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

    fn write_c_string(&mut self, s: &str) -> Result<()> {
        try!(self.flush_bits());
        try!(self.output.write_all(s.as_bytes()));
        self.write_u8(0)
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
        let add = [color_transform.a_add, color_transform.g_add,
            color_transform.b_add, color_transform.a_add];
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

    fn write_tag(&mut self, tag: &Tag) -> Result<()> {
        match tag {
            &Tag::ShowFrame => try!(self.write_tag_header(TagCode::ShowFrame, 0)),

            &Tag::DefineShape(ref shape) => try!(self.write_define_shape(shape)),
            &Tag::DefineSprite(ref sprite) => try!(self.write_define_sprite(sprite)),

            // TODO: Allow clone of color.
            &Tag::SetBackgroundColor(ref color) => {
                try!(self.write_tag_header(TagCode::SetBackgroundColor, 3));
                try!(self.write_rgb(color));
            }

            &Tag::PlaceObject(ref place_object) => match (*place_object).version {
                1 => try!(self.write_place_object(place_object)),
                2 => try!(self.write_place_object_2(place_object)),
                3 => unimplemented!(),
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
                try!(self.write_bit(style_change.move_delta_x != 0f32 ||
                                    style_change.move_delta_y != 0f32));
                if style_change.move_delta_x != 0f32 || style_change.move_delta_y != 0f32 {
                    let move_twips_x = (style_change.move_delta_x * 20f32) as i32;
                    let move_twips_y = (style_change.move_delta_y * 20f32) as i32;
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
                try!(self.write_fixed88(focal_point));
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
        if shape_version >= 3 {
            try!(self.write_rgba(&line_style.color));
        } else {
            try!(self.write_rgb(&line_style.color));
        }
        Ok(())
    }

    fn write_gradient(&mut self, gradient: &Gradient, shape_version: u8) -> Result<()> {
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
                try!(self.write_u16(character_id));
            } else {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "PlaceObject version 1 can only use a Place action."
                ));
            }
            try!(self.write_i16(place_object.depth));
            if let Some(ref matrix) = place_object.matrix {
                try!(self.write_matrix(&matrix));
            } else {
                try!(self.write_matrix(&Matrix::new()));
            }
            if let Some(ref color_transform) = place_object.color_transform {
                try!(self.write_color_transform_no_alpha(color_transform));
            }
        }
        try!(self.write_tag_header(TagCode::PlaceObject, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_place_object_2(&mut self, place_object: &PlaceObject) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            let flags: u8 =
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
                };
            try!(writer.write_u8(flags));
            try!(writer.write_i16(place_object.depth));
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
            if !place_object.clip_actions.is_empty() {
                try!(writer.write_clip_actions(&place_object.clip_actions));
            }
            try!(writer.flush_bits());
        }
        try!(self.write_tag_header(TagCode::PlaceObject2, buf.len() as u32));
        try!(self.output.write_all(&buf));
        Ok(())
    }

    fn write_clip_actions(&mut self, clip_actions: &Vec<ClipAction>) -> Result<()> {
        unimplemented!()
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

    fn write_tag_to_buf(tag: &Tag, swf_version: u8) -> Vec<u8> {
        let mut buf = Vec::new();
        Writer::new(&mut buf, swf_version).write_tag(tag).unwrap();
        buf
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
    fn write_fixed88() {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_fixed88(0f32).unwrap();
            writer.write_fixed88(1f32).unwrap();
            writer.write_fixed88(6.5f32).unwrap();
            writer.write_fixed88(-20.75f32).unwrap();
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

    // TAGS
    #[test]
    fn write_unknown_tag() {
        {
            let tag = Tag::Unknown {
                tag_code: 512,
                data: vec![0, 1, 2, 3],
            };
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_tag(&tag).unwrap();
            }
            assert_eq!(buf, [0b00_000100, 0b10000000, 0, 1, 2, 3]);
        }
        {
            let tag = Tag::Unknown {
                tag_code: 513,
                data: vec![0; 63],
            };
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_tag(&tag).unwrap();
            }
            let mut expected: Vec<u8> = vec![0b01_111111, 0b10000000, 0b00111111, 0, 0, 0];
            expected.extend_from_slice(&[0; 63]);
            assert_eq!(buf, expected);
        }
    }

    #[test]
    fn write_simple_tags() {
        {
            let mut buf = Vec::new();
            {
                let mut writer = Writer::new(&mut buf, 1);
                writer.write_tag(&Tag::ShowFrame).unwrap();
            }
            assert_eq!(buf, [0b01_000000, 0b00000000]);
        }
    }

    #[test]
    fn write_set_background_color() {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_tag(&Tag::SetBackgroundColor(Color {
                    r: 255,
                    g: 128,
                    b: 0,
                    a: 255,
                }))
                .unwrap();
        }
        assert_eq!(buf, [0b01_000011, 0b00000010, 255, 128, 0]);
    }

    #[test]
    fn write_file_attributes() {
        let file_attributes = FileAttributes {
            use_direct_blit: false,
            use_gpu: true,
            has_metadata: false,
            is_action_script_3: true,
            use_network_sandbox: false,
        };
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, 1);
            writer.write_tag(&Tag::FileAttributes(file_attributes)).unwrap();
        }
        assert_eq!(buf, [0b01_000100, 0b00010001, 0b00101000, 0, 0, 0]);
    }

    #[test]
    fn write_define_scene_and_frame_label_data() {
        let (tag, tag_bytes) = test_data::define_scene_and_frame_label_data();
        assert_eq!(write_tag_to_buf(&tag, 1), tag_bytes);
    }

    #[test]
    fn write_define_shape() {
        let (tag, tag_bytes) = test_data::define_shape();
        assert_eq!(write_tag_to_buf(&tag, 1), tag_bytes);
    }

    #[test]
    fn write_define_sprite() {
        let (tag, tag_bytes) = test_data::define_sprite();
        assert_eq!(write_tag_to_buf(&tag, 1), tag_bytes);
    }

    #[test]
    fn write_place_object_2() {
        let (tag, tag_bytes) = test_data::place_object_2();
        assert_eq!(write_tag_to_buf(&tag, 1), tag_bytes);
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
