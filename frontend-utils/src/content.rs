use crate::bundle::Bundle;
use std::fmt::{Debug, Formatter};
use url::Url;

/// Describes the content to load.
///
/// In case of local content, it contains not only the URL, but also the
/// root content path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentDescriptor {
    pub url: Url,

    /// Path representing the root of the content, optional even for local
    /// files. If not specified, Ruffle can assume the URL points to
    /// a standalone content that does not require neighboring files.
    #[cfg(feature = "fs")]
    pub root_content_path: Option<std::path::PathBuf>,
}

impl ContentDescriptor {
    pub fn new_remote(url: Url) -> Self {
        Self {
            url,
            #[cfg(feature = "fs")]
            root_content_path: None,
        }
    }

    #[cfg(feature = "fs")]
    pub fn new_local(
        file: &std::path::Path,
        root_content_path: Option<std::path::PathBuf>,
    ) -> Option<Self> {
        Some(Self {
            url: Url::from_file_path(file).ok()?,
            root_content_path,
        })
    }

    pub fn describe(&self) -> String {
        #[cfg(not(feature = "fs"))]
        {
            format!("{}", self.url)
        }

        #[cfg(feature = "fs")]
        if let Some(dir) = &self.root_content_path {
            format!("{} within {}", self.url, dir.display())
        } else {
            format!("{}", self.url)
        }
    }
}

/// Similar to [`ContentDescriptor`], but represents content that is already
/// opened and playing. Contains additional metadata.
pub enum PlayingContent {
    DirectFile(ContentDescriptor),
    Bundle(ContentDescriptor, Box<Bundle>),
}

impl Debug for PlayingContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayingContent::DirectFile(desc) => f
                .debug_tuple("PlayingContent::DirectFile")
                .field(desc)
                .finish(),
            PlayingContent::Bundle(desc, _) => f
                .debug_tuple("PlayingContent::Bundle")
                .field(desc)
                .field(&"_")
                .finish(),
        }
    }
}

impl PlayingContent {
    pub fn initial_swf_url(&self) -> &Url {
        match self {
            PlayingContent::DirectFile(desc) => &desc.url,
            PlayingContent::Bundle(_, bundle) => &bundle.information().url,
        }
    }

    pub fn name(&self) -> String {
        match self {
            PlayingContent::DirectFile(desc) => crate::url_to_readable_name(&desc.url).to_string(),
            PlayingContent::Bundle(_, bundle) => bundle.information().name.to_string(),
        }
    }

    #[cfg(feature = "navigator")]
    pub async fn get_local_file(
        &self,
        url: &Url,
        interface: impl crate::backends::navigator::NavigatorInterface,
    ) -> Result<Vec<u8>, std::io::Error> {
        use std::io::{ErrorKind, Read};

        match self {
            PlayingContent::DirectFile(_) => {
                let path = url
                    .to_file_path()
                    .map_err(|_| std::io::Error::other("Could not turn url into file path"))?;
                let mut result = vec![];
                let mut file = interface.open_file(&path).await?;
                file.read_to_end(&mut result)?;
                Ok(result)
            }
            PlayingContent::Bundle(_, bundle) => {
                if url.scheme() != "file" {
                    return Err(ErrorKind::NotFound.into());
                }
                let mut path = String::new();
                if let Some(segments) = url.path_segments() {
                    for segment in segments {
                        path.push('/');
                        path.push_str(
                            urlencoding::decode(segment)
                                .map_err(std::io::Error::other)?
                                .as_ref(),
                        );
                    }
                }
                bundle.source().read_content(&path)
            }
        }
    }
}
