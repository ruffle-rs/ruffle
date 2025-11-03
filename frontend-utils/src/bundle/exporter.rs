pub mod helpers;

use super::info::{BundleInformation, BUNDLE_INFORMATION_FILENAME};
use std::{
    io::{Read, Seek, Write},
    path::Path,
};
use zip::{result::ZipError, write::FileOptions, ZipWriter};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum BundleExportError {
    #[error("Error writing ZIP stream: {0}")]
    ZipError(#[from] ZipError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Missing bundle information")]
    MissingBundleInfo,
}

pub type BundleExportResult<T> = Result<T, BundleExportError>;

pub struct BundleExporter<W: Write + Seek> {
    writer: ZipWriter<W>,
    info: BundleInformation,
}

impl<W: Write + Seek> BundleExporter<W> {
    pub fn new(write: W, info: BundleInformation) -> Self {
        Self {
            writer: ZipWriter::new(write),
            info,
        }
    }

    pub fn write_content<R: Read, P: AsRef<Path>>(
        &mut self,
        path: P,
        content: &mut R,
    ) -> BundleExportResult<()> {
        let options: FileOptions<'_, ()> = FileOptions::default();

        self.writer
            .start_file_from_path(Path::new("content").join(path), options)?;

        std::io::copy(content, &mut self.writer)?;

        Ok(())
    }

    pub fn finish(mut self) -> BundleExportResult<()> {
        self.write_info()?;
        self.writer.finish()?;
        Ok(())
    }

    fn write_info(&mut self) -> BundleExportResult<()> {
        let options: FileOptions<'_, ()> = FileOptions::default();

        self.writer
            .start_file_from_path(Path::new(BUNDLE_INFORMATION_FILENAME), options)?;
        self.info.serialize(&mut self.writer)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::io::Seek;
    use std::io::SeekFrom;
    use url::Url;

    use crate::bundle::exporter::BundleExporter;
    use crate::bundle::info::BundleInformation;
    use crate::bundle::source::BundleSource;
    use crate::bundle::Bundle;
    use crate::player_options::PlayerOptions;

    #[test]
    fn basic_export() {
        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let info = BundleInformation {
            name: "test".to_owned(),
            url: Url::parse("http://example.com").unwrap(),
            player: PlayerOptions {
                frame_rate: Some(5.0),
                ..Default::default()
            },
        };
        let exporter = BundleExporter::new(&mut buffer, info);
        exporter.finish().unwrap();

        buffer.seek(SeekFrom::Start(0)).unwrap();

        let source = BundleSource::from_reader(buffer).unwrap();
        let bundle = Bundle::from_source(source).unwrap();

        assert_eq!(bundle.information().name, "test");
        assert_eq!(
            bundle.information().url,
            Url::parse("http://example.com").unwrap()
        );
        assert_eq!(bundle.information().player.frame_rate, Some(5.0));
    }

    #[test]
    fn export_with_content() {
        let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let info = BundleInformation {
            name: "test2".to_owned(),
            url: Url::parse("https://example.com").unwrap(),
            player: PlayerOptions {
                ..Default::default()
            },
        };

        let mut exporter = BundleExporter::new(&mut buffer, info);
        exporter
            .write_content("test.txt", &mut Cursor::new("some content"))
            .unwrap();
        exporter.finish().unwrap();

        buffer.seek(SeekFrom::Start(0)).unwrap();

        let source = BundleSource::from_reader(buffer).unwrap();
        let bundle = Bundle::from_source(source).unwrap();

        assert_eq!(bundle.information().name, "test2");
        assert_eq!(
            bundle.information().url,
            Url::parse("https://example.com").unwrap()
        );
        assert_eq!(
            String::from_utf8(bundle.source().read_content("test.txt").unwrap()).unwrap(),
            "some content"
        );
    }
}
