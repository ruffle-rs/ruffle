//! Classes that store formatting options

use crate::context::UpdateContext;
use crate::html::iterators::TextSpanIter;
use crate::string::{AvmString, Integer, Units, WStr, WString};
use crate::tag_utils::SwfMovie;
use crate::xml::{XmlDocument, XmlName, XmlNode};
use gc_arena::{Collect, MutationContext};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::borrow::Cow;
use std::cmp::{min, Ordering};
use std::sync::Arc;

/// Replace HTML entities with their equivalent characters.
///
/// Unknown entities will be ignored.
fn process_html_entity(src: &WStr) -> Option<WString> {
    let amp_index = match src.find(b'&') {
        Some(i) => i,
        None => return None, // No entities.
    };

    // Contains entities; copy and replace.
    let mut result_str = WString::with_capacity(src.len(), src.is_wide());

    // Copy initial segment.
    let (initial, src) = src.split_at(amp_index);
    result_str.push_str(initial);

    let mut entity_start = None;
    let mut unit_indices = src.iter().enumerate().peekable();
    while let Some((i, ch)) = unit_indices.next() {
        if let Some(start) = entity_start {
            if ch == b';' as u16 {
                let s = &src[start + 1..i];
                if s.eq_ignore_case(WStr::from_units(b"amp")) {
                    result_str.push_byte(b'&');
                } else if s.eq_ignore_case(WStr::from_units(b"lt")) {
                    result_str.push_byte(b'<');
                } else if s.eq_ignore_case(WStr::from_units(b"gt")) {
                    result_str.push_byte(b'>');
                } else if s.eq_ignore_case(WStr::from_units(b"quot")) {
                    result_str.push_byte(b'"');
                } else if s.eq_ignore_case(WStr::from_units(b"apos")) {
                    result_str.push_byte(b'\'');
                } else if s.eq_ignore_case(WStr::from_units(b"nbsp")) {
                    result_str.push_byte(b'\xA0');
                } else if s.len() >= 2 && s.at(0) == b'#' as u16 {
                    // Number entity: &#nnnn; or &#xhhhh;
                    let (digits, radix) = if src.at(1) == b'x' as u16 {
                        // Only trailing 4 hex digits are used.
                        let start = usize::max(s.len(), 6) - 4;
                        (&s[start..], 16)
                    } else {
                        // Only trailing 16 digits are used.
                        let start = usize::max(s.len(), 17) - 16;
                        (&s[start..], 10)
                    };
                    if let Ok(n) = u32::from_wstr_radix(digits, radix) {
                        if let Some(c) = std::char::from_u32(n) {
                            result_str.push_char(c);
                        }
                    } else {
                        // Invalid entity; output text as is.
                        if let Some((next_idx, _)) = unit_indices.peek() {
                            result_str.push_str(&src[start..*next_idx]);
                        } else {
                            result_str.push_str(&src[start..]);
                        }
                    }
                } else {
                    // Invalid entity; output text as is.
                    if let Some((next_idx, _)) = unit_indices.peek() {
                        result_str.push_str(&src[start..*next_idx]);
                    } else {
                        result_str.push_str(&src[start..]);
                    }
                }

                entity_start = None;
            } else if ch == b'&' as u16 {
                result_str.push_str(&src[start..i]);
                entity_start = Some(i);
            }
        } else if ch == b'&' as u16 {
            entity_start = Some(i);
        } else {
            result_str.push(ch);
        }
    }

    // Output remaining text if we were in the middle of parsing an entity.
    if let Some(start) = entity_start {
        result_str.push_str(&src[start..]);
    }

    Some(result_str)
}

/// A set of text formatting options to be applied to some part, or the whole
/// of, a given text field.
///
/// Any property set to `None` is treated as undefined, which has different
/// meanings based on the context by which the `TextFormat` is used. For
/// example, when getting the format of a particular region of text, `None`
/// means that multiple regions of text apply. When setting the format of a
/// particular region of text, `None` means that the existing setting for that
/// property will be retained.
#[derive(Clone, Debug, Collect, Default)]
#[collect(require_static)]
pub struct TextFormat {
    pub font: Option<WString>,
    pub size: Option<f64>,
    pub color: Option<swf::Color>,
    pub align: Option<swf::TextAlign>,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub left_margin: Option<f64>,
    pub right_margin: Option<f64>,
    pub indent: Option<f64>,
    pub block_indent: Option<f64>,
    pub kerning: Option<bool>,
    pub leading: Option<f64>,
    pub letter_spacing: Option<f64>,
    pub tab_stops: Option<Vec<f64>>,
    pub bullet: Option<bool>,
    pub url: Option<WString>,
    pub target: Option<WString>,
}

