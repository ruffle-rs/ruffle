use crate::parse::{DocumentHolder, ParseContext, ParseDetails, ParseWarning, ReadExt};
use crate::recents::{Recent, Recents};
use toml_edit::DocumentMut;
use url::Url;

pub fn read_recents(input: &str) -> ParseDetails<Recents> {
    let document = match input.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(e) => {
            return ParseDetails {
                result: Default::default(),
                warnings: vec![ParseWarning::InvalidToml(e)],
            }
        }
    };

    let mut result = Vec::new();
    let mut cx = ParseContext::default();

    document.get_array_of_tables(&mut cx, "recent", |cx, recents| {
        for recent in recents.iter() {
            let url = match recent.parse_from_str(cx, "url") {
                Some(url) => url,
                None => Url::parse(crate::INVALID_URL).expect("Url is constant and valid"),
            };

            let name = recent
                .parse_from_str(cx, "name")
                .unwrap_or_else(|| crate::url_to_readable_name(&url).into_owned());

            result.push(Recent { url, name });
        }
    });

    ParseDetails {
        warnings: cx.warnings,
        result: DocumentHolder::new(result, document),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let result = read_recents("");
        assert_eq!(&Vec::<Recent>::new(), result.values());
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn invalid_array_type() {
        let result = read_recents("[recent]");
        assert_eq!(&Vec::<Recent>::new(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "array of tables",
                actual: "table",
                path: "recent".to_string()
            }],
            result.warnings
        );
    }

    #[test]
    fn empty_entry() {
        let result = read_recents("[[recent]]");
        assert_eq!(
            &vec![Recent {
                url: Url::parse(crate::INVALID_URL).unwrap(),
                name: "".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn invalid_url() {
        let result = read_recents("[[recent]]\nurl = \"invalid\"");
        assert_eq!(
            &vec![Recent {
                url: Url::parse(crate::INVALID_URL).unwrap(),
                name: "".to_string()
            }],
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "recent.url".to_string()
            }],
            result.warnings,
        );
    }

    #[test]
    fn valid_entry() {
        let result = read_recents("[[recent]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\n");
        assert_eq!(
            &vec![Recent {
                url: Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                name: "logo-anim.swf".to_string()
            }],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn name() {
        let result = read_recents(
            "[[recent]]\nurl = \"file:///name_test.swf\"\nname = \"This is not a test!\"",
        );

        assert_eq!(
            &vec![Recent {
                url: Url::parse("file:///name_test.swf").unwrap(),
                name: "This is not a test!".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn multiple() {
        let result = read_recents(
            r#"
            [[recent]]
            url = "file:///first.swf"

            [[recent]]
            url = "file:///second.swf"
        "#,
        );
        assert_eq!(
            &vec![
                Recent {
                    url: Url::parse("file:///first.swf").unwrap(),
                    name: "first.swf".to_string()
                },
                Recent {
                    url: Url::parse("file:///second.swf").unwrap(),
                    name: "second.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn multiple_with_invalid_entries() {
        let result = read_recents(
            r#"
            [[recent]]
            url = "file:///first.swf"

            [[recent]]

            [[recent]]
            url = 10

            [[recent]]
            url = "yes"

            [[recent]]
            url = "file:///second.swf"
        "#,
        );
        assert_eq!(
            &vec![
                Recent {
                    url: Url::parse("file:///first.swf").unwrap(),
                    name: "first.swf".to_string()
                },
                Recent {
                    url: Url::parse(crate::INVALID_URL).unwrap(),
                    name: "".to_string()
                },
                Recent {
                    url: Url::parse(crate::INVALID_URL).unwrap(),
                    name: "".to_string()
                },
                Recent {
                    url: Url::parse(crate::INVALID_URL).unwrap(),
                    name: "".to_string()
                },
                Recent {
                    url: Url::parse("file:///second.swf").unwrap(),
                    name: "second.swf".to_string(),
                },
            ],
            result.values()
        );
        assert_eq!(
            vec![
                ParseWarning::UnexpectedType {
                    expected: "string",
                    actual: "integer",
                    path: "recent.url".to_string()
                },
                ParseWarning::UnsupportedValue {
                    value: "yes".to_string(),
                    path: "recent.url".to_string()
                }
            ],
            result.warnings
        );
    }
}
