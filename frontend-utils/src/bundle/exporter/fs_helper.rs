use std::path::{Path, PathBuf};

use thiserror::Error;
use url::Url;

use crate::bundle::exporter::{BundleExportError, BundleExporter};
use crate::bundle::info::BundleInformation;

#[derive(Debug, Error)]
pub enum FilesystemHelperError {
    #[error("Some provided paths are not absolute")]
    NonAbsolutePaths,

    #[error("Provided files do not have a common directory")]
    NoCommonDirectory,

    #[error("Provided a local path, but there are no files to export")]
    NoFilesToExport,

    #[error("Provided path is outside of the common directory: {0}")]
    PathOutOfCommonDirectory(PathBuf),

    #[error("Path not representable as URL: {0}")]
    PathNotRepresentableAsUrl(PathBuf),

    #[error("Error exporting bundle: {0}")]
    ErrorExportingBundle(#[from] BundleExportError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct FilesystemHelper<P: AsRef<Path>> {
    root_dir: Option<PathBuf>,
    files_to_export: Vec<P>,
}

impl<P: AsRef<Path>> FilesystemHelper<P> {
    pub fn new(files_to_export: Vec<P>) -> Result<Self, FilesystemHelperError> {
        let root_dir = Self::calculate_common_prefix(&files_to_export)?;

        Ok(Self {
            root_dir,
            files_to_export,
        })
    }

    fn calculate_common_prefix(files: &[P]) -> Result<Option<PathBuf>, FilesystemHelperError> {
        let Some(first_file) = files.first() else {
            return Ok(None);
        };

        let mut common_prefix = first_file
            .as_ref()
            .parent()
            .ok_or(FilesystemHelperError::NoCommonDirectory)?
            .to_owned();

        for file in files {
            if !file.as_ref().is_absolute() {
                return Err(FilesystemHelperError::NonAbsolutePaths);
            }

            while !file.as_ref().starts_with(&common_prefix) {
                common_prefix = common_prefix
                    .parent()
                    .ok_or(FilesystemHelperError::NoCommonDirectory)?
                    .to_owned();
            }
        }

        Ok(Some(common_prefix))
    }

    pub fn real_path_to_bundle_path<'b>(
        &self,
        real_path: &'b Path,
    ) -> Result<&'b Path, FilesystemHelperError> {
        let Some(root_path) = self.root_dir.as_ref() else {
            return Err(FilesystemHelperError::NoFilesToExport);
        };