impl TextFormat {
    /// Construct a `TextFormat` from an `EditText`'s SWF tag.
    ///
    /// This requires an `UpdateContext` as we will need to retrieve some font
    /// information from the actually-referenced font.
    pub fn from_swf_tag<'gc>(
        et: swf::EditText<'_>,
        swf_movie: Arc<SwfMovie>,
        context: &mut UpdateContext<'_, 'gc, '_>,
    ) -> Self {
        let encoding = swf_movie.encoding();
        let movie_library = context.library.library_for_movie_mut(swf_movie);
        let font = et.font_id.and_then(|fid| movie_library.get_font(fid));
        let font_class = et
            .font_class_name
            .map(|s| WString::from_utf8(&s.to_string_lossy(encoding)))
            .or_else(|| font.map(|font| WString::from_utf8(font.descriptor().class())))
            .unwrap_or_else(|| WString::from_utf8("Times New Roman"));
        let align = et.layout.as_ref().map(|l| l.align);
        let left_margin = et.layout.as_ref().map(|l| l.left_margin.to_pixels());
        let right_margin = et.layout.as_ref().map(|l| l.right_margin.to_pixels());
        let indent = et.layout.as_ref().map(|l| l.indent.to_pixels());
        let leading = et.layout.map(|l| l.leading.to_pixels());

        // TODO: Text fields that don't specify a font are assumed to be 12px
        // Times New Roman non-bold, non-italic. This will need to be revised
        // when we start supporting device fonts.
        Self {
            font: Some(font_class),
            size: et.height.map(|h| h.to_pixels()),
            color: et
                .color
                .map(|color| swf::Color::from_rgb(color.to_rgb(), 0)),
            align,
            bold: Some(font.map(|font| font.descriptor().bold()).unwrap_or(false)),
            italic: Some(font.map(|font| font.descriptor().italic()).unwrap_or(false)),
            underline: Some(false),
            left_margin,
            right_margin,
            indent,
            block_indent: Some(0.0), // TODO: This isn't specified by the tag itself
            kerning: Some(false),
            leading,
            letter_spacing: Some(0.0), // TODO: This isn't specified by the tag itself
            tab_stops: Some(vec![]),   // TODO: Are there default tab stops?
            bullet: Some(false),       // TODO: Default tab stops?

            // TODO: These are probably empty strings by default
            url: Some(WString::new()),
            target: Some(WString::new()),
        }
    }

    /// Given two text formats, construct a new `TextFormat` where only
    /// matching properties between the two formats are defined.
    pub fn merge_matching_properties(self, rhs: TextFormat) -> Self {
        TextFormat {
            font: if self.font == rhs.font {
                self.font
            } else {
                None
            },
            size: if self.size == rhs.size {
                self.size
            } else {
                None
            },
            color: if self.color == rhs.color {
                self.color
            } else {
                None
            },
            align: if self.align == rhs.align {
                self.align
            } else {
                None
            },
            bold: if self.bold == rhs.bold {
                self.bold
            } else {
                None
            },
            italic: if self.italic == rhs.italic {
                self.italic
            } else {
                None
            },
            underline: if self.underline == rhs.underline {
                self.underline
            } else {
                None
            },
            left_margin: if self.left_margin == rhs.left_margin {
                self.left_margin
            } else {
                None
            },
            right_margin: if self.right_margin == rhs.right_margin {
                self.right_margin
            } else {
                None
            },
            indent: if self.indent == rhs.indent {
                self.indent
            } else {
                None
            },
            block_indent: if self.block_indent == rhs.block_indent {
                self.block_indent
            } else {
                None
            },
            kerning: if self.kerning == rhs.kerning {
                self.kerning
            } else {
                None
            },
            leading: if self.leading == rhs.leading {
                self.leading
            } else {
                None
            },
            letter_spacing: if self.letter_spacing == rhs.letter_spacing {
                self.letter_spacing
            } else {
                None
            },
            tab_stops: if self.tab_stops == rhs.tab_stops {
                self.tab_stops
            } else {
                None
            },
            bullet: if self.bullet == rhs.bullet {
                self.bullet
            } else {
                None
            },
            url: if self.url == rhs.url { self.url } else { None },
            target: if self.target == rhs.target {
                self.target
            } else {
                None
            },
        }
    }

    /// Given two text formats, construct a new `TextFormat` where properties
    /// defined in either `TextFormat` are defined.
    ///
    /// Properties defined in both will resolve to the one defined in `self`.
    pub fn mix_with(self, rhs: TextFormat) -> Self {
        Self {
            font: self.font.or(rhs.font),
            size: self.size.or(rhs.size),
            color: self.color.or(rhs.color),
            align: self.align.or(rhs.align),
            bold: self.bold.or(rhs.bold),
            italic: self.italic.or(rhs.italic),
            underline: self.underline.or(rhs.underline),
            left_margin: self.left_margin.or(rhs.left_margin),
            right_margin: self.right_margin.or(rhs.right_margin),
            indent: self.indent.or(rhs.indent),
            block_indent: self.block_indent.or(rhs.block_indent),
            kerning: self.kerning.or(rhs.kerning),
            leading: self.leading.or(rhs.leading),
            letter_spacing: self.letter_spacing.or(rhs.letter_spacing),
            tab_stops: self.tab_stops.or(rhs.tab_stops),
            bullet: self.bullet.or(rhs.bullet),
            url: self.url.or(rhs.url),
            target: self.target.or(rhs.target),
        }
    }
}

/// Represents the application of a `TextFormat` to a particular text span.
///
/// The actual string data is not stored here; a `TextSpan` is meaningless
/// without its underlying string content. Furthermore, the start position
/// within the string is implicit in the sum of all previous text span's
/// lengths. See `TextSpans` for more information.
///
/// This struct also contains a resolved version of the `TextFormat` structure
/// listed above.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct TextSpan {
    /// How many characters are subsumed by this text span.
    ///
    /// This value must not cause the resulting set of text spans to exceed the
    /// length of the underlying source string.
    pub span_length: usize,

    pub font: WString,
    pub size: f64,
    pub color: swf::Color,
    pub align: swf::TextAlign,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub left_margin: f64,
    pub right_margin: f64,
    pub indent: f64,
    pub block_indent: f64,
    pub kerning: bool,
    pub leading: f64,
    pub letter_spacing: f64,
    pub tab_stops: Vec<f64>,
    pub bullet: bool,
    pub url: WString,
    pub target: WString,
}

