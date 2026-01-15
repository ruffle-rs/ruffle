use crate::bookmarks::{Bookmark, Bookmarks};
use crate::content::ContentDescriptor;
use crate::parse::{DocumentHolder, ParseContext, ParseDetails, ParseWarning, ReadExt};
use toml_edit::DocumentMut;
use url::Url;

pub fn read_bookmarks(input: &str) -> ParseDetails<Bookmarks> {
    let document = match input.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(e) => {
            return ParseDetails {
                result: Default::default(),
                warnings: vec![ParseWarning::InvalidToml(e)],
            };
        }
    };

    let mut result = Vec::new();
    let mut cx = ParseContext::default();

    document.get_array_of_tables(&mut cx, "bookmark", |cx, bookmarks| {
        for bookmark in bookmarks.iter() {
            let url = match bookmark.parse_from_str(cx, "url") {
                Some(value) => value,
                None => Url::parse(crate::INVALID_URL).expect("Url is constant and valid"),
            };

            #[cfg(feature = "fs")]
            let root_content_path = bookmark
                .parse_from_str::<String>(cx, "dir")
                .map(std::path::PathBuf::from);

            let name = match bookmark.parse_from_str(cx, "name") {
                Some(value) => value,
                // Fallback to using the URL as the name.
                None => crate::url_to_readable_name(&url).into_owned(),
            };

            let content_descriptor = ContentDescriptor {
                url,
                #[cfg(feature = "fs")]
                root_content_path,
            };
            result.push(Bookmark {
                content_descriptor,
                name,
            });
        }
    });

    ParseDetails {
        warnings: cx.warnings,
        result: DocumentHolder::new(result, document),
    }
}

#[expect(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark() {
        let result = read_bookmarks("[bookmark]");
        assert_eq!(&Vec::<Bookmark>::new(), result.values());
        assert_eq!(
            vec![ParseWarning::UnexpectedType {
                expected: "array of tables",
                actual: "table",
                path: "bookmark".to_string()
            }],
            result.warnings
        );

        let result = read_bookmarks("[[bookmark]]");
        assert_eq!(
            &vec![Bookmark {
                content_descriptor: ContentDescriptor::new_remote(
                    Url::parse(crate::INVALID_URL).unwrap(),
                ),
                name: "".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_bookmarks("[[bookmark]]\nurl = \"invalid\"");
        assert_eq!(
            &vec![Bookmark {
                content_descriptor: ContentDescriptor::new_remote(
                    Url::parse(crate::INVALID_URL).unwrap(),
                ),
                name: "".to_string(),
            }],
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "bookmark.url".to_string()
            }],
            result.warnings
        );

        let result = read_bookmarks(
            "[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\nname = \"Logo SWF\"",
        );
        assert_eq!(
            &vec![Bookmark {
                content_descriptor: ContentDescriptor::new_remote(
                    Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                ),
                name: "Logo SWF".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }

    #[test]
    fn multiple_bookmarks() {
        let result = read_bookmarks(
            r#"
            [[bookmark]]
            url = "file:///home/user/example.swf"

            [[bookmark]]
            url = "https://ruffle.rs/logo-anim.swf"
            "#,
        );
        assert_eq!(
            &vec![
                Bookmark {
                    content_descriptor: ContentDescriptor::new_remote(
                        Url::parse("file:///home/user/example.swf").unwrap()
                    ),
                    name: "example.swf".to_string(),
                },
                Bookmark {
                    content_descriptor: ContentDescriptor::new_remote(
                        Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                    ),
                    name: "logo-anim.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_bookmarks(
            r#"
            [[bookmark]]
            url = "file:///home/user/example.swf"

            [[bookmark]]
            url = "invalid"

            [[bookmark]]

            [[bookmark]]
            url = "https://ruffle.rs/logo-anim.swf"
            "#,
        );
        assert_eq!(
            &vec![
                Bookmark {
                    content_descriptor: ContentDescriptor::new_remote(
                        Url::parse("file:///home/user/example.swf").unwrap(),
                    ),
                    name: "example.swf".to_string(),
                },
                Bookmark {
                    content_descriptor: ContentDescriptor::new_remote(
                        Url::parse(crate::INVALID_URL).unwrap(),
                    ),
                    name: "".to_string(),
                },
                Bookmark {
                    content_descriptor: ContentDescriptor::new_remote(
                        Url::parse(crate::INVALID_URL).unwrap(),
                    ),
                    name: "".to_string(),
                },
                Bookmark {
                    content_descriptor: ContentDescriptor::new_remote(
                        Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                    ),
                    name: "logo-anim.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "bookmark.url".to_string()
            }],
            result.warnings
        );
    }

    #[cfg(feature = "fs")]
    #[test]
    fn dir() {
        let result = read_bookmarks(
            r#"
            [[bookmark]]
            url = "file:///home/user/example.swf"
            dir = "/home/user"

            [[bookmark]]
            url = "https://ruffle.rs/logo-anim.swf"
            dir = "/home/user2"
            "#,
        );
        assert_eq!(
            &vec![
                Bookmark {
                    content_descriptor: ContentDescriptor {
                        url: Url::parse("file:///home/user/example.swf").unwrap(),
                        root_content_path: Some(std::path::PathBuf::from("/home/user")),
                    },
                    name: "example.swf".to_string(),
                },
                Bookmark {
                    content_descriptor: ContentDescriptor {
                        url: Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                        root_content_path: Some(std::path::PathBuf::from("/home/user2")),
                    },
                    name: "logo-anim.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);

        let result = read_bookmarks(
            r#"
            [[bookmark]]
            url = "invalid"
            dir = "/home/user"
            "#,
        );
        assert_eq!(
            &vec![Bookmark {
                content_descriptor: ContentDescriptor {
                    url: Url::parse(crate::INVALID_URL).unwrap(),
                    root_content_path: Some(std::path::PathBuf::from("/home/user")),
                },
                name: "".to_string(),
            },],
            result.values()
        );
        assert_eq!(
            vec![ParseWarning::UnsupportedValue {
                value: "invalid".to_string(),
                path: "bookmark.url".to_string()
            }],
            result.warnings
        );
    }
}
