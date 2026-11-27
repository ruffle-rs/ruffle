use crate::{
    error::{Error, Result},
    extensions::ReadSwfExt as _,
    read::{BitReader, Reader},
    types::*,
};

impl<'a> Reader<'a> {
    pub fn read_define_shape(&mut self, version: u8) -> Result<Shape> {
        let id = self.read_u16()?;
        let shape_bounds = self.read_rectangle()?;
        let (edge_bounds, flags) = if version >= 4 {
            (
                self.read_rectangle()?,
                ShapeFlag::from_bits_truncate(self.read_u8()?),
            )
        } else {
            (shape_bounds, ShapeFlag::HAS_NON_SCALING_STROKES)
        };

        let (styles, num_fill_bits, num_line_bits) = self.read_shape_styles(version)?;
        let mut records = Vec::new();
        let mut shape_context = ShapeContext {
            swf_version: self.version(),
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
                is_smoothed: self.version() >= 8 && (fill_style_type & 0b10) == 0,
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
            let num_bits = bits.read_ubits_ct::<4>()? + 2;
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
            let flags = ShapeRecordFlag::from_bits_truncate(bits.read_ubits_ct::<5>()? as u8);
            if !flags.is_empty() {
                // StyleChange
                let num_fill_bits = context.num_fill_bits as u32;
                let num_line_bits = context.num_line_bits as u32;
                let move_to = if flags.contains(ShapeRecordFlag::MOVE_TO) {
                    let num_bits = bits.read_ubits_ct::<5>()?;
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
                    *bits.reader() = reader.as_slice();
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

    pub fn read_define_morph_shape(&mut self, version: u8) -> Result<DefineMorphShape> {
        let id = self.read_character_id()?;
        let start_shape_bounds = self.read_rectangle()?;
        let end_shape_bounds = self.read_rectangle()?;
        let (start_edge_bounds, end_edge_bounds, flags) = if version >= 2 {
            (
                self.read_rectangle()?,
                self.read_rectangle()?,
                DefineMorphShapeFlag::from_bits_truncate(self.read_u8()?),
            )
        } else {
            (
                start_shape_bounds,
                end_shape_bounds,
                DefineMorphShapeFlag::HAS_NON_SCALING_STROKES,
            )
        };

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
        let swf_version = self.version();
        let mut bits = self.bits();
        let mut shape_context = ShapeContext {
            swf_version,
            shape_version: version,
            num_fill_bits: bits.read_ubits_ct::<4>()? as u8,
            num_line_bits: bits.read_ubits_ct::<4>()? as u8,
        };
        let mut start_shape = Vec::new();
        while let Some(record) = Self::read_shape_record(&mut bits, &mut shape_context)? {
            start_shape.push(record);
        }

        let mut end_shape = Vec::new();
        self.read_u8()?; // NumFillBits and NumLineBits are written as 0 for the end shape.
        let mut shape_context = ShapeContext {
            swf_version: self.version(),
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
                swf_version: self.version(),
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
            let offsets = (0..num_glyphs)
                .map(|_| {
                    if flags.contains(FontFlag::HAS_WIDE_OFFSETS) {
                        self.read_u32()
                    } else {
                        self.read_u16().map(u32::from)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;

            // CodeTableOffset
            let code_table_offset = if flags.contains(FontFlag::HAS_WIDE_OFFSETS) {
                self.read_u32()?
            } else {
                self.read_u16()?.into()
            };

            // GlyphShapeTable
            for (i, glyph) in glyphs.iter_mut().enumerate() {
                self.seek_absolute(offsets_ref, offsets[i] as usize);

                let num_bits = self.read_u8()?;
                let mut shape_context = ShapeContext {
                    swf_version: self.version(),
                    shape_version: 1,
                    num_fill_bits: num_bits >> 4,
                    num_line_bits: num_bits & 0b1111,
                };

                let mut bits = self.bits();
                while let Some(record) = Self::read_shape_record(&mut bits, &mut shape_context)? {
                    glyph.shape_records.push(record);
                }
            }

            self.seek_absolute(offsets_ref, code_table_offset as usize);

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
                glyph.advance = self.read_u16()?;
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
}

#[cfg(test)]
#[expect(clippy::unusual_byte_groupings)]
pub mod tests {
    use super::*;

    fn reader(data: &[u8]) -> Reader<'_> {
        let default_version = 13;
        Reader::new(data, default_version)
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
        matrix.tx = Twips::ONE_PX;
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
            .with_width(Twips::ZERO)
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
                swf_version: reader.version(),
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
}
