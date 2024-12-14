use crate::parse::{DocumentHolder, ParseContext, ParseDetails, ReadExt};
use crate::player_options::{read_player_options, PlayerOptions};
use toml_edit::DocumentMut;
use url::Url;

pub const BUNDLE_INFORMATION_FILENAME: &str = "ruffle-bundle.toml";

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum BundleInformationParseError {
    #[error("File is not valid TOML")]
    InvalidToml(#[from] toml_edit::TomlError),

    #[error("Invalid or missing [bundle] section")]
    InvalidBundleSection,

    #[error("Invalid or missing bundle.name")]
    InvalidName,

    #[error("Invalid or missing bundle.url")]
    InvalidUrl,
}

#[derive(Debug, PartialEq)]
pub struct BundleInformation {
    pub name: String,
    pub url: Url,

    pub player: PlayerOptions,
}

impl BundleInformation {
    pub fn parse(
        input: &str,
    ) -> Result<ParseDetails<BundleInformation>, BundleInformationParseError> {
        let document = input.parse::<DocumentMut>()?;

        let mut cx = ParseContext::default();

        let (name, url) = document
            .get_table_like(&mut cx, "bundle", |cx, bundle| {
                let Some(name) = bundle.parse_from_str(cx, "name") else {
                    return Err(BundleInformationParseError::InvalidName);
                };
                let Some(url) = bundle.parse_from_str(cx, "url") else {
                    return Err(BundleInformationParseError::InvalidUrl);
                };

                Ok((name, url))
            })
            .unwrap_or(Err(BundleInformationParseError::InvalidBundleSection))?;

        let player_options = document.get_table_like(&mut cx, "player", |cx, table| {
            read_player_options(cx, table)
        });

        Ok(ParseDetails {
            warnings: cx.warnings,
            result: DocumentHolder::new(
                BundleInformation {
                    name,
                    url,
                    player: player_options.unwrap_or_default(),
                },
                document,
            ),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::bundle::info::{BundleInformation, BundleInformationParseError};
    use crate::parse::ParseWarning;
    use crate::player_options::PlayerOptions;
    use ruffle_core::PlayerRuntime;
    use url::Url;

    fn read(
        input: &str,
    ) -> Result<(BundleInformation, Vec<ParseWarning>), BundleInformationParseError> {
        BundleInformation::parse(input).map(|details| (details.result.take(), details.warnings))
    }

    #[test]
    fn invalid_toml() {
        // [NA] Can't construct TomlError to be able to test this properly
        assert!(matches!(
            read("???"),
            Err(BundleInformationParseError::InvalidToml(_))
        ));
    }

    #[test]
    fn empty() {
        assert_eq!(
            read(""),
            Err(BundleInformationParseError::InvalidBundleSection)
        )
    }

    #[test]
    fn missing_name() {
        assert_eq!(
            read("[bundle]"),
            Err(BundleInformationParseError::InvalidName)
        )
    }

    #[test]
    fn invalid_name() {
        assert_eq!(
            read(
                r#"
                [bundle]
                name = 1234
                "#
            ),
            Err(BundleInformationParseError::InvalidName)
        )
    }

    #[test]
    fn missing_url() {
        assert_eq!(
            read(
                r#"
                [bundle]
                name = "Cool Game!"
                "#
            ),
            Err(BundleInformationParseError::InvalidUrl)
        )
    }

    #[test]
    fn invalid_url_type() {
        assert_eq!(
            read(
                r#"
                [bundle]
                name = "Cool Game!"
                url = 1234
                "#
            ),
            Err(BundleInformationParseError::InvalidUrl)
        )
    }

    #[test]
    fn invalid_url_value() {
        assert_eq!(
            read(
                r#"
                [bundle]
                name = "Cool Game!"
                url = "invalid"
                "#
            ),
            Err(BundleInformationParseError::InvalidUrl)
        )
    }

    #[test]
    fn minimally_valid() {
        assert_eq!(
            read(
                r#"
                [bundle]
                name = "Cool Game!"
                url = "file:///game.swf"
                "#
            ),
            Ok((
                BundleInformation {
                    name: "Cool Game!".to_string(),
                    url: Url::parse("file:///game.swf").unwrap(),
                    player: Default::default(),
                },
                vec![]
            ))
        )
    }

    #[test]
    fn valid_with_player_options() {
        assert_eq!(
            read(
                r#"
            [bundle]
            name = "Player Options Example"
            url = "file:///example.swf"

            [player]
            frame_rate = 15.0
            upgrade_http_to_https = true
            runtime = "air"

            [player.parameters]
            value1 = "Hello"
            value2 = "World!"
            "#
            ),
            Ok((
                BundleInformation {
                    name: "Player Options Example".to_string(),
                    url: Url::parse("file:///example.swf").unwrap(),
                    player: PlayerOptions {
                        parameters: vec![
                            ("value1".to_string(), "Hello".to_string()),
                            ("value2".to_string(), "World!".to_string())
                        ],
                        upgrade_to_https: Some(true),
                        player_runtime: Some(PlayerRuntime::AIR),
                        frame_rate: Some(15.0),
                        ..Default::default()
                    }
                },
                vec![]
            ))
        )
    }
}
