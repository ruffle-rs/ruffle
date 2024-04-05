use crate::bookmarks::{Bookmark, Bookmarks, INVALID_URL};
use crate::parse::{DocumentHolder, ParseContext, ParseDetails, ReadExt};
use toml_edit::DocumentMut;
use url::Url;

pub fn read_bookmarks(input: &str) -> ParseDetails<Bookmarks> {
    let document = match input.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(e) => {
            return ParseDetails {
                result: Default::default(),
                warnings: vec![format!("Invalid TOML: {e}")],
            }
        }
    };

    let mut result = Vec::new();
    let mut cx = ParseContext::default();

    document.get_array_of_tables(&mut cx, "bookmark", |cx, bookmarks| {
        for bookmark in bookmarks.iter() {
            let url = match bookmark.parse_from_str(cx, "url") {
                Some(value) => value,
                None => Url::parse(INVALID_URL).expect("Url is constant and valid"),
            };

            let name = match bookmark.parse_from_str(cx, "name") {
                Some(value) => value,
                // Fallback to using the URL as the name.
                None => crate::url_to_readable_name(&url).into_owned(),
            };

            result.push(Bookmark { url, name });
        }
    });

    ParseDetails {
        result: DocumentHolder::new(result, document),
        warnings: cx.warnings,
    }
}

#[allow(clippy::unwrap_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bookmark() {
        let result = read_bookmarks("[bookmark]");
        assert_eq!(&Vec::<Bookmark>::new(), result.values());
        assert_eq!(
            vec!["Invalid bookmark: expected array of tables but found table".to_string()],
            result.warnings
        );

        let result = read_bookmarks("[[bookmark]]");
        assert_eq!(
            &vec![Bookmark {
                url: Url::parse(INVALID_URL).unwrap(),
                name: "".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);

        let result = read_bookmarks("[[bookmark]]\nurl = \"invalid\"");
        assert_eq!(
            &vec![Bookmark {
                url: Url::parse(INVALID_URL).unwrap(),
                name: "".to_string(),
            }],
            result.values()
        );
        assert_eq!(
            vec!["Invalid bookmark.url: unsupported value \"invalid\"".to_string()],
            result.warnings
        );

        let result = read_bookmarks(
            "[[bookmark]]\nurl = \"https://ruffle.rs/logo-anim.swf\"\nname = \"Logo SWF\"",
        );
        assert_eq!(
            &vec![Bookmark {
                url: Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                name: "Logo SWF".to_string(),
            }],
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);
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
                    url: Url::parse("file:///home/user/example.swf").unwrap(),
                    name: "example.swf".to_string(),
                },
                Bookmark {
                    url: Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                    name: "logo-anim.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(Vec::<String>::new(), result.warnings);

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
                    url: Url::parse("file:///home/user/example.swf").unwrap(),
                    name: "example.swf".to_string(),
                },
                Bookmark {
                    url: Url::parse(INVALID_URL).unwrap(),
                    name: "".to_string(),
                },
                Bookmark {
                    url: Url::parse(INVALID_URL).unwrap(),
                    name: "".to_string(),
                },
                Bookmark {
                    url: Url::parse("https://ruffle.rs/logo-anim.swf").unwrap(),
                    name: "logo-anim.swf".to_string(),
                }
            ],
            result.values()
        );
        assert_eq!(
            vec!["Invalid bookmark.url: unsupported value \"invalid\"".to_string()],
            result.warnings
        );
    }
}
