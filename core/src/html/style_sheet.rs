use fnv::FnvHashMap;
use ruffle_wstr::{WStr, WString};
use std::borrow::Cow;

pub type CssProperties<'a> = FnvHashMap<&'a WStr, &'a WStr>;

/// There's very few ways in which Flash thinks CSS is invalid.
/// Whilst it doesn't appear to give any error to the user/developer -
/// this could be useful for debugging.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CssError {
    #[error("Invalid selector syntax: Name cannot contain a space")]
    SpaceInSelectorName,

    #[error("Invalid selector syntax: Expected a property block to start")]
    ExpectedPropertiesBlock,

    #[error("Invalid property syntax: Name cannot contain a space")]
    SpaceInPropertyName,

    #[error("Invalid property syntax: No value specified")]
    PropertyValueMissing,
}

pub struct CssStream<'a> {
    pos: usize,
    input: &'a WStr,
}

const ASTERISK: u16 = '*' as u16;
const OPEN_BLOCK: u16 = '{' as u16;
const CLOSE_BLOCK: u16 = '}' as u16;
const COMMA: u16 = ',' as u16;
const COLON: u16 = ':' as u16;
const SEMI_COLON: u16 = ';' as u16;
const SLASH: u16 = '/' as u16;
const SPACE: u16 = ' ' as u16;
const NEWLINE: u16 = '\n' as u16;
const RETURN: u16 = '\r' as u16;
const TAB: u16 = '\t' as u16;

const ANY_VALID_WHITESPACE: [u16; 4] = [SPACE, NEWLINE, RETURN, TAB];

impl<'a> CssStream<'a> {
    pub fn new(input: &'a WStr) -> Self {
        Self { pos: 0, input }
    }

    fn skip_whitespace_and_comments(&mut self) -> bool {
        let mut found = false;
        while self.skip_comment() || self.consume_any(&ANY_VALID_WHITESPACE).is_some() {
            found = true;
        }
        found
    }

    fn skip_comment(&mut self) -> bool {
        if self.peek2() == Some((SLASH, ASTERISK)) {
            self.pos += 2; // Skip the `/*`
            while self.peek2() != Some((ASTERISK, SLASH)) {
                self.pos += 1;
                if self.peek().is_none() {
                    return false; // EOF without closing comment
                }
            }
            self.pos += 2; // Skip the `*/`
            return true;
        }
        false
    }

    fn peek(&self) -> Option<u16> {
        self.input.get(self.pos)
    }

    fn peek2(&self) -> Option<(u16, u16)> {
        Some((self.input.get(self.pos)?, self.input.get(self.pos + 1)?))
    }

    #[cfg(test)]
    pub(crate) fn pos(&self) -> usize {
        self.pos
    }

    fn consume_any(&mut self, expected: &[u16]) -> Option<u16> {
        let actual = self.peek()?;

        for value in expected {
            if *value == actual {
                self.pos += 1;
                return Some(actual);
            }
        }

        None
    }

    fn consume_until_any(&mut self, expected: &[u16]) -> &'a WStr {
        let start = self.pos;
        while let Some(next) = self.peek() {
            for value in expected {
                if *value == next {
                    return &self.input[start..self.pos];
                }
            }
            self.pos += 1;
        }

