use regress::Regex;
use std::{ops::Deref, sync::LazyLock};

static ENTITY_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"&[^;]*;").unwrap());

/// Handles flash-specific XML unescaping behavior.
/// We accept all XML entities, and also accept standalone '&' without
/// a corresponding ';'
pub fn custom_unescape(
    data: &[u8],
    decoder: quick_xml::Decoder,
) -> Result<String, quick_xml::Error> {
    let input = decoder.decode(data)?;

    let re = ENTITY_REGEX.deref();
    let mut result = String::new();
    let mut last_end = 0;

    // Find all entities, and try to unescape them.
    // Our regular expression will skip over '&' without a matching ';',
    // which will preserve them as-is in the output
    for cap in re.find_iter(&input) {
        let start = cap.start();
        let end = cap.end();
        result.push_str(&input[last_end..start]);

        let entity = &input[start..end];
        // Unfortunately, we need to call this on each entity individually,
        // since it bails out if *any* entities in the string lack a terminating ';'
        match quick_xml::escape::unescape(entity) {
            Ok(decoded) => result.push_str(&decoded),
            // FIXME - check the actual error once https://github.com/tafia/quick-xml/pull/584 is merged
            Err(_) => result.push_str(entity),
        }

        last_end = end;
    }

    result.push_str(&input[last_end..]);
    Ok(result)
}
