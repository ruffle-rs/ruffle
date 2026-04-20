use quick_xml::escape::EscapeError as XmlEscapeError;
use regress::Regex;
use std::str::Utf8Error;
use std::sync::LazyLock;

// Matches XML entities like &amp; or &#39; or &#x1F600;
// We exclude both ';' and '&' from the middle to avoid matching across multiple
// ampersands - e.g. in "& &thing;" we want to match only "&thing;", not "& &thing;"
static AVM1_ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&[^;&]*;").unwrap());

// AVM2's E4X parser does not decode entities preceded by a bare '&',
// e.g. "&&amp;" is preserved as-is rather than decoded to "&&".
// We achieve this by only excluding ';' from the middle, so "&&amp;" is
// matched as a single entity, which fails to decode and is preserved.
static AVM2_ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&[^;]*;").unwrap());

/// AVM1 XML unescaping. Decodes entities individually, even when
/// preceded by a bare '&' (e.g. "&&amp;" becomes "&&").
pub fn avm1_unescape(input: &[u8]) -> Result<String, Utf8Error> {
    custom_unescape(input, &AVM1_ENTITY_REGEX)
}

/// AVM2 E4X XML unescaping. Does not decode entities preceded by a
/// bare '&' (e.g. "&&amp;" is preserved as "&&amp;").
pub fn avm2_unescape(input: &[u8]) -> Result<String, Utf8Error> {
    custom_unescape(input, &AVM2_ENTITY_REGEX)
}

/// Handles flash-specific XML unescaping behavior.
/// We accept all XML entities, and also accept standalone '&' without
/// a corresponding ';'
fn custom_unescape(input: &[u8], entity_regex: &Regex) -> Result<String, Utf8Error> {
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
            // Unknown or malformed entities are preserved as-is.
            // The AVM2 regex can match strings like "&&amp;" which contain
            // multiple '&' characters, causing unescape to report UnterminatedEntity
            // for the first '&' before it reaches the ';'.
            Err(
                XmlEscapeError::InvalidCharRef(_)
                | XmlEscapeError::UnrecognizedEntity(_, _)
                | XmlEscapeError::UnterminatedEntity(_),
            ) => result.push_str(entity),
        }

        last_end = end;
    }

    result.push_str(&input[last_end..]);

    Ok(result)
}