        &self.input[start..self.pos]
    }

    /// Parses a map of CSS selectors and properties out from the document.
    /// Expects to be top level (not inside a block), and ends when no more input is available
    /// This is the "main" parsing method
    pub fn parse(&mut self) -> Result<FnvHashMap<&'a WStr, CssProperties<'a>>, CssError> {
        let mut result = FnvHashMap::default();

        loop {
            self.skip_whitespace_and_comments();
            if self.peek().is_none() {
                return Ok(result);
            }

            let selectors = self.parse_selectors()?;
            let properties = self.parse_properties()?;
            for selector in selectors {
                result.insert(selector, properties.clone());
            }
        }
    }

    /// Parses a list of selector names in preparation for an upcoming block
    /// Expects to be top level (not inside a block), and ends when we enter a block ('{')
    pub(crate) fn parse_selectors(&mut self) -> Result<Vec<&'a WStr>, CssError> {
        let mut result = Vec::new();

        loop {
            self.skip_whitespace_and_comments();
            let name = self.consume_until_any(&[OPEN_BLOCK, COMMA, SPACE, NEWLINE, RETURN, TAB]);
            if !name.is_empty() {
                result.push(name);
            }
            self.skip_whitespace_and_comments();

            match self.peek() {
                Some(OPEN_BLOCK) => {
                    self.pos += 1;
                    if result.is_empty() {
                        result.push(WStr::empty());
                    }
                    return Ok(result);
                }
                Some(COMMA) => {
                    self.pos += 1;
                    continue;
                }
                None => return Err(CssError::ExpectedPropertiesBlock),
                _ => return Err(CssError::SpaceInSelectorName),
            }
        }
    }

    /// Parses a list of `key:value` properties from inside a block
    /// Expects to already be inside a block (after the `{`), and ends when either:
    /// - We ended a block ('}')
    /// - There is no more input, and we're not in the middle of a property
    pub(crate) fn parse_properties(&mut self) -> Result<CssProperties<'a>, CssError> {
        let mut result = CssProperties::default();

        'main: loop {
            // [NA] This is a bit awkward:
            // - spaces at the start of a name are skipped
            // - spaces in the middle of a name are errors
            // - spaces at the end of the name are kept

            self.skip_whitespace_and_comments();

            match self.peek() {
                Some(CLOSE_BLOCK) => {
                    self.pos += 1;
                    return Ok(result);
                }
                None => {
                    return Ok(result);
                }
                _ => {}
            }

            let name_start = self.pos;
            let mut name = self.consume_until_any(&[COLON, SPACE, NEWLINE, RETURN, TAB]);

            if self.skip_whitespace_and_comments() {
                let next = self.peek();
                if next == Some(COLON) {
                    // Expand to contain the spaces at the end of a name, this is valid
                    name = &self.input[name_start..self.pos];
                } else if next.is_some() {
                    // Anything else is invalid
                    return Err(CssError::SpaceInPropertyName);
                }
            }

            match self.peek() {
                Some(COLON) => {
                    self.pos += 1;
                }
                None => {
                    return Err(CssError::PropertyValueMissing);
                }
                _ => unreachable!(),
            }

            // Okay, we've got the name sorted, now it's the value!
            self.skip_whitespace_and_comments();

            let value_start = self.pos;
            let mut value = self.consume_until_any(&[SEMI_COLON, COLON, CLOSE_BLOCK]);
            loop {
                match self.peek() {
                    Some(COLON) => {
                        self.pos = value_start;
                        let possible_value = self.consume_until_any(&[NEWLINE, RETURN]);
                        match self.peek() {
                            Some(NEWLINE) | Some(RETURN) => {
                                self.pos += 1;
                                result.insert(name, possible_value);
                                continue 'main;
                            }
                            _ => {
                                self.pos = value_start;
                                value = self.consume_until_any(&[SEMI_COLON, CLOSE_BLOCK]);
                                continue;
                            }
                        }
                    }
                    Some(SEMI_COLON) => {
                        self.pos += 1;
                        result.insert(name, value);
                        continue 'main;
                    }
                    Some(CLOSE_BLOCK) => {
                        self.pos += 1;
                        let mut end_value = value.len();
                        for (index, ch) in value.iter().enumerate() {
                            if [NEWLINE, RETURN].contains(&ch) {
                                end_value = index;
                                break;
                            }
                        }
                        result.insert(name, &value[..end_value]);
                        return Ok(result);
                    }
                    None => return Err(CssError::PropertyValueMissing),
                    _ => unreachable!(),
                }
            }
        }
    }
}

