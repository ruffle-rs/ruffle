//! Browser-related platform functions

use crate::loader::Error;
use crate::string::WStr;
use indexmap::IndexMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use swf::avm1::types::SendVarsMethod;
use url::{ParseError, Url};

/// Attempt to convert a relative URL into an absolute URL, using the base URL
/// if necessary.
///
/// If the relative URL is actually absolute, then the base will not be used.
pub fn url_from_relative_url(base: &str, relative: &str) -> Result<Url, ParseError> {
    let parsed = Url::parse(relative);
    if let Err(ParseError::RelativeUrlWithoutBase) = parsed {
        let base = Url::parse(base)?;
        return base.join(relative);
    }

    parsed
}

/// Enumerates all possible navigation methods.
#[derive(Copy, Clone)]
pub enum NavigationMethod {
    /// Indicates that navigation should generate a GET request.
    Get,

    /// Indicates that navigation should generate a POST request.
    Post,
}

impl NavigationMethod {
    /// Convert an SWF method enum into a NavigationMethod.
    pub fn from_send_vars_method(s: SendVarsMethod) -> Option<Self> {
        match s {
            SendVarsMethod::None => None,
            SendVarsMethod::Get => Some(Self::Get),
            SendVarsMethod::Post => Some(Self::Post),
        }
    }

    pub fn from_method_str(method: &WStr) -> Option<Self> {
        if method == b"GET" {
            Some(Self::Get)
        } else if method == b"POST" {
            Some(Self::Post)
        } else {
            None
        }
    }
}

/// A fetch request.
pub struct Request {
    /// The URL of the request.
    url: String,

    /// The HTTP method to be used to make the request.
    method: NavigationMethod,

    /// The contents of the request body, if the request's HTTP method supports
    /// having a body.
    ///
    /// The body consists of data and a mime type.
    body: Option<(Vec<u8>, String)>,
}

impl Request {
    /// Construct a GET request.
    pub fn get(url: String) -> Self {
        Self {
            url,
            method: NavigationMethod::Get,
            body: None,
        }
    }

    /// Construct a POST request.
    pub fn post(url: String, body: Option<(Vec<u8>, String)>) -> Self {
        Self {
            url,
            method: NavigationMethod::Post,
            body,
        }
    }

    /// Retrieve the URL of this request.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Retrieve the navigation method for this request.
    pub fn method(&self) -> NavigationMethod {
        self.method
    }

    /// Retrieve the body of this request, if it exists.
    pub fn body(&self) -> &Option<(Vec<u8>, String)> {
        &self.body
    }
}

/// A response to a fetch request.
pub struct Response {
    /// The final URL obtained after any redirects.
    pub url: String,

    /// The contents of the response body.
    pub body: Vec<u8>,
}

/// Type alias for pinned, boxed, and owned futures that output a falliable
/// result of type `Result<T, E>`.
pub type OwnedFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'static>>;

/// A backend interacting with a browser environment.
pub trait NavigatorBackend {
    /// Cause a browser navigation to a given URL.
    ///
    /// The URL given may be any URL scheme a browser can support. This may not
    /// be meaningful for all environments: for example, `javascript:` URLs may
    /// not be executable in a desktop context.
    ///
    /// The `window` parameter, if provided, should be treated identically to
    /// the `window` parameter on an HTML `<a>nchor` tag.
    ///
    /// This function may be used to send variables to an eligible target. If
    /// desired, the `vars_method` will be specified with a suitable
    /// `NavigationMethod` and a key-value representation of the variables to
    /// be sent. What the backend needs to do depends on the `NavigationMethod`:
    ///
    /// * `GET` - Variables are appended onto the query parameters of the given
    ///   URL.
    /// * `POST` - Variables are sent as form data in a POST request, as if the
    ///   user had filled out and submitted an HTML form.
    ///
    /// Flash Player implemented sandboxing to prevent certain kinds of XSS
    /// attacks. The `NavigatorBackend` is not responsible for enforcing this
    /// sandbox.
    fn navigate_to_url(
        &self,
        url: String,
        window: Option<String>,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    );

