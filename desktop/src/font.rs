#[cfg(any(target_os = "macos", target_os = "ios", target_family = "windows"))]
use font_kit::family_name::FamilyName;
use font_kit::font::Font;
use font_kit::hinting::HintingOptions;
use font_kit::outline::OutlineSink;
#[cfg(any(target_os = "macos", target_os = "ios", target_family = "windows"))]
use font_kit::properties::{Properties, Stretch, Style, Weight};
#[cfg(any(target_os = "macos", target_os = "ios", target_family = "windows"))]
use font_kit::source::SystemSource;
use pathfinder_geometry::line_segment::LineSegment2F;
use pathfinder_geometry::vector::Vector2F;
use ruffle_core::backend::ui::FontDefinition;
use ruffle_core::swf;
use ruffle_core::swf::{FontFlag, Glyph, Point, ShapeRecord, StyleChangeData, Twips};
use std::error::Error;

struct ShapeBuilder {
    multiplier: f32,
    shape: Vec<swf::ShapeRecord>,
    current_point: Point<Twips>,
    unsupported: bool,
}

impl ShapeBuilder {
    fn to_point(&self, v: Vector2F) -> Point<Twips> {
        let to_twips = |v| Twips::new((v * self.multiplier) as i32);

        // Flip the axis
        Point::new(to_twips(v.x()), to_twips(0.0 - v.y()))
    }
}

impl OutlineSink for ShapeBuilder {
    fn move_to(&mut self, to: Vector2F) {
        self.shape
            .push(ShapeRecord::StyleChange(Box::new(StyleChangeData {
                move_to: Some(self.to_point(to)),
                fill_style_0: Some(1), // this must be Some(1), but probably should just define a style
                fill_style_1: None,
                line_style: None,
                new_styles: None,
            })));
        self.current_point = self.to_point(to);
    }
    fn line_to(&mut self, to: Vector2F) {
        let delta = self.to_point(to) - self.current_point;
        self.current_point = self.to_point(to);
        self.shape.push(ShapeRecord::StraightEdge { delta });
    }
    fn quadratic_curve_to(&mut self, ctrl: Vector2F, to: Vector2F) {
        self.shape.push(ShapeRecord::CurvedEdge {
            control_delta: self.to_point(ctrl) - self.current_point,
            anchor_delta: self.to_point(to) - self.to_point(ctrl),
        });
        self.current_point = self.to_point(to);
    }
    fn cubic_curve_to(&mut self, _ctrl: LineSegment2F, _to: Vector2F) {
        // TODO: Cubic curves are not yet supported. (e.g. Courier)
        self.unsupported = true;
    }
    fn close(&mut self) {}
}

// On non-Apple/Windows (i.e. Unix/Linux) use fontconfig directly to
// search for fonts, because font-kit can't find replacement fonts.
// e.g. "Tinos" as a stand-in for the commonly used "Times New Roman" font.
#[cfg(not(any(target_os = "macos", target_os = "ios", target_family = "windows")))]
fn find_font(name: &str, is_bold: bool, is_italic: bool) -> Result<Font, Box<dyn Error>> {
    let fc = fontconfig::Fontconfig::new().ok_or_else(|| "Fontconfig::new() failed".to_string())?;
    let name = match name {
        "" | "_serif" => "Times New Roman",
        "_sans" => "Arial",
        "_typewriter" => "Courier New",
        _ => name,
    };
    let style = match (is_bold, is_italic) {
        (true, true) => Some("Bold Italic"),
        (true, false) => Some("Bold"),
        (false, true) => Some("Italic"),
        (false, false) => None,
    };
    let font = fc
        .find(name, style)
        .ok_or_else(|| "Fontconfig::find() failed".to_string())?;
    Ok(Font::from_path(font.path, font.index.unwrap_or(0) as u32)?)
}

#[cfg(any(target_os = "macos", target_os = "ios", target_family = "windows"))]
fn find_font(name: &str, is_bold: bool, is_italic: bool) -> Result<Font, Box<dyn Error>> {
    let family_names = match name {
        "" | "_serif" => [
            FamilyName::Title("Times New Roman".into()),
            FamilyName::Serif,
        ],
        "_sans" => [FamilyName::Title("Arial".into()), FamilyName::SansSerif],
        "_typewriter" => [
            FamilyName::Title("Courier New".into()),
            FamilyName::Monospace,
        ],
        _ => [FamilyName::Title(name.into()), FamilyName::Serif],
    };
    let handle = SystemSource::new().select_best_match(
        &family_names,
        &Properties {
            style: if is_italic {
                Style::Italic
            } else {
                Style::Normal
            },
            weight: if is_bold {
                Weight::BOLD
            } else {
                Weight::NORMAL
            },
            stretch: Stretch::NORMAL,
        },
    )?;
    Ok(handle.load()?)
}

pub(crate) fn load(name: &str, is_bold: bool, is_italic: bool) -> Option<FontDefinition> {
    let font = match find_font(name, is_bold, is_italic) {
        Ok(font) => font,
        Err(err) => {
            tracing::warn!("Failed to find device font \"{name}\": {err}");
            return None;
        }
    };

    // TODO: Add a cache based on the actual system font, because
    // different (name, is_bold, is_italic) variations could resolve to the
    // same font.

    // TODO: I don't really know how to correctly calculate this value.
    // This was derived by experimentation.
    let metrics = font.metrics();
    let multiplier = 20_480.0 / (metrics.units_per_em as f32);

    let mut glyphs = Vec::new();
    for i in 2u8..254u8 {
        let glyph_id = if let Some(id) = font.glyph_for_char(i as char) {
            id
        } else {
            continue;
        };

        let mut sink = ShapeBuilder {
            multiplier,
            shape: Vec::new(),
            current_point: Point::ZERO,
            unsupported: false,
        };
        if font
            .outline(glyph_id, HintingOptions::None, &mut sink)
            .is_err()
        {
            continue;
        }
        if sink.unsupported {
            tracing::error!("Could not generate outline for font \"{name}\"");
            return None;
        }

        let advance = if let Ok(advance) = font.advance(glyph_id) {
            (advance.x() * multiplier) as i16
        } else {
            continue;
        };

        let glyph = Glyph {
            shape_records: sink.shape,
            code: i as u16,
            advance,
            // TODO: We could probably generate the bounds, but it seems
            // unnecessary for now.
            bounds: None,
        };

        glyphs.push(glyph);
    }

    let mut flags = FontFlag::empty();
    if is_bold {
        flags |= FontFlag::IS_BOLD;
    }
    if is_italic {
        flags |= FontFlag::IS_ITALIC;
    }

    let layout = swf::FontLayout {
        ascent: (metrics.ascent * multiplier) as u16,
        descent: (-metrics.descent * multiplier) as u16,
        leading: 6000,       // TODO: Implement this
        kerning: Vec::new(), // TODO: and this
    };

    tracing::info!(
        "Using the device font \"{}\" ({}) for font \"{name}\" (bold: {is_bold} italic: {is_italic})",
        font.full_name(), font.postscript_name().unwrap_or_else(|| "<error>".into())
    );

    let font = swf::Font {
        version: 3,
        id: 0,
        name: swf::SwfStr::from_utf8_str(name),
        language: swf::Language::Unknown,
        layout: Some(layout),
        glyphs,
        flags,
    };

    Some(FontDefinition::SwfTag(
        font,
        swf::SwfStr::encoding_for_version(6),
    ))
}
