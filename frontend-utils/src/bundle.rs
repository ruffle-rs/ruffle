use crate::bundle::info::{
    BundleInformation, BundleInformationParseError, BUNDLE_INFORMATION_FILENAME,
};
use crate::bundle::source::BundleSource;
use crate::parse::ParseWarning;
use std::path::Path;

pub mod info;
pub mod source;

#[derive(Debug, thiserror::Error)]
pub enum BundleError {
    #[error("Invalid ruffle-bundle.toml")]
    InvalidBundleInformation(#[from] BundleInformationParseError),

    #[error("Missing or corrupt ruffle-bundle.toml")]
    MissingBundleInformation,

    #[error("Invalid bundle source")]
    InvalidSource(#[from] source::BundleSourceError),

    #[error("Bundle does not exist")]
    BundleDoesntExist,
}

pub struct Bundle {
    source: BundleSource,
    information: BundleInformation,
    warnings: Vec<ParseWarning>,
}

impl Bundle {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Bundle, BundleError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(BundleError::BundleDoesntExist);
        }
        let source = BundleSource::from_path(path)?;
        let info_file = source
            .read_file(BUNDLE_INFORMATION_FILENAME)
            .map_err(|_| BundleError::MissingBundleInformation)?;
        let info_text =
            String::from_utf8(info_file).map_err(|_| BundleError::MissingBundleInformation)?;
        let information = BundleInformation::parse(&info_text)?;

        Ok(Bundle {
            source,
            information: information.result.take(),
            warnings: information.warnings,
        })
    }

    pub fn source(&self) -> &BundleSource {
        &self.source
    }

    pub fn warnings(&self) -> &[ParseWarning] {
        &self.warnings
    }

    pub fn information(&self) -> &BundleInformation {
        &self.information
    }
}

#[cfg(test)]
mod tests {
    use crate::bundle::info::{
        BundleInformation, BundleInformationParseError, BUNDLE_INFORMATION_FILENAME,
    };
    use crate::bundle::source::BundleSourceError;
    use crate::bundle::{Bundle, BundleError};
    use crate::parse::ParseWarning;
    use tempfile::tempdir;
    use url::Url;

    #[test]
    fn from_path_nonexistent() {
        assert!(matches!(
            Bundle::from_path("/this/path/likely/doesnt/exist"),
            Err(BundleError::BundleDoesntExist)
        ))
    }

    #[test]
    fn from_path_directory_without_bundle() {
        let tmp_dir = tempdir().unwrap();
        let result = Bundle::from_path(tmp_dir.path());
        drop(tmp_dir);
        assert!(matches!(
            result,
            Err(BundleError::InvalidSource(BundleSourceError::UnknownSource))
        ))
    }

    #[test]
    fn from_path_directory_with_folder_as_info() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::create_dir(tmp_dir.path().join(BUNDLE_INFORMATION_FILENAME));
        let result = Bundle::from_path(tmp_dir.path());
        drop(tmp_dir);
        assert!(matches!(
            result,
            Err(BundleError::InvalidSource(BundleSourceError::UnknownSource))
        ))
    }

    #[test]
    fn from_path_directory_with_binary_info() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::write(
            tmp_dir.path().join(BUNDLE_INFORMATION_FILENAME),
            [0, 159, 146, 150],
        );
        let result = Bundle::from_path(tmp_dir.path());
        drop(tmp_dir);
        assert!(matches!(result, Err(BundleError::MissingBundleInformation)))
    }

    #[test]
    fn from_path_directory_with_bad_toml() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::write(tmp_dir.path().join(BUNDLE_INFORMATION_FILENAME), "???");
        let result = Bundle::from_path(tmp_dir.path());
        drop(tmp_dir);
        assert!(matches!(
            result,
            Err(BundleError::InvalidBundleInformation(
                BundleInformationParseError::InvalidToml(_)
            ))
        ))
    }

    #[test]
    fn from_path_directory_valid() {
        let tmp_dir = tempdir().unwrap();
        let _ = std::fs::write(
            tmp_dir.path().join(BUNDLE_INFORMATION_FILENAME),
            r#"
                [bundle]
                name = "Cool Game!"
                url = "file:///game.swf"
                "#,
        );
        let result = Bundle::from_path(tmp_dir.path());
        drop(tmp_dir);
        let result = result.unwrap();
        assert_eq!(
            BundleInformation {
                name: "Cool Game!".to_string(),
                url: Url::parse("file:///game.swf").unwrap(),
                player: Default::default(),
            },
            result.information
        );
        assert_eq!(Vec::<ParseWarning>::new(), result.warnings);
    }
}