        real_path
            .strip_prefix(root_path)
            .map_err(|_| FilesystemHelperError::PathOutOfCommonDirectory(real_path.to_owned()))
    }

    pub fn real_path_to_bundle_url(&self, real_path: &Path) -> Result<Url, FilesystemHelperError> {
        let bundle_path = self.real_path_to_bundle_path(real_path)?;
        let bundle_path = PathBuf::from("/").join(bundle_path);

        #[cfg(unix)]
        return Url::from_file_path(&bundle_path)
            .map_err(|_| FilesystemHelperError::PathNotRepresentableAsUrl(real_path.to_owned()));

        // Non-unix paths are weird, just manually create a file:///a/b/c URL
        #[cfg(not(unix))]
        {
            fn path_to_url_non_unix(path: &Path) -> Result<Url, ()> {
                let mut url_path = String::new();
                for component in path.components() {
                    let component = component.as_os_str().to_str().ok_or(())?;
                    url_path.push('/');
                    url_path.push_str(component);
                }
                let mut url = Url::parse("file://").unwrap();
                url.set_path(&url_path);
                Ok(url)
            }

            return path_to_url_non_unix(&bundle_path).map_err(|_| {
                FilesystemHelperError::PathNotRepresentableAsUrl(real_path.to_owned())
            });
        }
    }

    pub fn write_files<W: std::io::Write + std::io::Seek>(
        &self,
        exporter: &mut BundleExporter<W>,
    ) -> Result<(), FilesystemHelperError> {
        for file in &self.files_to_export {
            let path = self.real_path_to_bundle_path(file.as_ref())?;
            exporter.write_content(path, &mut std::fs::File::open(file)?)?;
        }

        Ok(())
    }

    pub fn export_bundle(
        &self,
        mut info: BundleInformation,
        output: &Path,
    ) -> Result<(), FilesystemHelperError> {
        let file = std::fs::File::create(output)?;

        if let Ok(root_movie_path) = info.url.to_file_path() {
            info.url = self.real_path_to_bundle_url(&root_movie_path)?;
        }

        let mut exporter = BundleExporter::new(file, info);
        self.write_files(&mut exporter)?;
        exporter.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use url::Url;

    use crate::bundle::Bundle;
    use crate::bundle::exporter::fs_helper::FilesystemHelper;
    use crate::bundle::exporter::fs_helper::FilesystemHelperError;
    use crate::bundle::info::BundleInformation;
    use crate::bundle::source::BundleSource;
    use crate::player_options::PlayerOptions;

    #[cfg(not(windows))]
    fn new_pathbuf(string_path: &str) -> PathBuf {
        PathBuf::from(string_path)
    }

    #[cfg(windows)]
    fn new_pathbuf(string_path: &str) -> PathBuf {
        PathBuf::from(format!("d:{}", string_path))
    }

    fn tempdir() -> tempfile::TempDir {
        tempfile::tempdir().expect("Failed to create a temp directory for tests")
    }

    fn new_tmp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> PathBuf {
        let path = dir.path().join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Cannot create dirs for a tmp file")
        }
        std::fs::write(&path, content).expect("Cannot write a tmp file");
        path
    }

    fn url(url: &str) -> Url {
        Url::parse(url).expect("url in test should parse")
    }

    fn url_from_path(path: &Path) -> Url {
        Url::from_file_path(path).expect("Path should be representable as URL")
    }

    fn open_bundle(path: &Path) -> Bundle {
        Bundle::from_path(path).expect("Bundle should exist and be valid")
    }

    fn bundle_content_to_str(bundle: &Bundle, file_name: &str) -> String {
        let content = bundle
            .source()
            .read_content(file_name)
            .unwrap_or_else(|e| panic!("Failed reading {file_name}: {e}"));
        let content_str = str::from_utf8(&content).expect("File content should be UTF8");
        content_str.to_owned()
    }

    macro_rules! assert_bundle_file_names_eq {
        ($bundle:expr, $file_names:expr) => {
            let BundleSource::ZipFile(zip_source) = $bundle.source() else {
                panic!("Expected bundle to be a zip file");
            };
            assert_eq!(zip_source.file_names(), $file_names);
        };
    }

    macro_rules! assert_bundle_file_eq {
        ($bundle:expr, $file_name:expr, $expected_content:expr) => {
            assert_eq!(
                bundle_content_to_str(&$bundle, $file_name),
                $expected_content
            );
        };
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_basic() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            Some(new_pathbuf("/a/b/c"))
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_advanced() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            Some(new_pathbuf("/a/b"))
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_root() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
            new_pathbuf("/u/b/g/h/file5"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            Some(new_pathbuf("/"))
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_single_file() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/d/file1"),
        ];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            Some(new_pathbuf("/a/b/c/d"))
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_no_files() {
        let files_to_export: Vec<PathBuf> = vec![];

        assert_eq!(
            FilesystemHelper::calculate_common_prefix(&files_to_export).unwrap(),
            None
        );
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_relative() {
        let files_to_export = vec![
            //
            new_pathbuf("a/b/c/file1"),
            new_pathbuf("a/b/c/file2"),
        ];

        assert!(matches!(
            FilesystemHelper::calculate_common_prefix(&files_to_export),
            Err(FilesystemHelperError::NonAbsolutePaths)
        ));
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_mixed1() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("a/b/c/file2"),
        ];

        assert!(matches!(
            FilesystemHelper::calculate_common_prefix(&files_to_export),
            Err(FilesystemHelperError::NonAbsolutePaths)
        ));
    }

    #[test]
    fn test_calculate_common_prefix_root_dir_mixed2() {
        let files_to_export = vec![
            //
            new_pathbuf("a/b/c/file2"),
            new_pathbuf("/a/b/c/file1"),
        ];

        assert!(matches!(
            FilesystemHelper::calculate_common_prefix(&files_to_export),
            Err(FilesystemHelperError::NonAbsolutePaths)
        ));
    }

    #[test]
    fn test_real_path_to_bundle_path() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
        ];
        let helper = FilesystemHelper::new(files_to_export).unwrap();

        assert_eq!(
            helper
                .real_path_to_bundle_path(&new_pathbuf("/a/b/c/movie.swf"))
                .unwrap(),
            Path::new("c/movie.swf")
        );
        assert_eq!(
            helper
                .real_path_to_bundle_path(&new_pathbuf("/a/b/c/inner/movie.swf"))
                .unwrap(),
            Path::new("c/inner/movie.swf")
        );
        assert_eq!(
            helper
                .real_path_to_bundle_path(&new_pathbuf("/a/b/image.png"))
                .unwrap(),
            Path::new("image.png")
        );
        assert!(matches!(
            helper.real_path_to_bundle_path(&new_pathbuf("/a/bc/image.png")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
        assert!(matches!(
            helper.real_path_to_bundle_path(&new_pathbuf("/a/g/image.png")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
        assert!(matches!(
            helper.real_path_to_bundle_path(&new_pathbuf("/a/image.png")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
    }

    #[test]
    fn test_real_path_to_bundle_url() {
        let files_to_export = vec![
            //
            new_pathbuf("/a/b/c/file1"),
            new_pathbuf("/a/b/c/file2"),
            new_pathbuf("/a/b/f/file3"),
            new_pathbuf("/a/b/g/file4"),
            new_pathbuf("/a/b/g/h/file5"),
        ];
        let helper = FilesystemHelper::new(files_to_export).unwrap();

        assert_eq!(
            helper
                .real_path_to_bundle_url(&new_pathbuf("/a/b/c/movie.swf"))
                .unwrap(),
            Url::parse("file:///c/movie.swf").unwrap()
        );
        assert_eq!(
            helper
                .real_path_to_bundle_url(&new_pathbuf("/a/b/movie.swf"))
                .unwrap(),
            Url::parse("file:///movie.swf").unwrap()
        );
        assert_eq!(
            helper
                .real_path_to_bundle_url(&new_pathbuf("/a/b/g/inner/movie.swf"))
                .unwrap(),
            Url::parse("file:///g/inner/movie.swf").unwrap()
        );
        assert!(matches!(
            helper.real_path_to_bundle_url(&new_pathbuf("/a/bg/root_movie.swf")),
            Err(FilesystemHelperError::PathOutOfCommonDirectory(_))
        ));
    }

    async fn perform_export(
        output: &Path,
        bundle_name: String,
        player_options: PlayerOptions,
        movie_url: Url,
        exported_files: Vec<PathBuf>,
    ) -> Result<(), FilesystemHelperError> {
        let info = BundleInformation {
            name: bundle_name,
            url: movie_url,
            player: player_options,
        };

        FilesystemHelper::new(exported_files).and_then(|h| h.export_bundle(info, output))
    }

    #[tokio::test]
    async fn export_bundle_remote() {
        let tmp_dir = tempdir();
        let output_path = tmp_dir.path().join("bundle.ruf");

        let bundle_name = "remote".to_owned();
        let player_options = PlayerOptions::default();
        let movie_url = url("http://example.com");

        let result = perform_export(
            &output_path,
            bundle_name.clone(),
            player_options.clone(),
            movie_url.clone(),
            Vec::new(),
        )
        .await;
        assert!(result.is_ok());

        let bundle = open_bundle(&output_path);
        assert_eq!(bundle.information().name, bundle_name);
        assert_eq!(
            bundle.information().url,
            movie_url,
            "URL should be the same as input URL, as it refers to a remote \
            source."
        );
        assert_eq!(bundle.information().player, player_options);
        assert!(bundle.warnings().is_empty());

        assert_bundle_file_names_eq!(bundle, vec!["ruffle-bundle.toml"]);
    }

    #[tokio::test]
    async fn perform_export_no_files_to_export() {
        let tmp_dir = tempdir();
        let output_path = tmp_dir.path().join("bundle.ruf");
        let movie_path = tmp_dir.path().join("root_movie.swf");

        let result = perform_export(
            &output_path,
            "no_files_to_export".to_owned(),
            PlayerOptions::default(),
            url_from_path(&movie_path),
            Vec::new(),
        )
        .await;
        assert!(result.is_err());

        assert!(
            matches!(result.unwrap_err(), FilesystemHelperError::NoFilesToExport),
            "Exporting a bundle referencing a local file without exporting \
            the file itself should result in an error."
        );
    }

    #[tokio::test]
    async fn perform_export_local() {
        let tmp_dir = tempdir();
        let output_path = tmp_dir.path().join("bundle.ruf");

        let movie_path = new_tmp_file(&tmp_dir, "root_movie.swf", "root_movie content");

        let bundle_name = "local".to_owned();
        let player_options = PlayerOptions::default();

        let result: Result<(), FilesystemHelperError> = perform_export(
            &output_path,
            bundle_name.clone(),
            player_options.clone(),
            url_from_path(&movie_path),
            vec![movie_path.clone()],
        )
        .await;
        assert!(result.is_ok());

        let bundle = open_bundle(&output_path);
        assert_eq!(bundle.information().name, bundle_name);
        assert_eq!(
            bundle.information().url,
            url("file:///root_movie.swf"),
            "URL should be transformed to refer to a local file inside the \
            bundle."
        );
        assert_eq!(bundle.information().player, player_options);
        assert!(bundle.warnings().is_empty());

        assert_bundle_file_names_eq!(bundle, vec!["content/root_movie.swf", "ruffle-bundle.toml"]);
        assert_bundle_file_eq!(bundle, "root_movie.swf", "root_movie content");
        assert_bundle_file_eq!(bundle, "/root_movie.swf", "root_movie content");
    }

    #[tokio::test]
    async fn perform_export_multiple_files() {
        let tmp_dir = tempdir();
        let output_path = tmp_dir.path().join("bundle.ruf");

        let movie_path = new_tmp_file(&tmp_dir, "root_movie.swf", "root_movie content");
        let file_a = new_tmp_file(&tmp_dir, "a", "a content");
        let file_b = new_tmp_file(&tmp_dir, "b", "b content");
        let file_c = new_tmp_file(&tmp_dir, "c", "c content");

        let result = perform_export(
            &output_path,
            "multiple_files".to_owned(),
            PlayerOptions::default(),
            url_from_path(&movie_path),
            vec![
                movie_path.clone(),
                file_a.clone(),
                file_c.clone(),
                file_b.clone(),
            ],
        )
        .await;
        assert!(result.is_ok());

        let bundle = open_bundle(&output_path);
        assert_eq!(bundle.information().url, url("file:///root_movie.swf"));
        assert!(bundle.warnings().is_empty());

        assert_bundle_file_names_eq!(
            bundle,
            vec![
                "content/root_movie.swf",
                "content/a",
                "content/c",
                "content/b",
                "ruffle-bundle.toml"
            ]
        );
        assert_bundle_file_eq!(bundle, "root_movie.swf", "root_movie content");
        assert_bundle_file_eq!(bundle, "a", "a content");
        assert_bundle_file_eq!(bundle, "b", "b content");
        assert_bundle_file_eq!(bundle, "c", "c content");
    }

    #[tokio::test]
    async fn perform_export_root_swf_in_subdir() {
        let tmp_dir = tempdir();
        let output_path = tmp_dir.path().join("bundle.ruf");

        let movie_path = new_tmp_file(&tmp_dir, "dir/root_movie.swf", "root_movie content");
        let file_other = new_tmp_file(&tmp_dir, "other.swf", "other content");

        let result = perform_export(
            &output_path,
            "root_swf_in_subdir".to_owned(),
            PlayerOptions::default(),
            url_from_path(&movie_path),
            vec![movie_path.clone(), file_other.clone()],
        )
        .await;
        assert!(result.is_ok());

        let bundle = open_bundle(&output_path);
        assert_eq!(
            bundle.information().url,
            url("file:///dir/root_movie.swf"),
            "URL should take into account that the movie is not in the root \
            directory."
        );
        assert!(bundle.warnings().is_empty());

        assert_bundle_file_names_eq!(
            bundle,
            vec![
                "content/dir/root_movie.swf",
                "content/other.swf",
                "ruffle-bundle.toml"
            ]
        );
        assert_bundle_file_eq!(bundle, "dir/root_movie.swf", "root_movie content");
        assert_bundle_file_eq!(bundle, "other.swf", "other content");
    }

    #[tokio::test]
    async fn perform_export_non_existent_file() {
        let tmp_dir = tempdir();
        let output_path = tmp_dir.path().join("bundle.ruf");
        let movie_path = tmp_dir.path().join("root_movie.swf");

        let result = perform_export(
            &output_path,
            "non_existent_file".to_owned(),
            PlayerOptions::default(),
            url_from_path(&movie_path),
            vec![movie_path],
        )
        .await;
        assert!(result.is_err());

        assert!(
            matches!(result.unwrap_err(), FilesystemHelperError::IoError(_)),
            "Exporting a bundle referencing a non-existent local file should \
            result in an IO error."
        );
    }

    #[tokio::test]
    async fn perform_export_custom_player_options() {
        let tmp_dir = tempdir();
        let output_path = tmp_dir.path().join("bundle.ruf");

        let player_options = PlayerOptions {
            dummy_external_interface: Some(true),
            align: Some(ruffle_core::StageAlign::empty()),
            letterbox: Some(ruffle_core::config::Letterbox::Off),
            ..Default::default()
        };

        let result = perform_export(
            &output_path,
            "custom_player_options".to_owned(),
            player_options.clone(),
            url("http://example.com"),
            Vec::new(),
        )
        .await;
        assert!(result.is_ok());

        let bundle = open_bundle(&output_path);
        assert_eq!(bundle.information().player, player_options);
        assert!(bundle.warnings().is_empty());
    }
}
