use quick_xml::escape::EscapeError as XmlEscapeError;
use regress::Regex;
use std::{ops::Deref, sync::LazyLock};

// Matches XML entities like &amp; or &#39; or &#x1F600;
// We exclude both ';' and '&' from the middle to avoid matching across multiple
// ampersands - e.g. in "& &thing;" we want to match only "&thing;", not "& &thing;"
static ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&[^;&]*;").unwrap());

/// Handles flash-specific XML unescaping behavior.
/// We accept all XML entities, and also accept standalone '&' without
/// a corresponding ';'
pub fn custom_unescape(input: &[u8]) -> Result<String, std::str::Utf8Error> {
    let input = std::str::from_utf8(input)?;

    let re = ENTITY_REGEX.deref();
    let mut result = String::new();
    let mut last_end = 0;

    // Find all entities, and try to unescape them.
    // Our regular expression will skip over '&' without a matching ';',
    // which will preserve them as-is in the output
    for cap in re.find_iter(input) {
        let start = cap.start();
        let end = cap.end();

        result.push_str(&input[last_end..start]);

        let entity = &input[start..end];
        // Unfortunately, we need to call this on each entity individually,
        // since it bails out if *any* entities in the string lack a terminating ';'
        match quick_xml::escape::unescape(entity) {
            Ok(decoded) => result.push_str(&decoded),
            Err(err) => match err {
                XmlEscapeError::InvalidCharRef(_) | XmlEscapeError::UnrecognizedEntity(_, _) => {
                    result.push_str(entity)
                }
                // The regex guarantees a semicolon
                XmlEscapeError::UnterminatedEntity(_) => unreachable!(),
            },
        }

        last_end = end;
    }

    result.push_str(&input[last_end..]);

    Ok(result)
}
