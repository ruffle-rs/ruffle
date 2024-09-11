use crate::backends::navigator::NavigatorInterface;
use crate::bundle::Bundle;
use std::fmt::{Debug, Formatter};
use std::io::{ErrorKind, Read};
use url::Url;

pub enum PlayingContent {
    DirectFile(Url),
    Bundle(Url, Bundle),
}

impl Debug for PlayingContent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayingContent::DirectFile(url) => f
                .debug_tuple("PlayingContent::DirectFile")
                .field(url)
                .finish(),
            PlayingContent::Bundle(url, _) => f
                .debug_tuple("PlayingContent::Bundle")
                .field(url)
                .field(&"_")
                .finish(),
        }
    }
}

impl PlayingContent {
    pub fn initial_swf_url(&self) -> &Url {
        match self {
            PlayingContent::DirectFile(url) => url,
            PlayingContent::Bundle(_, bundle) => &bundle.information().url,
        }
    }

    pub fn name(&self) -> String {
        match self {
            PlayingContent::DirectFile(url) => crate::url_to_readable_name(url).to_string(),
            PlayingContent::Bundle(_, bundle) => bundle.information().name.to_string(),
        }
    }

    pub async fn get_local_file(
        &self,
        url: &Url,
        interface: impl NavigatorInterface,
    ) -> Result<Vec<u8>, std::io::Error> {
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
