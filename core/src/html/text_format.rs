//! Classes that store formatting options

use crate::context::UpdateContext;
use crate::html::iterators::TextSpanIter;
use crate::string::{Integer, SwfStrExt as _, Units, WStr, WString};
use crate::tag_utils::SwfMovie;
use gc_arena::Collect;
use quick_xml::{escape::escape, events::Event, Reader};
use ruffle_wstr::utils::swf_is_newline;
use std::borrow::Cow;
use std::cmp::{min, Ordering};
use std::collections::VecDeque;
use std::fmt::Write;
use std::sync::Arc;

const HTML_NEWLINE: u16 = b'\r' as u16;
const HTML_SPACE: u16 = b' ' as u16;

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

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextDisplay {
    #[default]
    Block,
    Inline,
    None,
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
    pub display: Option<TextDisplay>,
}

impl TextFormat {
    /// Construct a `TextFormat` from an `EditText`'s SWF tag.
    ///
    /// This requires an `UpdateContext` as we will need to retrieve some font
    /// information from the actually-referenced font.
    pub fn from_swf_tag(
        et: swf::EditText<'_>,
        swf_movie: Arc<SwfMovie>,
        context: &mut UpdateContext<'_>,
    ) -> Self {
        let encoding = swf_movie.encoding();
        let movie_library = context.library.library_for_movie_mut(swf_movie);
        let font = et.font_id().and_then(|fid| movie_library.get_font(fid));
        let font_class = et
            .font_class()
            .map(|s| s.decode(encoding).into_owned())
            .or_else(|| font.map(|font| WString::from_utf8(font.descriptor().name())))
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
            bold: if et.is_html() {
                Some(false)
            } else {
                Some(font.map(|font| font.descriptor().bold()).unwrap_or(false))
            },
            italic: if et.is_html() {
                Some(false)
            } else {
                Some(font.map(|font| font.descriptor().italic()).unwrap_or(false))
            },
            underline: Some(false),
            display: Some(TextDisplay::Block),
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
            display: if self.display == rhs.display {
                self.display
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
            display: self.display.or(rhs.display),
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
#[derive(Clone, Debug)]
pub struct TextSpan {
    /// How many characters are subsumed by this text span.
    ///
    /// This value must not cause the resulting set of text spans to exceed the
    /// length of the underlying source string.
    pub span_length: usize,

    pub font: TextSpanFont,
    pub style: TextSpanStyle,
    pub align: swf::TextAlign,
    pub left_margin: f64,
    pub right_margin: f64,
    pub indent: f64,
    pub block_indent: f64,
    pub leading: f64,
    pub tab_stops: Vec<f64>,
    pub bullet: bool,
    pub url: WString,
    pub target: WString,
    pub display: TextDisplay,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextSpanFont {
    pub face: WString,
    pub size: f64,
    pub color: swf::Color,
    pub letter_spacing: f64,
    pub kerning: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextSpanStyle {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

impl Default for TextSpan {
    fn default() -> Self {
        Self {
            span_length: 0,
            font: TextSpanFont::default(),
            style: TextSpanStyle::default(),
            align: swf::TextAlign::Left,
            left_margin: 0.0,
            right_margin: 0.0,
            indent: 0.0,
            block_indent: 0.0,
            leading: 0.0,
            tab_stops: vec![],
            bullet: false,
            url: WString::new(),
            target: WString::new(),
            display: TextDisplay::default(),
        }
    }
}

impl Default for TextSpanFont {
    fn default() -> Self {
        Self {
            face: WString::new(),
            size: 12.0,
            color: swf::Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            kerning: false,
            letter_spacing: 0.0,
        }
    }
}

impl TextSpanFont {
    fn with_format(tf: &TextFormat) -> Self {
        let mut data = Self::default();
        data.set_text_format(tf);
        data
    }

    fn set_text_format(&mut self, tf: &TextFormat) {
        if let Some(font) = &tf.font {
            self.face = font.clone();
        }

        if let Some(size) = &tf.size {
            self.size = *size;
        }

        if let Some(color) = tf.color {
            self.color = color;
        }

        if let Some(kerning) = &tf.kerning {
            self.kerning = *kerning;
        }

        if let Some(letter_spacing) = &tf.letter_spacing {
            self.letter_spacing = *letter_spacing;
        }
    }
}

impl TextSpan {
    pub fn with_length_and_format(length: usize, tf: &TextFormat) -> Self {
        let mut data = Self {
            span_length: length,
            ..Default::default()
        };

        data.set_text_format(tf);

        data
    }

    /// Determine if this and another span have identical text formats.
    ///
    /// It is assumed that the two text spans being considered are adjacent;
    /// and we have no way of checking, so this function doesn't check that.
    fn can_merge(&self, rhs: &Self) -> bool {
        self.font == rhs.font
            && self.style == rhs.style
            && self.align == rhs.align
            && self.left_margin == rhs.left_margin
            && self.right_margin == rhs.right_margin
            && self.indent == rhs.indent
            && self.block_indent == rhs.block_indent
            && self.leading == rhs.leading
            && self.tab_stops == rhs.tab_stops
            && self.bullet == rhs.bullet
            && self.url == rhs.url
            && self.target == rhs.target
            && self.display == rhs.display
    }

    /// Apply a text format to this text span.
    ///
    /// Properties marked `None` on the `TextFormat` will remain unchanged.
    fn set_text_format(&mut self, tf: &TextFormat) {
        if let Some(align) = &tf.align {
            self.align = *align;
        }

        if let Some(bold) = &tf.bold {
            self.style.bold = *bold;
        }

        if let Some(italic) = &tf.italic {
            self.style.italic = *italic;
        }

        if let Some(underline) = &tf.underline {
            self.style.underline = *underline;
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

        if let Some(leading) = &tf.leading {
            self.leading = *leading;
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

        if let Some(display) = tf.display {
            self.display = display;
        }

        self.font.set_text_format(tf);
    }

    /// Convert the text span into a format.
    ///
    /// The text format returned will have all properties defined.
    pub fn get_text_format(&self) -> TextFormat {
        TextFormat {
            font: Some(self.font.face.clone()),
            size: Some(self.font.size),
            color: Some(self.font.color),
            align: Some(self.align),
            bold: Some(self.style.bold),
            italic: Some(self.style.italic),
            underline: Some(self.style.underline),
            left_margin: Some(self.left_margin),
            right_margin: Some(self.right_margin),
            indent: Some(self.indent),
            block_indent: Some(self.block_indent),
            kerning: Some(self.font.kerning),
            leading: Some(self.leading),
            letter_spacing: Some(self.font.letter_spacing),
            tab_stops: Some(self.tab_stops.clone()),
            bullet: Some(self.bullet),
            url: Some(self.url.clone()),
            target: Some(self.target.clone()),
            display: Some(self.display),
        }
    }
}

/// Struct which contains text formatted by `TextSpan`s.
#[derive(Clone, Debug)]
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
            spans: vec![TextSpan::with_length_and_format(len, &format)],
            default_format: format,
        }
    }

    /// Lower an HTML tree into text-span representation.
    ///
    /// This is the "legacy" implementation of this process: it only looks for
    /// a handful of presentational attributes in the HTML tree to generate
    /// styling. There's also a `lower_from_css` that respects both
    /// presentational markup and CSS stylesheets.
    pub fn from_html(
        html: &WStr,
        default_format: TextFormat,
        is_multiline: bool,
        condense_white: bool,
        swf_version: u8,
    ) -> Self {
        // For SWF version 6, the multiline property exists and may be changed,
        // but its value is ignored and fields always behave as multiline.
        let is_multiline = is_multiline || swf_version <= 6;

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
        let mut opened_starts: Vec<usize> = Vec::new();

        // For the weird behaviors of <p>
        let mut p_open = false;
        let mut last_closed_font: Option<TextSpanFont> = None;

        let mut reader = Reader::from_reader(&raw_bytes[..]);
        let reader_config = reader.config_mut();
        reader_config.expand_empty_elements = true;
        reader_config.check_end_names = false;
        reader_config.allow_unmatched_ends = true;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    let tag_name = &e.name().into_inner().to_ascii_lowercase()[..];
                    let attributes: Result<Vec<_>, _> = e.attributes().with_checks(false).collect();
                    let attributes = match attributes {
                        Ok(attributes) => attributes,
                        Err(e) => {
                            tracing::warn!("Error while parsing HTML: {}", e);
                            return Default::default();
                        }
                    };
                    let attribute = move |name| {
                        attributes.iter().find_map(|attribute| {
                            attribute
                                .key
                                .into_inner()
                                .eq_ignore_ascii_case(name)
                                .then(|| decode_to_wstr(&attribute.value))
                        })
                    };
                    let mut format = format_stack.last().unwrap().clone();
                    match tag_name {
                        b"br" => {
                            if is_multiline {
                                text.push(HTML_NEWLINE);
                                spans.push(TextSpan::with_length_and_format(1, &format));
                            }

                            // Skip push to `format_stack`.
                            continue;
                        }
                        b"sbr" => {
                            // TODO: <sbr> tags do not add a newline, but rather only break
                            // the format span.
                            text.push(HTML_NEWLINE);
                            spans.push(TextSpan::with_length_and_format(1, &format));

                            // Skip push to `format_stack`.
                            continue;
                        }
                        b"p" => {
                            p_open = true;
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
                                // if the number starts with + or -, the size is relative
                                let (prefix, size) = if size.starts_with(&[b'+', b'-'][..]) {
                                    (Some(size.at(0) as u8), &size[1..])
                                } else {
                                    (None, &size[..])
                                };

                                // text is ignored from the first non-numeric character
                                // (including the decimal separator)
                                let first_foreign_char = size
                                    .find(|c| c < '0' as u16 || c > '9' as u16)
                                    .unwrap_or(size.len());
                                let size = &size[0..first_foreign_char];

                                let size: Option<f64> = size.parse().ok();

                                if let Some(size) = size
                                    .and_then(|size| match prefix {
                                        Some(b'+') => format.size.map(|last_size| last_size + size),
                                        Some(b'-') => format.size.map(|last_size| last_size - size),
                                        _ => Some(size),
                                    })
                                    .map(|size| {
                                        if swf_version < 13 {
                                            size.clamp(1.0, 127.0)
                                        } else {
                                            size.max(1.0)
                                        }
                                    })
                                {
                                    format.size = Some(size);
                                } else {
                                    // malformed sizes are ignored
                                }
                            }

                            if let Some(color) = attribute(b"color") {
                                // FIXME - handle alpha
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
                                if kerning == WStr::from_units(b"1") && swf_version >= 8 {
                                    // Enabling kerning works only for SWF >=8
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
                            let is_last_nl = text.iter().last() == Some(HTML_NEWLINE);
                            if is_multiline && !is_last_nl && text.len() > 0 {
                                // If the last paragraph was not closed and
                                // there was some text since then,
                                // we need to close it here.
                                text.push(HTML_NEWLINE);
                                spans.push(TextSpan::with_length_and_format(
                                    1,
                                    format_stack.last().unwrap(),
                                ));
                            }
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
                    opened_starts.push(opened_buffer.len());
                    opened_buffer.extend(tag_name);
                    format_stack.push(format);
                }
                Ok(Event::Text(e)) if !e.is_empty() => 'text: {
                    let e = decode_to_wstr(&e.into_inner());
                    let e = process_html_entity(&e).unwrap_or(e);
                    let format = format_stack.last().unwrap().clone();
                    if swf_version <= 7 && e.trim().is_empty() {
                        // SWFs version 6,7 ignore whitespace-only text.
                        // But whitespace is preserved when there
                        // is any non-whitespace character.
                        break 'text;
                    }
                    let e = if condense_white {
                        Self::condense_white_in_text(e)
                    } else {
                        e.replace(swf_is_newline, WStr::from_units(&[HTML_NEWLINE]))
                    };
                    text.push_str(&e);
                    spans.push(TextSpan::with_length_and_format(e.len(), &format));
                }
                Ok(Event::End(e)) => {
                    let tag_name = &e.name().into_inner().to_ascii_lowercase()[..];
                    // Check for a mismatch.
                    match opened_starts.last() {
                        Some(start) => {
                            if tag_name != &opened_buffer[*start..] {
                                continue;
                            } else {
                                opened_buffer.truncate(*start);
                                opened_starts.pop();
                            }
                        }
                        None => continue,
                    }

                    match tag_name {
                        b"br" | b"sbr" => {
                            // Skip pop from `format_stack`.
                            continue;
                        }
                        b"li" if is_multiline => {
                            text.push(HTML_NEWLINE);
                            spans.push(TextSpan::with_length_and_format(
                                1,
                                format_stack.last().unwrap(),
                            ));
                        }
                        b"p" if is_multiline => 'p: {
                            if !p_open {
                                // Skip multiple </p>'s without <p>
                                break 'p;
                            }
                            p_open = false;

                            text.push(HTML_NEWLINE);
                            let mut span =
                                TextSpan::with_length_and_format(1, format_stack.last().unwrap());
                            // </p> has some weird behaviors related to the format of its children (b,i,u,a),
                            // sometimes it resets it, sometimes it uses the format from within <p>.
                            // That's probably some relic from FP's internal implementation.
                            // Not that it would matter -- it affects generating empty tags only
                            // and it should not be problematic.
                            span.style = TextSpanStyle::default();
                            span.url = WString::new();
                            span.target = WString::new();
                            if let Some(last_closed_font) = last_closed_font.clone() {
                                // </p> uses the font from the last </font> if available
                                // does not make any sense but everything seems that's the case
                                span.font = last_closed_font;
                            } else {
                                span.font = TextSpanFont::with_format(&default_format);
                            }
                            spans.push(span);
                        }
                        b"font" => {
                            let tf = format_stack.last().unwrap();
                            last_closed_font = Some(TextSpanFont::with_format(tf));
                        }
                        _ => {}
                    }
                    format_stack.pop();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    tracing::warn!("Error while parsing HTML: {}", e);
                    break;
                }
                _ => {}
            }
        }

        let mut ret = Self {
            text,
            displayed_text: WString::new(),
            spans,
            default_format,
        };
        if condense_white && swf_version >= 8 {
            ret.condense_white_swf8();
        }
        ret.normalize();
        ret
    }

    fn condense_white_in_text(string: WString) -> WString {
        let mut result = WString::with_capacity(string.len(), string.is_wide());
        let mut last_white = false;
        for ch in string.iter() {
            if ruffle_wstr::utils::swf_is_whitespace(ch) {
                if !last_white {
                    result.push(HTML_SPACE);
                    last_white = true;
                }
            } else {
                result.push(ch);
                last_white = false;
            }
        }
        result
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

    /// SWF8+ condenses whitespace not only in text, but across HTML elements too.
    /// This method assumes that whitespace in text has already been condensed.
    fn condense_white_swf8(&mut self) {
        let mut removal_start = Some(0);
        let mut to_remove = Vec::new();
        for (i, ch) in self.text().iter().enumerate() {
            let is_newline = ch == HTML_NEWLINE;
            let is_space = ch == HTML_SPACE;

            // We have to preserve newlines here, as newlines inputted in text
            // are already condensed into space.
            if is_newline || !is_space {
                if let Some(space_start) = removal_start {
                    to_remove.push((space_start, i));
                }
                removal_start = None;
            }

            // However, HTML newlines are also considered as space here.
            if (is_newline || is_space) && removal_start.is_none() {
                removal_start = Some(i + 1);
            }
        }
        if let Some(space_start) = removal_start {
            to_remove.push((space_start, self.text().len()));
        }
        for &(from, to) in to_remove.iter().rev() {
            if from != to {
                self.replace_text(from, to, WStr::empty(), None);
            }
        }
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
                &self.default_format,
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
                &self.default_format,
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
                TextSpan::with_length_and_format(with.len(), &new_tf),
            );
        } else {
            self.spans.push(TextSpan::with_length_and_format(
                with.len(),
                new_tf.unwrap_or(&self.default_format),
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
        if self.text.is_empty() {
            return WString::new();
        }

        let mut state = FormatState {
            result: WString::new(),
            font_stack: VecDeque::new(),
            current_span: &TextSpan::default(),
            open_tags: Vec::new(),
        };

        let spans = self.iter_spans();

        for (_start, _end, text, span) in spans {
            state.set_span(span);
            state.push_text(text);
        }

        state.close_all_tags();
        state.result
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
enum HtmlTag {
    Textformat,
    P,
    Li,
    Font,
    A,
    B,
    I,
    U,

    Br,
    Sbr,
}

impl HtmlTag {
    fn closeable(&self) -> bool {
        self != &Self::Br && self != &Self::Sbr
    }
}

/// Holds required state for HTML formatting.
struct FormatState<'a> {
    result: WString,
    font_stack: VecDeque<&'a TextSpanFont>,
    current_span: &'a TextSpan,
    open_tags: Vec<HtmlTag>,
}

impl<'a> FormatState<'a> {
    /// Set a new span. Tags are automatically adapted to the new span.
    fn set_span(&mut self, span: &'a TextSpan) {
        // When there's a difference in style, all style tags
        // need to be closed and later reopened if necessary.
        if span.style != self.current_span.style {
            self.close_tags_till(HtmlTag::B);
        }

        if span.url != self.current_span.url {
            self.close_tags_till(HtmlTag::A);
        }

        self.close_font_if_feasible(&span.font);

        self.current_span = span;

        if self.current_span.left_margin != 0.0
            || self.current_span.right_margin != 0.0
            || self.current_span.indent != 0.0
            || self.current_span.leading != 0.0
            || self.current_span.block_indent != 0.0
            || !self.current_span.tab_stops.is_empty()
        {
            self.open_tag(HtmlTag::Textformat);
        }

        if !self.open_tags.contains(&HtmlTag::P) && !self.open_tags.contains(&HtmlTag::Li) {
            if self.current_span.bullet {
                self.open_tag(HtmlTag::Li);
            } else {
                self.open_tag(HtmlTag::P);
            }
        }

        self.set_font(&self.current_span.font);

        if !self.current_span.url.is_empty() {
            self.open_tag(HtmlTag::A);
        }

        if self.current_span.style.bold {
            self.open_tag(HtmlTag::B);
        }

        if self.current_span.style.italic {
            self.open_tag(HtmlTag::I);
        }

        if self.current_span.style.underline {
            self.open_tag(HtmlTag::U);
        }
    }

    fn open_tag(&mut self, tag: HtmlTag) {
        if tag.closeable() {
            if self.open_tags.contains(&tag) {
                // Tag already opened.
                return;
            }
            if self
                .open_tags
                .last()
                .map(|last| *last > tag)
                .unwrap_or(false)
            {
                // A child tag is already opened.
                // Order of tags must be ensured.
                return;
            }

            self.open_tags.push(tag);
        }

        match tag {
            HtmlTag::Textformat => {
                self.result.push_str(WStr::from_units(b"<TEXTFORMAT"));
                if self.current_span.left_margin != 0.0 {
                    let _ = write!(
                        self.result,
                        " LEFTMARGIN=\"{}\"",
                        self.current_span.left_margin,
                    );
                }
                if self.current_span.right_margin != 0.0 {
                    let _ = write!(
                        self.result,
                        " RIGHTMARGIN=\"{}\"",
                        self.current_span.right_margin,
                    );
                }
                if self.current_span.indent != 0.0 {
                    let _ = write!(self.result, " INDENT=\"{}\"", self.current_span.indent);
                }
                if self.current_span.leading != 0.0 {
                    let _ = write!(self.result, " LEADING=\"{}\"", self.current_span.leading);
                }
                if self.current_span.block_indent != 0.0 {
                    let _ = write!(
                        self.result,
                        " BLOCKINDENT=\"{}\"",
                        self.current_span.block_indent
                    );
                }
                if !self.current_span.tab_stops.is_empty() {
                    let _ = write!(
                        self.result,
                        " TABSTOPS=\"{}\"",
                        self.current_span
                            .tab_stops
                            .iter()
                            .map(f64::to_string)
                            .collect::<Vec<_>>()
                            .join(",")
                    );
                }
                self.result.push_byte(b'>');
            }
            HtmlTag::P => {
                let _ = write!(
                    self.result,
                    "<P ALIGN=\"{}\">",
                    match self.current_span.align {
                        swf::TextAlign::Left => "LEFT",
                        swf::TextAlign::Center => "CENTER",
                        swf::TextAlign::Right => "RIGHT",
                        swf::TextAlign::Justify => "JUSTIFY",
                    }
                );
            }
            HtmlTag::B => {
                self.result.push_str(WStr::from_units(b"<B>"));
            }
            HtmlTag::I => {
                self.result.push_str(WStr::from_units(b"<I>"));
            }
            HtmlTag::U => {
                self.result.push_str(WStr::from_units(b"<U>"));
            }
            HtmlTag::Li => {
                self.result.push_str(WStr::from_units(b"<LI>"));
            }
            HtmlTag::A => {
                let _ = write!(
                    self.result,
                    "<A HREF=\"{}\" TARGET=\"{}\">",
                    self.current_span.url, self.current_span.target
                );
            }
            HtmlTag::Br => {
                self.result.push_str(WStr::from_units(b"<BR>"));
            }
            HtmlTag::Sbr => {
                self.result.push_str(WStr::from_units(b"<SBR>"));
            }
            // Opening <font> is slightly different. See set_font
            HtmlTag::Font => unreachable!(),
        }
    }

    fn set_font(&mut self, font: &'a TextSpanFont) {
        if let Some(&last_font) = self.font_stack.back() {
            if last_font == font {
                return;
            }

            self.close_tags_till(HtmlTag::A);

            self.result.push_str(WStr::from_units(b"<FONT"));
            if font.face != last_font.face {
                let _ = write!(self.result, " FACE=\"{}\"", font.face);
            }
            if font.size != last_font.size {
                let _ = write!(self.result, " SIZE=\"{}\"", font.size);
            }
            if font.color != last_font.color {
                let _ = write!(
                    self.result,
                    " COLOR=\"#{:0>2X}{:0>2X}{:0>2X}\"",
                    font.color.r, font.color.g, font.color.b
                );
            }
            if font.letter_spacing != last_font.letter_spacing {
                let _ = write!(self.result, " LETTERSPACING=\"{}\"", font.letter_spacing);
            }
            if font.kerning != last_font.kerning {
                let _ = write!(
                    self.result,
                    " KERNING=\"{}\"",
                    if font.kerning { "1" } else { "0" }
                );
            }
            self.result.push_byte(b'>');
            self.font_stack.push_back(font);
        } else {
            self.close_tags_till(HtmlTag::A);
            let _ = write!(
                self.result,
                "<FONT FACE=\"{}\" SIZE=\"{}\" COLOR=\"#{:0>2X}{:0>2X}{:0>2X}\" LETTERSPACING=\"{}\" KERNING=\"{}\">",
                font.face,
                font.size,
                font.color.r,
                font.color.g,
                font.color.b,
                font.letter_spacing,
                if font.kerning { "1" } else { "0" },
            );
            self.font_stack.push_back(font);
            self.open_tags.push(HtmlTag::Font);
        }
    }

    fn close_font_if_feasible(&mut self, font: &'a TextSpanFont) {
        let pos = self.font_stack.iter().position(|&f| f == font);
        if let Some(pos) = pos {
            if pos == self.font_stack.len() - 1 {
                return;
            }
            self.close_tags_till(HtmlTag::A);
            self.result
                .push_str(&WStr::from_units(b"</FONT>").repeat(self.font_stack.len() - pos - 1));
            self.font_stack.drain(pos + 1..);
        }
    }

    fn close_all_tags(&mut self) {
        self.close_tags_till(HtmlTag::Textformat);
    }

    /// Close the given tag and all its children if open.
    fn close_tags_till(&mut self, tag: HtmlTag) {
        while self.open_tags.last() >= Some(&tag) {
            let tag = self.open_tags.pop().unwrap();
            self.close_tag(tag);
        }
    }

    fn close_tag(&mut self, tag: HtmlTag) {
        if tag == HtmlTag::Font {
            self.result
                .push_str(&WStr::from_units(b"</FONT>").repeat(self.font_stack.len()));
            self.font_stack.clear();
            return;
        }

        self.result.push_str(match tag {
            HtmlTag::Textformat => WStr::from_units(b"</TEXTFORMAT>"),
            HtmlTag::P => WStr::from_units(b"</P>"),
            HtmlTag::Li => WStr::from_units(b"</LI>"),
            HtmlTag::B => WStr::from_units(b"</B>"),
            HtmlTag::I => WStr::from_units(b"</I>"),
            HtmlTag::U => WStr::from_units(b"</U>"),
            HtmlTag::A => WStr::from_units(b"</A>"),
            _ => unreachable!(),
        });
    }

    fn push_text(&mut self, text: &WStr) {
        let (text, ends_with_nl) = if text.ends_with(swf_is_newline) {
            (&text[0..text.len() - 1], true)
        } else {
            (text, false)
        };

        let mut first = true;
        for text in text.split(swf_is_newline) {
            if !first {
                self.close_all_tags();
                // Ensure that tags are open after closing them.
                self.set_span(self.current_span);
            } else {
                first = false;
            }
            self.push_line(text);
        }

        if ends_with_nl {
            self.close_all_tags();
        }
    }

    fn push_line(&mut self, line: &WStr) {
        if line.is_empty() {
            return;
        }

        let encoded = line.to_utf8_lossy();
        let escaped = escape(&encoded);

        if let Cow::Borrowed(_) = &encoded {
            // Optimization: if the utf8 conversion was a no-op, we know the text is ASCII;
            // escaping special characters cannot insert new non-ASCII characters, so we can
            // simply append the bytes directly without converting from UTF8.
            self.result.push_str(WStr::from_units(escaped.as_bytes()));
        } else {
            // TODO: updating our quick_xml fork to upstream will allow removing this UTF8 check.
            let escaped =
                std::str::from_utf8(escaped.as_bytes()).expect("escaped text should be utf8");
            self.result.push_utf8(escaped);
        }
    }
}
