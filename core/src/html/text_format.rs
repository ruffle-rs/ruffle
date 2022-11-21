//! Classes that store formatting options

use crate::context::UpdateContext;
use crate::html::iterators::TextSpanIter;
use crate::string::{Integer, Units, WStr, WString};
use crate::tag_utils::SwfMovie;
use gc_arena::Collect;
use quick_xml::{escape::escape, events::Event, Reader};
use std::borrow::Cow;
use std::cmp::{min, Ordering};
use std::collections::VecDeque;
use std::fmt::Write;
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
        let font = et.font_id().and_then(|fid| movie_library.get_font(fid));
        let font_class = et
            .font_class()
            .map(|s| WString::from_utf8(&s.to_string_lossy(encoding)))
            .or_else(|| font.map(|font| WString::from_utf8(font.descriptor().class())))
            .unwrap_or_else(|| WString::from_utf8("Times New Roman"));
        let align = et.layout().map(|l| l.align);
        let left_margin = et.layout().map(|l| l.left_margin.to_pixels());
        let right_margin = et.layout().map(|l| l.right_margin.to_pixels());
        let indent = et.layout().map(|l| l.indent.to_pixels());
        let leading = et.layout().map(|l| l.leading.to_pixels());

        // TODO: Text fields that don't specify a font are assumed to be 12px
        // Times New Roman non-bold, non-italic. This will need to be revised
        // when we start supporting device fonts.
        Self {
            font: Some(font_class),
            size: et.height().map(|h| h.to_pixels()),
            color: et
                .color()
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
            bullet: Some(self.bullet),
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
        Self {
            text: WString::new(),
            displayed_text: WString::new(),
            spans: vec![TextSpan::default()],
            default_format: TextFormat::default(),
        }
    }

    /// Construct a format span from its raw parts.
    #[allow(dead_code)]
    pub fn from_str_and_spans(text: &WStr, spans: &[TextSpan]) -> Self {
        Self {
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
    pub fn from_html(html: &WStr, default_format: TextFormat, is_multiline: bool) -> Self {
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
                false,
            ),
        };

        // Helper function to decode a byte sequence returned by quick_xml into a WString.
        // TODO: use Cow<'_, WStr>?
        let decode_to_wstr = |raw: &[u8]| -> WString {
            if is_raw_latin1 {
                WString::from_buf(raw.to_vec())
            } else {
                let utf8 = std::str::from_utf8(raw).expect("raw should be valid utf8");
                WString::from_utf8(utf8)
            }
        };

        // Flash ignores mismatched end tags (i.e. end tags with a missing/different corresponding
        // start tag). `quick-xml` checks end tag mismatches by default, but it cannot recover after
        // encountering one. Thus, we disable `quick-xml`'s check and do it ourselves in a similar
        // manner, but in a recoverable way.
        let mut opened_buffer: Vec<u8> = Vec::new();
        let mut opened_starts = Vec::new();

        let mut reader = Reader::from_reader(&raw_bytes[..]);
        reader.expand_empty_elements(true);
        reader.check_end_names(false);
        let mut buf = Vec::new();
        loop {
            buf.clear();
            match reader.read_event(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    opened_starts.push(opened_buffer.len());
                    opened_buffer.extend(e.name());

                    let attributes: Result<Vec<_>, _> = e.attributes().with_checks(false).collect();
                    let attributes = match attributes {
                        Ok(attributes) => attributes,
                        Err(e) => {
                            log::warn!("Error while parsing HTML: {}", e);
                            return Default::default();
                        }
                    };
                    let attribute = move |name| {
                        attributes.iter().find_map(|attribute| {
                            attribute
                                .key
                                .eq_ignore_ascii_case(name)
                                .then(|| decode_to_wstr(&attribute.value))
                        })
                    };
                    let mut format = format_stack.last().unwrap().clone();
                    match &e.name().to_ascii_lowercase()[..] {
                        b"br" => {
                            if is_multiline {
                                text.push_byte(b'\n');
                                if let Some(span) = spans.last_mut() {
                                    span.span_length += 1;
                                }
                            }

                            // Skip push to `format_stack`.
                            continue;
                        }
                        b"sbr" => {
                            // TODO: <sbr> tags do not add a newline, but rather only break
                            // the format span.
                            text.push_byte(b'\n');
                            if let Some(span) = spans.last_mut() {
                                span.span_length += 1;
                            }

                            // Skip push to `format_stack`.
                            continue;
                        }
                        b"p" if is_multiline => {
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
                        b"li" if is_multiline => {
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
                    let e = decode_to_wstr(e.escaped());
                    let e = process_html_entity(&e).unwrap_or(e);
                    let format = format_stack.last().unwrap().clone();
                    text.push_str(&e);
                    spans.push(TextSpan::with_length_and_format(e.len(), format));
                }
                Ok(Event::End(e)) => {
                    // Check for a mismatch.
                    match opened_starts.last() {
                        Some(start) => {
                            if e.name() != &opened_buffer[*start..] {
                                continue;
                            } else {
                                opened_buffer.truncate(*start);
                                opened_starts.pop();
                            }
                        }
                        None => continue,
                    }

                    match &e.name().to_ascii_lowercase()[..] {
                        b"br" | b"sbr" => {
                            // Skip pop from `format_stack`.
                            continue;
                        }
                        b"p" | b"li" if is_multiline => {
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

    /// Retrieve the last text span.
    ///
    /// Text span indices are ephemeral and can change arbitrarily any time the
    /// `FormatSpans` are mutated. You should not use this method directly; the
    /// `iter_spans` method will yield the string and span data directly.
    pub fn last_span(&self) -> Option<&TextSpan> {
        self.spans.last()
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
    pub fn iter_spans(&self) -> TextSpanIter {
        TextSpanIter::for_format_spans(self)
    }

    pub fn to_html(&self) -> WString {
        let mut spans = self.iter_spans();
        let mut state = if let Some((_start, _end, text, span)) = spans.next() {
            let mut state = FormatState {
                result: WString::new(),
                font_stack: VecDeque::new(),
                span,
                is_open: false,
            };
            state.push_text(text);
            state
        } else {
            return WString::new();
        };

        for (_start, _end, text, span) in spans {
            state.set_span(span);
            state.push_text(text);
        }

        state.close_tags();
        state.result
    }
}

/// Holds required state for HTML formatting.
struct FormatState<'a> {
    result: WString,
    font_stack: VecDeque<&'a TextSpan>,
    span: &'a TextSpan,
    is_open: bool,
}

impl<'a> FormatState<'a> {
    fn open_tags(&mut self) {
        if self.is_open {
            return;
        }

        if self.span.left_margin != 0.0
            || self.span.right_margin != 0.0
            || self.span.indent != 0.0
            || self.span.leading != 0.0
            || self.span.block_indent != 0.0
            || !self.span.tab_stops.is_empty()
        {
            self.result.push_str(WStr::from_units(b"<TEXTFORMAT"));
            if self.span.left_margin != 0.0 {
                let _ = write!(self.result, " LEFTMARGIN=\"{}\"", self.span.left_margin);
            }
            if self.span.right_margin != 0.0 {
                let _ = write!(self.result, " RIGHTMARGIN=\"{}\"", self.span.right_margin);
            }
            if self.span.indent != 0.0 {
                let _ = write!(self.result, " INDENT=\"{}\"", self.span.indent);
            }
            if self.span.leading != 0.0 {
                let _ = write!(self.result, " LEADING=\"{}\"", self.span.leading);
            }
            if self.span.block_indent != 0.0 {
                let _ = write!(self.result, " BLOCKINDENT=\"{}\"", self.span.block_indent);
            }
            if !self.span.tab_stops.is_empty() {
                let _ = write!(
                    self.result,
                    " TABSTOPS=\"{}\"",
                    self.span
                        .tab_stops
                        .iter()
                        .map(f64::to_string)
                        .collect::<Vec<_>>()
                        .join(",")
                );
            }
            self.result.push_byte(b'>');
        }

        if self.span.bullet {
            self.result.push_str(WStr::from_units(b"<LI>"));
        } else {
            let _ = write!(
                self.result,
                "<P ALIGN=\"{}\">",
                match self.span.align {
                    swf::TextAlign::Left => "LEFT",
                    swf::TextAlign::Center => "CENTER",
                    swf::TextAlign::Right => "RIGHT",
                    swf::TextAlign::Justify => "JUSTIFY",
                }
            );
        }

        let _ = write!(
            self.result,
            "<FONT FACE=\"{}\" SIZE=\"{}\" COLOR=\"#{:0>2X}{:0>2X}{:0>2X}\" LETTERSPACING=\"{}\" KERNING=\"{}\">",
            self.span.font,
            self.span.size,
            self.span.color.r,
            self.span.color.g,
            self.span.color.b,
            self.span.letter_spacing,
            if self.span.kerning { "1" } else { "0" },
        );
        self.font_stack.push_front(self.span);

        if !self.span.url.is_empty() {
            let _ = write!(
                self.result,
                "<A HREF=\"{}\" TARGET=\"{}\">",
                self.span.url, self.span.target
            );
        }

        if self.span.bold {
            self.result.push_str(WStr::from_units(b"<B>"));
        }

        if self.span.italic {
            self.result.push_str(WStr::from_units(b"<I>"));
        }

        if self.span.underline {
            self.result.push_str(WStr::from_units(b"<U>"));
        }

        self.is_open = true;
    }

    fn close_tags(&mut self) {
        if !self.is_open {
            return;
        }

        if self.span.underline {
            self.result.push_str(WStr::from_units(b"</U>"));
        }

        if self.span.italic {
            self.result.push_str(WStr::from_units(b"</I>"));
        }

        if self.span.bold {
            self.result.push_str(WStr::from_units(b"</B>"));
        }

        if !self.span.url.is_empty() {
            self.result.push_str(WStr::from_units(b"</A>"));
        }

        self.result
            .push_str(&WStr::from_units(b"</FONT>").repeat(self.font_stack.len()));
        self.font_stack.clear();

        if self.span.bullet {
            self.result.push_str(WStr::from_units(b"</LI>"));
        } else {
            self.result.push_str(WStr::from_units(b"</P>"));
        }

        if self.span.left_margin != 0.0
            || self.span.right_margin != 0.0
            || self.span.indent != 0.0
            || self.span.leading != 0.0
            || self.span.block_indent != 0.0
            || !self.span.tab_stops.is_empty()
        {
            self.result.push_str(WStr::from_units(b"</TEXTFORMAT>"));
        }

        self.is_open = false;
    }

    fn set_span(&mut self, span: &'a TextSpan) {
        if !span.underline && self.span.underline {
            self.result.push_str(WStr::from_units(b"</U>"));
        }

        if !span.italic && self.span.italic {
            self.result.push_str(WStr::from_units(b"</I>"));
        }

        if !span.bold && self.span.bold {
            self.result.push_str(WStr::from_units(b"</B>"));
        }

        if span.url != self.span.url && !self.span.url.is_empty() {
            self.result.push_str(WStr::from_units(b"</A>"));
        }

        if span.font != self.span.font
            || span.size != self.span.size
            || span.color != self.span.color
            || span.letter_spacing != self.span.letter_spacing
            || span.kerning != self.span.kerning
        {
            let pos = self.font_stack.iter().position(|font| {
                span.font == font.font
                    && span.size == font.size
                    && span.color == font.color
                    && span.letter_spacing == font.letter_spacing
                    && span.kerning == font.kerning
            });
            if let Some(pos) = pos {
                self.result
                    .push_str(&WStr::from_units(b"</FONT>").repeat(pos));
                self.font_stack.drain(0..pos);
            } else {
                self.result.push_str(WStr::from_units(b"<FONT"));
                if span.font != self.span.font {
                    let _ = write!(self.result, " FACE=\"{}\"", span.font);
                }
                if span.size != self.span.size {
                    let _ = write!(self.result, " SIZE=\"{}\"", span.size);
                }
                if span.color != self.span.color {
                    let _ = write!(
                        self.result,
                        " COLOR=\"#{:0>2X}{:0>2X}{:0>2X}\"",
                        span.color.r, span.color.g, span.color.b
                    );
                }
                if span.letter_spacing != self.span.letter_spacing {
                    let _ = write!(self.result, " LETTERSPACING=\"{}\"", span.letter_spacing);
                }
                if span.kerning != self.span.kerning {
                    let _ = write!(
                        self.result,
                        " KERNING=\"{}\"",
                        if span.kerning { "1" } else { "0" }
                    );
                }
                self.result.push_byte(b'>');
                self.font_stack.push_front(span);
            }
        }

        if span.url != self.span.url && !span.url.is_empty() {
            let _ = write!(
                self.result,
                "<A HREF=\"{}\" TARGET=\"{}\">",
                span.url, span.target
            );
        }

        if span.bold && !self.span.bold {
            self.result.push_str(WStr::from_units(b"<B>"));
        }

        if span.italic && !self.span.italic {
            self.result.push_str(WStr::from_units(b"<I>"));
        }

        if span.underline && !self.span.underline {
            self.result.push_str(WStr::from_units(b"<U>"));
        }

        self.span = span;
    }

    fn push_text(&mut self, text: &WStr) {
        for (i, text) in text.split(&[b'\n', b'\r'][..]).enumerate() {
            self.open_tags();
            if i > 0 {
                self.close_tags();
            }
            let encoded = text.to_utf8_lossy();
            let escaped = escape(encoded.as_bytes());
            self.result.push_str(WStr::from_units(&*escaped));
        }
    }
}