impl Default for TextSpan {
    fn default() -> Self {
        Self {
            span_length: 0,
            font: WString::new(),
            size: 12.0,
            color: swf::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            align: swf::TextAlign::Left,
            bold: false,
            italic: false,
            underline: false,
            left_margin: 0.0,
            right_margin: 0.0,
            indent: 0.0,
            block_indent: 0.0,
            kerning: false,
            leading: 0.0,
            letter_spacing: 0.0,
            tab_stops: vec![],
            bullet: false,
            url: WString::new(),
            target: WString::new(),
        }
    }
}

impl TextSpan {
    pub fn with_length_and_format(length: usize, tf: TextFormat) -> Self {
        let mut data = Self {
            span_length: length,
            ..Default::default()
        };

        data.set_text_format(&tf);

        data
    }

    /// Determine if this and another span have identical text formats.
    ///
    /// It is assumed that the two text spans being considered are adjacent;
    /// and we have no way of checking, so this function doesn't check that.
    #[allow(clippy::float_cmp)]
    fn can_merge(&self, rhs: &Self) -> bool {
        self.font == rhs.font
            && self.size == rhs.size
            && self.color == rhs.color
            && self.align == rhs.align
            && self.bold == rhs.bold
            && self.italic == rhs.italic
            && self.underline == rhs.underline
            && self.left_margin == rhs.left_margin
            && self.right_margin == rhs.right_margin
            && self.indent == rhs.indent
            && self.block_indent == rhs.block_indent
            && self.kerning == rhs.kerning
            && self.leading == rhs.leading
            && self.letter_spacing == rhs.letter_spacing
            && self.tab_stops == rhs.tab_stops
            && self.bullet == rhs.bullet
            && self.url == rhs.url
            && self.target == rhs.target
    }

    /// Apply a text format to this text span.
    ///
    /// Properties marked `None` on the `TextFormat` will remain unchanged.
    fn set_text_format(&mut self, tf: &TextFormat) {
        if let Some(font) = &tf.font {
            self.font = font.clone();
        }

        if let Some(size) = &tf.size {
            self.size = *size;
        }

        if let Some(color) = &tf.color {
            self.color = color.clone();
        }

        if let Some(align) = &tf.align {
            self.align = *align;
        }

        if let Some(bold) = &tf.bold {
            self.bold = *bold;
        }

        if let Some(italic) = &tf.italic {
            self.italic = *italic;
        }

        if let Some(underline) = &tf.underline {
            self.underline = *underline;
        }

        if let Some(left_margin) = &tf.left_margin {
            self.left_margin = *left_margin;
        }

        if let Some(right_margin) = &tf.right_margin {
            self.right_margin = *right_margin;
        }

        if let Some(indent) = &tf.indent {
            self.indent = *indent;
        }

        if let Some(block_indent) = &tf.block_indent {
            self.block_indent = *block_indent;
        }

        if let Some(kerning) = &tf.kerning {
            self.kerning = *kerning;
        }

        if let Some(leading) = &tf.leading {
            self.leading = *leading;
        }

        if let Some(letter_spacing) = &tf.letter_spacing {
            self.letter_spacing = *letter_spacing;
        }

        if let Some(tab_stops) = &tf.tab_stops {
            self.tab_stops = tab_stops.clone();
        }

        if let Some(bullet) = &tf.bullet {
            self.bullet = *bullet;
        }

        if let Some(url) = &tf.url {
            self.url = url.clone();
        }

        if let Some(target) = &tf.target {
            self.target = target.clone();
        }
    }

    /// Convert the text span into a format.
    ///
    /// The text format returned will have all properties defined.
    pub fn get_text_format(&self) -> TextFormat {
        TextFormat {
            font: Some(self.font.clone()),
            size: Some(self.size),
            color: Some(self.color.clone()),
            align: Some(self.align),
            bold: Some(self.bold),
            italic: Some(self.italic),
            underline: Some(self.underline),
            left_margin: Some(self.left_margin),
            right_margin: Some(self.right_margin),
            indent: Some(self.indent),
            block_indent: Some(self.block_indent),
            kerning: Some(self.kerning),
            leading: Some(self.leading),
            letter_spacing: Some(self.letter_spacing),
            tab_stops: Some(self.tab_stops.clone()),
            bullet: Some(self.bold),
            url: Some(self.url.clone()),
            target: Some(self.target.clone()),
        }
    }
}

/// Struct which contains text formatted by `TextSpan`s.
#[derive(Clone, Debug, Collect)]
#[collect(require_static)]
pub struct FormatSpans {
    text: WString,
    displayed_text: WString,
    spans: Vec<TextSpan>,
    default_format: TextFormat,
}

impl Default for FormatSpans {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatSpans {
    pub fn new() -> Self {
        FormatSpans {
            text: WString::new(),
            displayed_text: WString::new(),
            spans: vec![TextSpan::default()],
            default_format: TextFormat::default(),
        }
    }

    /// Construct a format span from its raw parts.
    #[allow(dead_code)]
    pub fn from_str_and_spans(text: &WStr, spans: &[TextSpan]) -> Self {
        FormatSpans {
            text: text.into(),
            displayed_text: WString::new(),
            spans: spans.to_vec(),
            default_format: Default::default(),
        }
    }

