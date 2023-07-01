use crate::util::runner::TestLogBackend;
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{
    NavigationMethod, NavigatorBackend, NullExecutor, NullSpawner, OwnedFuture, Request, Response,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::path::{Path, PathBuf};
use url::Url;

/// A `NavigatorBackend` used by tests that supports logging fetch requests.
///
/// This can be used by tests that fetch data to verify that the request is correct.
pub struct TestNavigatorBackend {
    spawner: NullSpawner,
    relative_base_path: PathBuf,
    log: Option<TestLogBackend>,
}

impl TestNavigatorBackend {
    pub fn new(
        path: &Path,
        executor: &NullExecutor,
        log: Option<TestLogBackend>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            spawner: executor.spawner(),
            relative_base_path: path.canonicalize()?,
            log,
        })
    }

    fn url_from_file_path(path: &Path) -> Result<Url, ()> {
        Url::from_file_path(path)
    }
}

impl NavigatorBackend for TestNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: &str,
        target: &str,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        // Log request.
        if let Some(log) = &self.log {
            log.avm_trace("Navigator::navigate_to_url:");
            log.avm_trace(&format!("  URL: {}", url));
            log.avm_trace(&format!("  Target: {}", target));
            if let Some((method, vars)) = vars_method {
                log.avm_trace(&format!("  Method: {}", method));
                for (key, value) in vars {
                    log.avm_trace(&format!("  Param: {}={}", key, value));
                }
            }
        }
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error> {
        // Log request.
        if let Some(log) = &self.log {
            log.avm_trace("Navigator::fetch:");
            log.avm_trace(&format!("  URL: {}", request.url()));
            log.avm_trace(&format!("  Method: {}", request.method()));
            let headers = request.headers();
            if !headers.is_empty() {
                log.avm_trace(&format!(
                    "  Headers:\n{}",
                    headers
                        .iter()
                        .map(|(key, val)| format!("{key}: {val}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
            if let Some((body, mime_type)) = request.body() {
                log.avm_trace(&format!("  Mime-Type: {}", mime_type));
                if mime_type == "application/x-www-form-urlencoded" {
                    log.avm_trace(&format!("  Body: {}", String::from_utf8_lossy(body)));
                } else {
                    log.avm_trace(&format!("  Body: ({} bytes)", body.len()));
                }
            }
        }

        let path = self.relative_base_path.clone();

        Box::pin(async move {
            let mut base_url = Self::url_from_file_path(path.as_path())
                .map_err(|_| Error::FetchError("Invalid base URL".to_string()))?;

            // Make sure we have a trailing slash, so that joining a request url like 'data.txt'
            // gets appended, rather than replacing the last component.
            base_url.path_segments_mut().unwrap().push("");
            let response_url = base_url
                .join(request.url())
                .map_err(|_| Error::FetchError("Invalid URL".to_string()))?;

            // Flash supports query parameters with local urls.
            // SwfMovie takes care of exposing those to ActionScript -
            // when we actually load a filesystem url, strip them out.
            let mut filesystem_url = response_url.clone();
            filesystem_url.set_query(None);

            let filesystem_path = filesystem_url
                .to_file_path()
                .map_err(|_| Error::FetchError("Invalid filesystem URL".to_string()))?;

            let body =
                std::fs::read(filesystem_path).map_err(|e| Error::FetchError(e.to_string()))?;

            Ok(Response {
                url: response_url.to_string(),
                body,
                status: 0,
                redirected: false,
            })
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.spawner.spawn_local(future);
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }
}