pub fn transform_dashes_to_camel_case(input: &WStr) -> Cow<WStr> {
    if !input.contains(b'-') {
        return Cow::Borrowed(input);
    }
    let mut result = WString::with_capacity(input.len(), input.is_wide());

    let mut make_upper = false;
    let mut pos = 0;
    while let Some(char) = input.get(pos) {
        match char {
            0x002D if !make_upper => make_upper = true, // - as u16, can't use `as` in arm
            _ if make_upper => {
                make_upper = false;
                result.push_str(&input[pos..=pos].to_ascii_uppercase())
            }
            _ => result.push(char),
        }
        pos += 1;
    }

    Cow::Owned(result)
}

// More exhaustive tests live inside avm2 stylesheet swf test
// These are just some useful ones extracted out
#[cfg(test)]
mod tests {
    use super::{CssError, CssStream};
    use fnv::FnvHashMap;
    use ruffle_wstr::WStr;

    #[test]
    fn parse_selectors_single() {
        let mut stream = CssStream::new(WStr::from_units(b"name {"));
        assert_eq!(
            stream.parse_selectors(),
            Ok(vec![WStr::from_units(b"name")])
        );
        assert_eq!(stream.pos(), 6)
    }

    #[test]
    fn parse_selectors_with_comment() {
        let mut stream = CssStream::new(WStr::from_units(b"name /* comment */ {"));
        assert_eq!(
            stream.parse_selectors(),
            Ok(vec![WStr::from_units(b"name")])
        );
        assert_eq!(stream.pos(), 20)
    }

    #[test]
    fn parse_property_without_semicolon() {
        let mut stream = CssStream::new(WStr::from_units(b"a { key: value \r\nkey2:v }"));
        let mut result = FnvHashMap::default();
        result.insert(WStr::from_units(b"a"), {
            let mut properties = FnvHashMap::default();
            properties.insert(WStr::from_units(b"key"), WStr::from_units(b"value "));
            properties.insert(WStr::from_units(b"key2"), WStr::from_units(b"v "));
            properties
        });
        assert_eq!(stream.parse(), Ok(result));
        assert_eq!(stream.pos(), 25)
    }

    #[test]
    fn parse_last_property_without_semicolon() {
        let mut stream = CssStream::new(WStr::from_units(b"a { key: value \r\n }"));
        let mut result = FnvHashMap::default();
        result.insert(WStr::from_units(b"a"), {
            let mut properties = FnvHashMap::default();
            properties.insert(WStr::from_units(b"key"), WStr::from_units(b"value "));
            properties
        });
        assert_eq!(stream.parse(), Ok(result));
        assert_eq!(stream.pos(), 19)
    }

    #[test]
    fn parse_selectors_multiple() {
        let mut stream = CssStream::new(WStr::from_units(b"one,two, three  ,\n\tfour {"));
        assert_eq!(
            stream.parse_selectors(),
            Ok(vec![
                WStr::from_units(b"one"),
                WStr::from_units(b"two"),
                WStr::from_units(b"three"),
                WStr::from_units(b"four")
            ])
        );
        assert_eq!(stream.pos(), 25)
    }

    #[test]
    fn parse_selectors_no_end() {
        let mut stream = CssStream::new(WStr::from_units(b"name"));
        assert_eq!(
            stream.parse_selectors(),
            Err(CssError::ExpectedPropertiesBlock)
        );
    }

    #[test]
    fn parse_selectors_empty() {
        let mut stream = CssStream::new(WStr::from_units(b"{"));
        assert_eq!(stream.parse_selectors(), Ok(vec![WStr::from_units(b"")]));
        assert_eq!(stream.pos(), 1)
    }

    #[test]
    fn parse_selectors_just_end() {
        let mut stream = CssStream::new(WStr::from_units(b"     {"));
        assert_eq!(stream.parse_selectors(), Ok(vec![WStr::from_units(b"")]));
        assert_eq!(stream.pos(), 6)
    }

