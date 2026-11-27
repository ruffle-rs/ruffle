use crate::{
    error::{Error, Result},
    tag_code::TagCode,
    types::*,
    write::{BitWriter, SwfWriteExt as _, Writer, count_sbits_twips, count_ubits},
};

use std::io::Write;

impl<W: Write> Writer<W> {
    pub(crate) fn write_define_shape(&mut self, shape: &Shape) -> Result<()> {
        let mut buf = Vec::new();
        {
            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_u16(shape.id)?;
            writer.write_rectangle(&shape.shape_bounds)?;
            if shape.version >= 4 {
                writer.write_rectangle(&shape.edge_bounds)?;
                writer.write_u8(shape.flags.bits())?;
            }

            let (num_fill_bits, num_line_bits) =
                writer.write_shape_styles(&shape.styles, shape.version)?;
            let mut shape_context = ShapeContext {
                swf_version: self.version,
                shape_version: shape.version,
                num_fill_bits,
                num_line_bits,
            };
            let mut bits = writer.bits();
            for shape_record in &shape.shape {
                Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
            }
            // End shape record.
            bits.write_ubits_ct::<6>(0)?;
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
        match record {
            ShapeRecord::StraightEdge { delta } => {
                bits.write_ubits_ct::<2>(0b11)?; // Straight edge
                // TODO: Check underflow?
                let num_bits = count_sbits_twips(delta.dx)
                    .max(count_sbits_twips(delta.dy))
                    .max(2);
                let is_axis_aligned = delta.dx == Twips::ZERO || delta.dy == Twips::ZERO;
                bits.write_ubits_ct::<4>(num_bits - 2)?;
                bits.write_bit(!is_axis_aligned)?;
                let is_vertical = is_axis_aligned && delta.dx == Twips::ZERO;
                if is_axis_aligned {
                    bits.write_bit(is_vertical)?;
                }
                if !is_axis_aligned || !is_vertical {
                    bits.write_sbits_twips(num_bits, delta.dx)?;
                }
                if !is_axis_aligned || is_vertical {
                    bits.write_sbits_twips(num_bits, delta.dy)?;
                }
            }
            ShapeRecord::CurvedEdge {
                control_delta,
                anchor_delta,
            } => {
                bits.write_ubits_ct::<2>(0b10)?; // Curved edge
                let num_bits = count_sbits_twips(control_delta.dx)
                    .max(count_sbits_twips(control_delta.dy))
                    .max(count_sbits_twips(anchor_delta.dx))
                    .max(count_sbits_twips(anchor_delta.dy))
                    .max(2);
                bits.write_ubits_ct::<4>(num_bits - 2)?;
                bits.write_sbits_twips(num_bits, control_delta.dx)?;
                bits.write_sbits_twips(num_bits, control_delta.dy)?;
                bits.write_sbits_twips(num_bits, anchor_delta.dx)?;
                bits.write_sbits_twips(num_bits, anchor_delta.dy)?;
            }
            ShapeRecord::StyleChange(style_change) => {
                bits.write_bit(false)?; // Style change
                let num_fill_bits = context.num_fill_bits.into();
                let num_line_bits = context.num_line_bits.into();
                let mut flags = ShapeRecordFlag::empty();
                flags.set(ShapeRecordFlag::MOVE_TO, style_change.move_to.is_some());
                flags.set(
                    ShapeRecordFlag::FILL_STYLE_0,
                    style_change.fill_style_0.is_some(),
                );
                flags.set(
                    ShapeRecordFlag::FILL_STYLE_1,
                    style_change.fill_style_1.is_some(),
                );
                flags.set(
                    ShapeRecordFlag::LINE_STYLE,
                    style_change.line_style.is_some(),
                );
                flags.set(
                    ShapeRecordFlag::NEW_STYLES,
                    style_change.new_styles.is_some(),
                );
                bits.write_ubits_ct::<5>(flags.bits().into())?;
                if let Some(move_to) = &style_change.move_to {
                    let num_bits = count_sbits_twips(move_to.x).max(count_sbits_twips(move_to.y));
                    bits.write_ubits_ct::<5>(num_bits)?;
                    bits.write_sbits_twips(num_bits, move_to.x)?;
                    bits.write_sbits_twips(num_bits, move_to.y)?;
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
            FillStyle::Color(color) => {
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
                self.write_rgba(*color)?;
            } else {
                return Err(Error::invalid_data("Unexpected line style fill type"));
            }
        } else {
            // LineStyle1
            let color = if let FillStyle::Color(color) = &line_style.fill_style {
                *color
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
                self.write_rgba(record.color)?;
            } else {
                self.write_rgb(record.color)?;
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

    pub(crate) fn write_define_morph_shape(&mut self, data: &DefineMorphShape) -> Result<()> {
        if data.start.fill_styles.len() != data.end.fill_styles.len()
            || data.start.line_styles.len() != data.end.line_styles.len()
        {
            return Err(Error::invalid_data(
                "Start and end state of a morph shape must have the same number of styles.",
            ));
        }

        let num_fill_styles = data.start.fill_styles.len();
        let num_line_styles = data.start.line_styles.len();
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
            for (start, end) in data
                .start
                .fill_styles
                .iter()
                .zip(data.end.fill_styles.iter())
            {
                writer.write_morph_fill_style(start, end)?;
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
            writer.write_u8((num_fill_bits << 4) | (num_line_bits & 0b1111))?;

            let mut shape_context = ShapeContext {
                swf_version: self.version,
                shape_version: 1,
                num_fill_bits,
                num_line_bits,
            };
            let mut bits = writer.bits();
            for shape_record in &data.start.shape {
                Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
            }
            // End shape record.
            bits.write_ubits_ct::<6>(0)?;
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
                writer.write_u8(data.flags.bits())?;
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
            for shape_record in &data.end.shape {
                Self::write_shape_record(shape_record, &mut bits, &mut shape_context)?;
            }
            // End shape record.
            bits.write_ubits_ct::<6>(0)?;
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

    fn write_morph_fill_style(&mut self, start: &FillStyle, end: &FillStyle) -> Result<()> {
        match (start, end) {
            (FillStyle::Color(start_color), FillStyle::Color(end_color)) => {
                self.write_u8(0x00)?; // Solid color.
                self.write_rgba(*start_color)?;
                self.write_rgba(*end_color)?;
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
                ));
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
            self.write_rgba(start_record.color)?;
            self.write_u8(end_record.ratio)?;
            self.write_rgba(end_record.color)?;
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
                    self.write_rgba(*start)?;
                    self.write_rgba(*end)?;
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
                        self.write_rgba(*start)?;
                        self.write_rgba(*end)?;
                    }
                    _ => {
                        return Err(Error::invalid_data("Unexpected line fill style fill type"));
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn write_define_font_1(&mut self, font: &FontV1) -> Result<()> {
        let num_glyphs = font.glyphs.len();
        let mut offsets = Vec::with_capacity(num_glyphs);
        let mut buf = vec![];
        {
            let mut writer = Writer::new(&mut buf, self.version);
            for glyph in &font.glyphs {
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
                bits.write_ubits_ct::<6>(0)?;
            }
        }

        let tag_len = (2 + 2 * font.glyphs.len() + buf.len()) as u32;
        self.write_tag_header(TagCode::DefineFont, tag_len)?;
        self.write_u16(font.id)?;
        for offset in offsets {
            self.write_u16(offset)?;
        }
        self.output.write_all(&buf)?;
        Ok(())
    }

    pub(crate) fn write_define_font_2(&mut self, font: &Font) -> Result<()> {
        let mut buf = Vec::new();
        {
            let num_glyphs = font.glyphs.len();

            // We must write the glyph shapes into a temporary buffer
            // so that we can calculate their offsets.
            // Note: these offsets are still wrong,
            // as there's a variable size CodeTableOffset field in between.
            // We correct for it with a +4/+2 addition later.
            let mut offsets = Vec::with_capacity(num_glyphs);
            let mut has_wide_offsets = false;
            let has_wide_codes = !font.flags.contains(FontFlag::IS_ANSI);
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
                for glyph in &font.glyphs {
                    // Store offset for later.
                    let offset = shape_writer.output.len();
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
                    bits.write_ubits_ct::<6>(0)?;
                }
            }

            let mut writer = Writer::new(&mut buf, self.version);
            writer.write_character_id(font.id)?;
            writer.write_u8(font.flags.bits())?;
            writer.write_language(font.language)?;
            writer.write_u8(font.name.len() as u8)?;
            writer.output.write_all(font.name.as_bytes())?;
            writer.write_u16(num_glyphs as u16)?;

            // If there are no glyphs, then the following tables are omitted.
            if num_glyphs > 0 {
                // OffsetTable size, plus CodeTableOffset size
                let init_offset = if has_wide_offsets {
                    num_glyphs * 4 + 4
                } else {
                    num_glyphs * 2 + 2
                };

                // OffsetTable
                for offset in offsets {
                    if has_wide_offsets {
                        writer.write_u32((offset + init_offset) as u32)?;
                    } else {
                        writer.write_u16((offset + init_offset) as u16)?;
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
                    writer.write_u16(glyph.advance)?;
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

    pub(crate) fn write_define_font_4(&mut self, font: &Font4) -> Result<()> {
        let mut tag_len = 4 + font.name.len();
        if let Some(data) = font.data {
            tag_len += data.len()
        };
        self.write_tag_header(TagCode::DefineFont4, tag_len as u32)?;
        self.write_character_id(font.id)?;
        self.write_u8(
            if font.data.is_some() { 0b100 } else { 0 }
                | if font.is_italic { 0b10 } else { 0 }
                | if font.is_bold { 0b1 } else { 0 },
        )?;
        self.write_string(font.name)?;
        if let Some(data) = font.data {
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
}