    pub fn from_text(text: WString, format: TextFormat) -> Self {
        let len = text.len();
        Self {
            text,
            displayed_text: WString::new(),
            spans: vec![TextSpan::with_length_and_format(len, format.clone())],
            default_format: format,
        }
    }

    /// Lower an HTML tree into text-span representation.
    ///
    /// This is the "legacy" implementation of this process: it only looks for
    /// a handful of presentational attributes in the HTML tree to generate
    /// styling. There's also a `lower_from_css` that respects both
    /// presentational markup and CSS stylesheets.
    pub fn from_html(html: &WStr, default_format: TextFormat) -> Self {
        let mut format_stack = vec![default_format.clone()];
        let mut text = WString::new();
        let mut spans: Vec<TextSpan> = Vec::new();

        // quick_xml::Reader requires a [u8] slice, but doesn't actually care about Unicode;
        // this means we can pass the raw buffer in the Latin1 case.
        let (raw_bytes, is_raw_latin1) = match html.units() {
            Units::Bytes(units) => (Cow::Borrowed(units), true),
            // TODO: In principle, we should be able to encode (and later decode)
            // the utf16 units in the [u8] array without discarding losing surrogates.
            Units::Wide(_) => (
                Cow::Owned(html.to_utf8_lossy().into_owned().into_bytes()),
                true,
            ),
        };

        let decode_to_wstr = |raw: Cow<'_, [u8]>| -> WString {
            if is_raw_latin1 {
                WString::from_buf(raw.into_owned())
            } else {
                WString::from_utf8(&String::from_utf8_lossy(&raw))
            }
        };

        let mut reader = Reader::from_reader(&raw_bytes[..]);
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf) {
                Ok(Event::Empty(ref e)) => match &e.name().to_ascii_lowercase()[..] {
                    b"br" | b"sbr" => {
                        text.push_byte(b'\n');
                        if let Some(span) = spans.last_mut() {
                            span.span_length += 1;
                        }
                    }
                    _ => {}
                },
                Ok(Event::Start(ref e)) => {
                    let attribute = move |name| {
                        e.attributes().with_checks(false).find_map(|attribute| {
                            let attribute = attribute.unwrap();
                            attribute
                                .key
                                .eq_ignore_ascii_case(name)
                                .then(|| decode_to_wstr(attribute.value))
                        })
                    };
                    let mut format = format_stack.last().unwrap().clone();
                    match &e.name().to_ascii_lowercase()[..] {
                        b"br" | b"sbr" => {
                            text.push_byte(b'\n');
                            if let Some(span) = spans.last_mut() {
                                span.span_length += 1;
                            }

                            // Skip push to `format_stack`.
                            continue;
                        }
                        b"p" => {
                            if let Some(align) = attribute(b"align") {
                                if align == WStr::from_units(b"left") {
                                    format.align = Some(swf::TextAlign::Left)
                                } else if align == WStr::from_units(b"center") {
                                    format.align = Some(swf::TextAlign::Center)
                                } else if align == WStr::from_units(b"right") {
                                    format.align = Some(swf::TextAlign::Right)
                                }
                            }
                        }
                        b"a" => {
                            if let Some(href) = attribute(b"href") {
                                format.url = Some(href);
                            }

                            if let Some(target) = attribute(b"target") {
                                format.target = Some(target);
                            }
                        }
                        b"font" => {
                            if let Some(face) = attribute(b"face") {
                                format.font = Some(face);
                            }

                            if let Some(size) = attribute(b"size") {
                                format.size = size.parse().ok();
                            }

                            if let Some(color) = attribute(b"color") {
                                if color.starts_with(b'#') {
                                    let rval = color
                                        .slice(1..3)
                                        .and_then(|v| u8::from_wstr_radix(v, 16).ok());
                                    let gval = color
                                        .slice(3..5)
                                        .and_then(|v| u8::from_wstr_radix(v, 16).ok());
                                    let bval = color
                                        .slice(5..7)
                                        .and_then(|v| u8::from_wstr_radix(v, 16).ok());

                                    if let (Some(r), Some(g), Some(b)) = (rval, gval, bval) {
                                        format.color = Some(swf::Color { r, g, b, a: 0 });
                                    }
                                }
                            }

                            if let Some(letter_spacing) = attribute(b"letterSpacing") {
                                format.letter_spacing = letter_spacing.parse().ok();
                            }

                            if let Some(kerning) = attribute(b"kerning") {
                                if kerning == WStr::from_units(b"1") {
                                    format.kerning = Some(true);
                                } else if kerning == WStr::from_units(b"0") {
                                    format.kerning = Some(false);
                                }
                            }
                        }
                        b"b" => {
                            format.bold = Some(true);
                        }
                        b"i" => {
                            format.italic = Some(true);
                        }
                        b"u" => {
                            format.underline = Some(true);
                        }
                        b"li" => {
                            format.bullet = Some(true);
                        }
                        b"textformat" => {
                            //TODO: Spec says these are all in twips. That doesn't seem to
                            //match Flash 8.
                            if let Some(left_margin) = attribute(b"leftmargin") {
                                format.left_margin = left_margin.parse().ok();
                            }

                            if let Some(right_margin) = attribute(b"rightmargin") {
                                format.right_margin = right_margin.parse().ok();
                            }

                            if let Some(indent) = attribute(b"indent") {
                                format.indent = indent.parse().ok();
                            }

                            if let Some(block_indent) = attribute(b"blockindent") {
                                format.block_indent = block_indent.parse().ok();
                            }

                            if let Some(leading) = attribute(b"leading") {
                                format.leading = leading.parse().ok();
                            }

                            if let Some(tab_stops) = attribute(b"tabstops") {
                                format.tab_stops = Some(
                                    tab_stops
                                        .split(b',')
                                        .filter_map(|v| v.trim().parse().ok())
                                        .collect(),
                                );
                            }
                        }
                        _ => {}
                    }
                    format_stack.push(format);
                }
                Ok(Event::Text(e)) if !e.is_empty() => {
                    let e = decode_to_wstr(Cow::Borrowed(&e[..]));
                    let e = process_html_entity(&e).unwrap_or(e);
                    let format = format_stack.last().unwrap().clone();
                    text.push_str(&e);
                    spans.push(TextSpan::with_length_and_format(e.len(), format));
                }
                Ok(Event::End(e)) => {
                    match &e.name().to_ascii_lowercase()[..] {
                        b"br" | b"sbr" => {
                            // Skip pop from `format_stack`.
                            continue;
                        }
                        b"p" | b"li" => {
                            text.push_byte(b'\n');
                            if let Some(span) = spans.last_mut() {
                                span.span_length += 1;
                            }
                        }
                        _ => {}
                    }
                    format_stack.pop();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    log::warn!("Error while parsing HTML: {}", e);
                    break;
                }
                _ => {}
            }