    /// Fetch data and return it some time in the future.
    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error>;

    /// Arrange for a future to be run at some point in the... well, future.
    ///
    /// This function must be called to ensure a future is actually computed.
    /// The future must output an empty value and not hold any stack references
    /// which would cause it to become invalidated.
    ///
    /// TODO: For some reason, `wasm_bindgen_futures` wants unpinnable futures.
    /// This seems highly limiting.
    fn spawn_future(&mut self, future: OwnedFuture<(), Error>);

    /// Handle any context specific pre-processing
    ///
    /// Changing http -> https for example. This function may alter any part of the
    /// URL (generally only if configured to do so by the user).
    fn pre_process_url(&self, url: Url) -> Url;
}

#[cfg(not(target_family = "wasm"))]
pub struct NullExecutor(futures::executor::LocalPool);

#[cfg(not(target_family = "wasm"))]
impl NullExecutor {
    pub fn new() -> Self {
        Self(futures::executor::LocalPool::new())
    }

    pub fn spawner(&self) -> NullSpawner {
        NullSpawner(self.0.spawner())
    }

    pub fn run(&mut self) {
        self.0.run();
    }
}

impl Default for NullExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_family = "wasm"))]
pub struct NullSpawner(futures::executor::LocalSpawner);

#[cfg(not(target_family = "wasm"))]
impl NullSpawner {
    pub fn spawn_local(&self, future: OwnedFuture<(), Error>) {
        use futures::task::LocalSpawnExt;
        let _ = self.0.spawn_local(async move {
            if let Err(e) = future.await {
                log::error!("Asynchronous error occurred: {}", e);
            }
        });
    }
}

#[cfg(target_family = "wasm")]
pub struct NullExecutor;

#[cfg(target_family = "wasm")]
impl NullExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn spawner(&self) -> NullSpawner {
        NullSpawner
    }

    pub fn run(&mut self) {}
}

#[cfg(target_family = "wasm")]
pub struct NullSpawner;

#[cfg(target_family = "wasm")]
impl NullSpawner {
    pub fn spawn_local(&self, future: OwnedFuture<(), Error>) {
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = future.await {
                log::error!("Asynchronous error occurred: {}", e);
            }
        });
    }
}

/// A null implementation for platforms that do not live in a web browser.
///
/// The NullNavigatorBackend includes a trivial executor that holds owned
/// futures and runs them to completion, blockingly.
pub struct NullNavigatorBackend {
    spawner: NullSpawner,

    /// The base path for all relative fetches.
    relative_base_path: PathBuf,
}

impl NullNavigatorBackend {
    pub fn new() -> Self {
        let executor = NullExecutor::new();
        Self {
            spawner: executor.spawner(),
            relative_base_path: PathBuf::new(),
        }
    }

    pub fn with_base_path(path: &Path, executor: &NullExecutor) -> Self {
        Self {
            spawner: executor.spawner(),
            relative_base_path: path.canonicalize().unwrap(),
        }
    }

    #[cfg(any(unix, windows, target_os = "redox"))]
    fn url_from_file_path(path: &Path) -> Result<Url, ()> {
        Url::from_file_path(path)
    }

    #[cfg(not(any(unix, windows, target_os = "redox")))]
    fn url_from_file_path(_path: &Path) -> Result<Url, ()> {
        Err(())
    }
}

impl Default for NullNavigatorBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigatorBackend for NullNavigatorBackend {
    fn navigate_to_url(
        &self,
        _url: String,
        _window: Option<String>,
        _vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error> {
        let mut path = self.relative_base_path.clone();
        path.push(request.url);

        Box::pin(async move {
            let url = Self::url_from_file_path(&path)
                .map_err(|()| Error::FetchError("Invalid URL".to_string()))?
                .into();

            let body = std::fs::read(path).map_err(|e| Error::FetchError(e.to_string()))?;

            Ok(Response { url, body })
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.spawner.spawn_local(future);
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }
}
