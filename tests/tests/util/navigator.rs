use ruffle_core::backend::navigator::{
    NavigationMethod, NavigatorBackend, NullExecutor, NullSpawner, OwnedFuture, Request, Response,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::path::{Path, PathBuf};
use url::Url;

pub struct NavigatorTestBackend {
    spawner: NullSpawner,
    relative_base_path: PathBuf,
}

impl NavigatorTestBackend {
    pub fn with_base_path(path: &Path, executor: &NullExecutor) -> Result<Self, std::io::Error> {
        Ok(Self {
            spawner: executor.spawner(),
            relative_base_path: path.canonicalize()?,
        })
    }

    fn url_from_file_path(path: &Path) -> Result<Url, ()> {
        Url::from_file_path(path)
    }
}

impl NavigatorBackend for NavigatorTestBackend {
    fn navigate_to_url(
        &self,
        _url: String,
        _target: String,
        _vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error> {
        let mut path = self.relative_base_path.clone();
        path.push(request.url());

        Box::pin(async move {
            let url = Self::url_from_file_path(&path)
                .map_err(|()| Error::FetchError("Invalid URL".to_string()))?
                .into();

            let body = std::fs::read(path).map_err(|e| Error::FetchError(e.to_string()))?;

            Ok(Response { url, body })
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        //self.spawner.spawn_local(future);
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }
}