    #[test]
    fn parse_selectors_invalid() {
        let mut stream = CssStream::new(WStr::from_units(b"name with a space {"));
        assert_eq!(stream.parse_selectors(), Err(CssError::SpaceInSelectorName));
    }

    #[test]
    fn parse_properties_empty() {
        let mut stream = CssStream::new(WStr::from_units(b""));
        assert_eq!(stream.parse_properties(), Ok(FnvHashMap::default()));
        assert_eq!(stream.pos(), 0)
    }

    #[test]
    fn parse_properties_semicolons_are_crazy() {
        let mut stream = CssStream::new(WStr::from_units(b";;key: value;;other: value;}"));
        let mut result = FnvHashMap::default();
        result.insert(WStr::from_units(b";;key"), WStr::from_units(b"value"));
        result.insert(WStr::from_units(b";other"), WStr::from_units(b"value"));
        assert_eq!(stream.parse_properties(), Ok(result));
        assert_eq!(stream.pos(), 28)
    }

    #[test]
    fn parse_properties_cursed() {
        let mut stream = CssStream::new(WStr::from_units(b"name}b{:value;"));
        let mut result = FnvHashMap::default();
        result.insert(WStr::from_units(b"name}b{"), WStr::from_units(b"value"));
        assert_eq!(stream.parse_properties(), Ok(result));
        assert_eq!(stream.pos(), 14)
    }

    #[test]
    fn parse_properties_whitespace() {
        let mut stream = CssStream::new(WStr::from_units(b"  property  :   test    value    ;"));
        let mut result = FnvHashMap::default();
        result.insert(
            WStr::from_units(b"property  "),
            WStr::from_units(b"test    value    "),
        );
        assert_eq!(stream.parse_properties(), Ok(result));
        assert_eq!(stream.pos(), 34)
    }

    #[test]
    fn parse_properties_no_value_eof() {
        let mut stream = CssStream::new(WStr::from_units(b"key:"));
        assert_eq!(
            stream.parse_properties(),
            Err(CssError::PropertyValueMissing)
        );
    }

    #[test]
    fn parse_properties_no_value_explicit_close() {
        let mut stream = CssStream::new(WStr::from_units(b"key:}"));
        let mut result = FnvHashMap::default();
        result.insert(WStr::from_units(b"key"), WStr::from_units(b""));
        assert_eq!(stream.parse_properties(), Ok(result));
        assert_eq!(stream.pos(), 5)
    }

    #[test]
    fn parse_properties_no_value_semicolon() {
        let mut stream = CssStream::new(WStr::from_units(b"key:;"));
        let mut result = FnvHashMap::default();
        result.insert(WStr::from_units(b"key"), WStr::from_units(b""));
        assert_eq!(stream.parse_properties(), Ok(result));
        assert_eq!(stream.pos(), 5)
    }

    #[test]
    fn parse_empty() {
        let mut stream = CssStream::new(WStr::from_units(b""));
        assert_eq!(stream.parse(), Ok(FnvHashMap::default()));
        assert_eq!(stream.pos(), 0)
    }

    #[test]
    fn parse_two_selectors() {
        let mut stream = CssStream::new(WStr::from_units(b"a, b { key: value }"));
        let mut result = FnvHashMap::default();
        result.insert(WStr::from_units(b"a"), {
            let mut properties = FnvHashMap::default();
            properties.insert(WStr::from_units(b"key"), WStr::from_units(b"value "));
            properties
        });
        result.insert(WStr::from_units(b"b"), {
            let mut properties = FnvHashMap::default();
            properties.insert(WStr::from_units(b"key"), WStr::from_units(b"value "));
            properties
        });
        assert_eq!(stream.parse(), Ok(result));
        assert_eq!(stream.pos(), 19)
    }

    #[test]
    fn parse_empty_property_name_and_unclosed() {
        let mut stream = CssStream::new(WStr::from_units(b"a{:"));
        assert_eq!(stream.parse(), Err(CssError::PropertyValueMissing));
    }
}
