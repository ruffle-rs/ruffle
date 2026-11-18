use quick_xml::Error as XmlError;
use quick_xml::escape::{EscapeError as XmlEscapeError, ParseCharRefError};
use quick_xml::events::Event;
use regress::Regex;
use std::str::Utf8Error;
use std::sync::LazyLock;

/// An event read through a [`TextCoalescer`].
pub enum CoalescedEvent<'a> {
    /// A run of text and general references, still escaped.
    Text(Vec<u8>),
    Event(Event<'a>),
}

/// Merges runs of `Event::Text` and `Event::GeneralRef` into one piece of text,
/// like quick-xml reported them before 0.38.
///
/// The text is kept as raw (UTF-8) bytes, as that is what the unescaping functions take.
#[derive(Default)]
pub struct TextCoalescer<'a> {
    lookahead: Option<Result<Event<'a>, XmlError>>,
}

impl<'a> TextCoalescer<'a> {
    pub fn read_event(
        &mut self,
        mut read: impl FnMut() -> Result<Event<'a>, XmlError>,
    ) -> Result<CoalescedEvent<'a>, XmlError> {
        let mut text = Vec::new();

        loop {
            match self.lookahead.take().unwrap_or_else(&mut read) {
                Ok(Event::Text(e)) => text.extend_from_slice(e.as_bytes()),
                Ok(Event::GeneralRef(e)) => {
                    text.push(b'&');
                    text.extend_from_slice(e.as_bytes());
                    text.push(b';');
                }
                other if text.is_empty() => return other.map(CoalescedEvent::Event),
                other => {
                    // Return the text first, and the following event or error next time.
                    self.lookahead = Some(other);
                    return Ok(CoalescedEvent::Text(text));
                }
            }
        }
    }
}

// Matches XML entities like &amp; or &#39; or &#x1F600;
// We exclude both ';' and '&' from the middle to avoid matching across multiple
// ampersands - e.g. in "& &thing;" we want to match only "&thing;", not "& &thing;"
static AVM1_ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&[^;&]*;").unwrap());

// AVM2's E4X parser does not decode entities preceded by a bare '&',
// e.g. "&&amp;" is preserved as-is rather than decoded to "&&".
// We achieve this by only excluding ';' from the middle, so "&&amp;" is
// matched as a single entity, which fails to decode and is preserved.
static AVM2_ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&[^;]*;").unwrap());

#[derive(Clone, Copy, PartialEq, Eq)]
enum Avm {
    Avm1,
    Avm2,
}

/// AVM1 XML unescaping. Decodes entities individually, even when
/// preceded by a bare '&' (e.g. "&&amp;" becomes "&&").
pub fn avm1_unescape(input: &[u8]) -> Result<String, Utf8Error> {
    custom_unescape(input, Avm::Avm1)
}

/// AVM2 E4X XML unescaping. Does not decode entities preceded by a
/// bare '&' (e.g. "&&amp;" is preserved as "&&amp;").
pub fn avm2_unescape(input: &[u8]) -> Result<String, Utf8Error> {
    custom_unescape(input, Avm::Avm2)
}

/// Handles flash-specific XML unescaping behavior.
/// We accept all XML entities, and also accept standalone '&' without
/// a corresponding ';'
fn custom_unescape(input: &[u8], avm: Avm) -> Result<String, Utf8Error> {
    let entity_regex = match avm {
        Avm::Avm1 => &AVM1_ENTITY_REGEX,
        Avm::Avm2 => &AVM2_ENTITY_REGEX,
    };
    let input = std::str::from_utf8(input)?;

    let mut result = String::new();
    let mut last_end = 0;

    // Find all entities, and try to unescape them.
    // Our regular expression will skip over '&' without a matching ';',
    // which will preserve them as-is in the output
    for cap in entity_regex.find_iter(input) {
        let start = cap.start();
        let end = cap.end();

        result.push_str(&input[last_end..start]);

        let entity = &input[start..end];
        // Unfortunately, we need to call this on each entity individually,
        // since it bails out if *any* entities in the string lack a terminating ';'
        match quick_xml::escape::unescape(entity) {
            Ok(decoded) => result.push_str(&decoded),
            Err(XmlEscapeError::InvalidCharRef(ParseCharRefError::IllegalCharacter(0)))
                if avm == Avm::Avm2 =>
            {
                result.push('\0')
            }
            // Unknown or malformed entities are preserved as-is.
            // The AVM2 regex can match strings like "&&amp;" which contain
            // multiple '&' characters, causing unescape to report UnterminatedEntity
            // for the first '&' before it reaches the ';'.
            Err(
                XmlEscapeError::InvalidCharRef(_)
                | XmlEscapeError::UnrecognizedEntity(_, _)
                | XmlEscapeError::UnterminatedEntity(_)
                | XmlEscapeError::TooManyNestedEntities,
            ) => result.push_str(entity),
        }

        last_end = end;
    }

    result.push_str(&input[last_end..]);

    Ok(result)
}