            buf.clear();
        }

        Self {
            text,
            displayed_text: WString::new(),
            spans,
            default_format,
        }
    }

    pub fn default_format(&self) -> &TextFormat {
        &self.default_format
    }

    pub fn set_default_format(&mut self, tf: TextFormat) {
        self.default_format = tf.mix_with(self.default_format.clone());
    }

    pub fn hide_text(&mut self) {
        self.displayed_text = WStr::from_units(b"*").repeat(self.text.len());
    }

    pub fn clear_displayed_text(&mut self) {
        self.displayed_text = WString::new();
    }

    pub fn has_displayed_text(&self) -> bool {
        !self.displayed_text.is_empty()
    }

    /// Retrieve the text backing the format spans.
    pub fn text(&self) -> &WStr {
        &self.text
    }

    pub fn displayed_text(&self) -> &WStr {
        if self.has_displayed_text() {
            &self.displayed_text
        } else {
            &self.text
        }
    }

    /// Retrieve the text span at a particular index.
    ///
    /// Text span indices are ephemeral and can change arbitrarily any time the
    /// `FormatSpans` are mutated. You should not use this method directly; the
    /// `iter_spans` method will yield the string and span data directly.
    pub fn span(&self, index: usize) -> Option<&TextSpan> {
        self.spans.get(index)
    }

    /// Find the index of the span that covers a given search position.
    ///
    /// This function returns both the index of the span which covers the
    /// search position, but how far into the span its position is.
    ///
    /// The index returned from this function is not valid across calls which
    /// mutate spans.
    pub fn resolve_position_as_span(&self, search_pos: usize) -> Option<(usize, usize)> {
        let mut position = 0;

        for (index, span) in self.spans.iter().enumerate() {
            if search_pos < position + span.span_length {
                return Some((index, search_pos - position));
            }

            position += span.span_length;
        }

        None
    }

    /// Create a text-span break at a particular position, if one does not
    /// already exist.
    ///
    /// If `search_pos` is out of bounds for the underlying set of spans, then
    /// this function returns `None`.
    ///
    /// The returned index refers to the index of the newly-created span at
    /// `search_pos`. It will be invalidated if another span break is created
    /// before `search_pos` or if the spans are normalized. If you need to
    /// create multiple span breaks, you must either:
    ///
    ///  * Always create spans in order of increasing `search_pos`.
    ///  * Discard the values returned by this function and redundantly resolve
    ///    each span again once all breaks are completed.
    pub fn ensure_span_break_at(&mut self, search_pos: usize) -> Option<usize> {
        if let Some((first_span_pos, break_index)) = self.resolve_position_as_span(search_pos) {
            if break_index == 0 {
                return Some(first_span_pos);
            }

            let first_span = self.spans.get_mut(first_span_pos).unwrap();
            let mut second_span = first_span.clone();
            second_span.span_length = first_span.span_length - break_index;
            first_span.span_length = break_index;

            self.spans.insert(first_span_pos + 1, second_span);

            Some(first_span_pos + 1)
        } else {
            None
        }
    }

    /// Retrieve the range of spans that encompass the text range [from, to).
    ///
    /// The range returned by this function is the clopen set [span_from,
    /// span_to) ready to be sliced as `&spans[span_from..span_to]`.
    ///
    /// The boundaries yielded by this function may extend beyond the text
    /// range, but will always at least encompass all of the text. To get an
    /// exact text range, you must first call `ensure_span_break_at` for both
    /// `from` and `to`.
    ///
    /// The indexes returned from this function is not valid across calls which
    /// mutate spans.
    pub fn get_span_boundaries(&self, from: usize, to: usize) -> (usize, usize) {
        let start_pos = self.resolve_position_as_span(from).unwrap_or((0, 0)).0;
        let end_pos = min(
            self.resolve_position_as_span(to.saturating_sub(1))
                .map(|(pos, i)| (pos.saturating_add(1), i))
                .unwrap_or_else(|| (self.spans.len(), 0))
                .0,
            self.spans.len(),
        );

        (start_pos, end_pos)
    }

    /// Adjust the format spans in several ways to ensure that other function
    /// invariants are upheld.
    ///
    /// The particular variants that `normalize` attempts to uphold are:
    ///
    ///  * The length implied by the list of text spans must match the length
    ///    of the string they are formatting.
    ///  * Adjacent text spans contain different text formats. (Stated
    ///    contrapositively, `normalize` attempts to merge text spans with
    ///    identical formatting.)
    ///  * All text spans have non-zero length, unless the associated string is
    ///    legitimately empty, in which case the span list should be a *single*
    ///    null-length span.
    ///  * There is always at least one text span.
    ///
    /// This function should always be called after mutating text spans in such
    /// a way that might violate the above-mentioned invariants.
    pub fn normalize(&mut self) {
        let mut span_length = 0;
        for span in self.spans.iter() {
            span_length += span.span_length;
        }

        match span_length.cmp(&self.text.len()) {
            Ordering::Less => self.spans.push(TextSpan::with_length_and_format(
                self.text.len() - span_length,
                self.default_format.clone(),
            )),
            Ordering::Greater => {
                let mut deficiency = span_length - self.text.len();
                while deficiency > 0 && !self.spans.is_empty() {
                    let removed_length = {
                        let last = self.spans.last_mut().unwrap();
                        if last.span_length > deficiency {
                            last.span_length -= deficiency;
                            break;
                        } else {
                            last.span_length
                        }
                    };

                    self.spans.pop();
                    deficiency -= removed_length;
                }
            }
            Ordering::Equal => {}
        }

        // Remove leading null-length spans. The null-length span removal in
        // the loop below cannot cope with the situation where the first span
        // is null-length, so we ensure it always gets a span with valid
        // length.
        while self
            .spans
            .get(0)
            .map(|span| span.span_length == 0)
            .unwrap_or(false)
        {
            self.spans.remove(0);
        }

        let mut i = 0;
        while i < self.spans.len().saturating_sub(1) {
            let remove_next = {
                let spans = self.spans.get_mut(i..i + 2).unwrap();

                if spans[0].can_merge(&spans[1]) || spans[1].span_length == 0 {
                    spans[0].span_length += spans[1].span_length;
                    true
                } else {
                    false
                }
            };

            if remove_next {
                self.spans.remove(i + 1);
            } else {
                i += 1;
            }
        }

        // Null span removal can possibly cause the span list to become empty.
        // If that happens, then insert a new span. We don't care if it's a
        // null span at this point.
        if self.spans.is_empty() {
            self.spans.push(TextSpan::with_length_and_format(
                self.text.len(),
                self.default_format.clone(),
            ));
        }
    }

    /// Retrieve a text format covering all of the properties applied to text
    /// from the start index to the end index.
    ///
    /// Any property that differs between spans of text will result in a `None`
    /// in the final text format.
    pub fn get_text_format(&self, from: usize, to: usize) -> TextFormat {
        let (start_pos, end_pos) = self.get_span_boundaries(from, to);
        let mut merged_fmt = if let Some(start_span) = self.spans.get(start_pos) {
            start_span.get_text_format()
        } else {
            return Default::default();
        };

        if let Some(spans) = self.spans.get(start_pos + 1..end_pos) {
            for span in spans {
                merged_fmt = merged_fmt.merge_matching_properties(span.get_text_format());
            }
        }

        merged_fmt
    }

    /// Change some portion of the text to have a particular set of text
    /// attributes.
    pub fn set_text_format(&mut self, from: usize, to: usize, fmt: &TextFormat) {
        self.ensure_span_break_at(from);
        self.ensure_span_break_at(to);

        let (start_pos, end_pos) = self.get_span_boundaries(from, to);

        if let Some(spans) = self.spans.get_mut(start_pos..end_pos) {
            for span in spans {
                span.set_text_format(fmt);
            }
        }

        self.normalize();
    }

    /// Replace the text in the range [from, to) with the contents of `with`.
    ///
    /// Attempts to remove degenerate ranges (e.g. [5, 2)) will fail silently.
    ///
    /// Text span formatting will be adjusted to match: specifically, the spans
    /// corresponding to the range will be removed and replaced with a single
    /// span for the newly inserted text. Its formatting will be determined by
    /// either the formatting of the last span in the range, or if the range
    /// extends beyond the end of the field, the default text format.
    ///
    /// (The text formatting behavior has been confirmed by manual testing with
    /// Flash Player 8.)
    pub fn replace_text(
        &mut self,
        from: usize,
        to: usize,
        with: &WStr,
        new_tf: Option<&TextFormat>,
    ) {
        if to < from {
            return;
        }

        if from < self.text.len() {
            self.ensure_span_break_at(from);
            self.ensure_span_break_at(to);

            let (start_pos, end_pos) = self.get_span_boundaries(from, to);
            let new_tf = new_tf
                .cloned()
                .or_else(|| self.spans.get(end_pos).map(|span| span.get_text_format()))
                .unwrap_or_else(|| self.default_format.clone());

            self.spans.drain(start_pos..end_pos);
            self.spans.insert(
                start_pos,
                TextSpan::with_length_and_format(with.len(), new_tf),
            );
        } else {
            self.spans.push(TextSpan::with_length_and_format(
                with.len(),
                new_tf
                    .cloned()
                    .unwrap_or_else(|| self.default_format.clone()),
            ));
        }

        let mut new_string = WString::new();
        if let Some(text) = self.text.slice(0..from) {
            new_string.push_str(text);
        } else {
            // `get` will fail if `from` exceeds the bounds of the text, rather
            // than just giving all of it to us. In that case, we append the
            // entire string.
            new_string.push_str(&self.text);
        }
        new_string.push_str(with);

        if let Some(text) = self.text.slice(to..) {
            new_string.push_str(text);
        }

        self.text = new_string;

        self.normalize();
    }

    /// Iterate over all text spans in the current list of format spans.
    ///
    /// The iterator returned by this function yields a tuple for each span,
    /// containing the following parameters:
    ///
    /// 1. The position of the first character covered by the span
    /// 2. The end of the span (or, more specifically, the position of the last
    ///    character covered by the span, plus one)
    /// 3. The string contents of the text span
    /// 4. The formatting applied to the text span.
    pub fn iter_spans(&self) -> impl Iterator<Item = (usize, usize, &WStr, &TextSpan)> {
        TextSpanIter::for_format_spans(self)
    }

    #[allow(clippy::float_cmp)]
    pub fn raise_to_html<'gc>(&self, mc: MutationContext<'gc, '_>) -> XmlDocument<'gc> {
        let document = XmlDocument::new(mc);
        let mut root = document.as_node();

        let mut last_span = self.span(0);

        //HTML elements are nested roughly in this order.
        //Some of them nest within themselves, but we only store the last one,
        //as Flash doesn't seem to un-nest them at all.
        let mut last_text_format_element = None;
        let mut last_bullet = None;
        let mut last_paragraph = None;
        let mut last_font = None;
        let mut last_a = None;
        let mut last_b = None;
        let mut last_i = None;
        let mut last_u = None;

        for (start, _end, text, span) in self.iter_spans() {
            let ls = &last_span.unwrap();

            if ls.left_margin != span.left_margin
                || ls.right_margin != span.right_margin
                || ls.indent != span.indent
                || ls.block_indent != span.block_indent
                || ls.leading != span.leading
                || ls.tab_stops != span.tab_stops
                || last_text_format_element.is_none()
            {
                let new_tf = XmlNode::new_element(mc, "TEXTFORMAT".into(), document);

                if ls.left_margin != 0.0 {
                    new_tf.set_attribute_value(
                        mc,
                        XmlName::from_str("LEFTMARGIN"),
                        AvmString::new_utf8(mc, span.left_margin.to_string()),
                    );
                }

                if ls.right_margin != 0.0 {
                    new_tf.set_attribute_value(
                        mc,
                        XmlName::from_str("RIGHTMARGIN"),
                        AvmString::new_utf8(mc, span.right_margin.to_string()),
                    );
                }

                if ls.indent != 0.0 {
                    new_tf.set_attribute_value(
                        mc,
                        XmlName::from_str("INDENT"),
                        AvmString::new_utf8(mc, span.indent.to_string()),
                    );
                }

                if ls.block_indent != 0.0 {
                    new_tf.set_attribute_value(
                        mc,
                        XmlName::from_str("BLOCKINDENT"),
                        AvmString::new_utf8(mc, span.block_indent.to_string()),
                    );
                }

                if ls.leading != 0.0 {
                    new_tf.set_attribute_value(
                        mc,
                        XmlName::from_str("LEADING"),
                        AvmString::new_utf8(mc, span.leading.to_string()),
                    );
                }

                if !ls.tab_stops.is_empty() {
                    let tab_stops = span
                        .tab_stops
                        .iter()
                        .map(f64::to_string)
                        .collect::<Vec<_>>()
                        .join(",");
                    new_tf.set_attribute_value(
                        mc,
                        XmlName::from_str("TABSTOPS"),
                        AvmString::new_utf8(mc, tab_stops),
                    );
                }

                last_text_format_element = Some(new_tf);
                last_bullet = None;
                last_paragraph = None;
                last_font = None;
                last_a = None;
                last_b = None;
                last_i = None;
                last_u = None;

                root.append_child(mc, new_tf).unwrap();
            }

            let mut can_span_create_bullets = start == 0;
            for line in text.split([b'\n', b'\r'].as_ref()) {
                if can_span_create_bullets && span.bullet
                    || !can_span_create_bullets && last_span.map(|ls| ls.bullet).unwrap_or(false)
                {
                    let new_li = XmlNode::new_element(mc, "LI".into(), document);

                    last_bullet = Some(new_li);
                    last_paragraph = None;
                    last_font = None;
                    last_a = None;
                    last_b = None;
                    last_i = None;
                    last_u = None;

                    last_text_format_element
                        .unwrap_or(root)
                        .append_child(mc, new_li)
                        .unwrap();
                }

                if ls.align != span.align || last_paragraph.is_none() {
                    let new_p = XmlNode::new_element(mc, "P".into(), document);
                    let align: &str = match span.align {
                        swf::TextAlign::Left => "LEFT",
                        swf::TextAlign::Center => "CENTER",
                        swf::TextAlign::Right => "RIGHT",
                        swf::TextAlign::Justify => "JUSTIFY",
                    };

                    new_p.set_attribute_value(mc, XmlName::from_str("ALIGN"), align.into());

                    last_bullet
                        .or(last_text_format_element)
                        .unwrap_or(root)
                        .append_child(mc, new_p)
                        .unwrap();
                    last_paragraph = Some(new_p);
                    last_font = None;
                    last_a = None;
                    last_b = None;
                    last_i = None;
                    last_u = None;
                }

                if ls.font != span.font
                    || ls.size != span.size
                    || ls.color != span.color
                    || ls.letter_spacing != span.letter_spacing
                    || ls.kerning != span.kerning
                    || last_font.is_none()
                {
                    let new_font = XmlNode::new_element(mc, "FONT".into(), document);

                    if ls.font != span.font || last_font.is_none() {
                        new_font.set_attribute_value(
                            mc,
                            XmlName::from_str("FACE"),
                            AvmString::new(mc, span.font.clone()),
                        );
                    }

                    if ls.size != span.size || last_font.is_none() {
                        new_font.set_attribute_value(
                            mc,
                            XmlName::from_str("SIZE"),
                            AvmString::new_utf8(mc, span.size.to_string()),
                        );
                    }

                    if ls.color != span.color || last_font.is_none() {
                        let color = format!(
                            "#{:0>2X}{:0>2X}{:0>2X}",
                            span.color.r, span.color.g, span.color.b
                        );
                        new_font.set_attribute_value(
                            mc,
                            XmlName::from_str("COLOR"),
                            AvmString::new_utf8(mc, color),
                        );
                    }

                    if ls.letter_spacing != span.letter_spacing || last_font.is_none() {
                        new_font.set_attribute_value(
                            mc,
                            XmlName::from_str("LETTERSPACING"),
                            AvmString::new_utf8(mc, span.letter_spacing.to_string()),
                        );
                    }

                    if ls.kerning != span.kerning || last_font.is_none() {
                        new_font.set_attribute_value(
                            mc,
                            XmlName::from_str("KERNING"),
                            if span.kerning { "1".into() } else { "0".into() },
                        );
                    }

                    last_font
                        .or(last_paragraph)
                        .or(last_bullet)
                        .or(last_text_format_element)
                        .unwrap_or(root)
                        .append_child(mc, new_font)
                        .unwrap();

                    last_font = Some(new_font);
                    last_a = None;
                    last_b = None;
                    last_i = None;
                    last_u = None;
                }

                if !span.url.is_empty() && (ls.url != span.url || last_a.is_none()) {
                    let new_a = XmlNode::new_element(mc, "A".into(), document);

                    new_a.set_attribute_value(
                        mc,
                        XmlName::from_str("HREF"),
                        AvmString::new(mc, span.url.clone()),
                    );

                    if !span.target.is_empty() {
                        new_a.set_attribute_value(
                            mc,
                            XmlName::from_str("TARGET"),
                            AvmString::new(mc, span.target.clone()),
                        );
                    }

                    last_font
                        .or(last_paragraph)
                        .or(last_bullet)
                        .or(last_text_format_element)
                        .unwrap_or(root)
                        .append_child(mc, new_a)
                        .unwrap();

                    last_b = None;
                    last_i = None;
                    last_u = None;
                } else if span.url.is_empty() && (ls.url != span.url || last_a.is_some()) {
                    last_a = None;
                    last_b = None;
                    last_i = None;
                    last_u = None;
                }

                if span.bold && last_b.is_none() {
                    let new_b = XmlNode::new_element(mc, "B".into(), document);

                    last_a
                        .or(last_font)
                        .or(last_paragraph)
                        .or(last_bullet)
                        .or(last_text_format_element)
                        .unwrap_or(root)
                        .append_child(mc, new_b)
                        .unwrap();

                    last_b = Some(new_b);
                    last_i = None;
                    last_u = None;
                } else if !span.bold && last_b.is_some() {
                    last_b = None;
                    last_i = None;
                    last_u = None;
                }

                if span.italic && last_i.is_none() {
                    let new_i = XmlNode::new_element(mc, "I".into(), document);

                    last_b
                        .or(last_a)
                        .or(last_font)
                        .or(last_paragraph)
                        .or(last_bullet)
                        .or(last_text_format_element)
                        .unwrap_or(root)
                        .append_child(mc, new_i)
                        .unwrap();

                    last_i = Some(new_i);
                    last_u = None;
                } else if !span.italic && last_i.is_some() {
                    last_i = None;
                    last_u = None;
                }

                if span.underline && last_u.is_none() {
                    let new_u = XmlNode::new_element(mc, "U".into(), document);

                    last_i
                        .or(last_b)
                        .or(last_a)
                        .or(last_font)
                        .or(last_paragraph)
                        .or(last_bullet)
                        .or(last_text_format_element)
                        .unwrap_or(root)
                        .append_child(mc, new_u)
                        .unwrap();

                    last_u = Some(new_u);
                } else if !span.underline && last_u.is_some() {
                    last_u = None;
                }

                let span_text = if last_bullet.is_some() {
                    XmlNode::new_text(mc, AvmString::new(mc, line), document)
                } else {
                    let line_start = line.offset_in(text).unwrap();
                    let line_with_newline = if line_start > 0 {
                        text.slice(line_start - 1..line.len() + 1).unwrap_or(line)
                    } else {
                        line
                    };

                    XmlNode::new_text(mc, AvmString::new(mc, line_with_newline), document)
                };

                last_u
                    .or(last_i)
                    .or(last_b)
                    .or(last_a)
                    .or(last_font)
                    .or(last_paragraph)
                    .or(last_bullet)
                    .or(last_text_format_element)
                    .unwrap_or(root)
                    .append_child(mc, span_text)
                    .unwrap();

                last_span = Some(span);
                can_span_create_bullets = true;
            }
        }

        document
    }
}
