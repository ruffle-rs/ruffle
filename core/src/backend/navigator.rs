//! Browser-related platform functions

use crate::loader::Error;
use crate::socket::{ConnectionState, SocketAction, SocketHandle};
use crate::string::WStr;
use async_channel::{Receiver, Sender};
use downcast_rs::Downcast;
use encoding_rs::Encoding;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::future::Future;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;
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

#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SocketMode {
    /// Allows movies to connect to any host using sockets.
    Allow,

    /// Refuse all socket connection requests
    Deny,

    /// Ask the user every time a socket connection is requested
    Ask,
}

/// The handling mode of links opening a new website.
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpenURLMode {
    /// Allow all links to open a new website.
    #[cfg_attr(feature = "serde", serde(rename = "allow"))]
    Allow,

    /// A confirmation dialog opens with every link trying to open a new website.
    #[cfg_attr(feature = "serde", serde(rename = "confirm"))]
    Confirm,

    /// Deny all links to open a new website.
    #[cfg_attr(feature = "serde", serde(rename = "deny"))]
    Deny,
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
        // Methods seem to be case insensitive
        let method = method.to_ascii_lowercase();
        if &method == b"get" {
            Some(Self::Get)
        } else if &method == b"post" {
            Some(Self::Post)
        } else {
            None
        }
    }
}

impl fmt::Display for NavigationMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let method = match self {
            Self::Get => "GET",
            Self::Post => "POST",
        };
        f.write_str(method)
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

    /// The headers for the request, as (header_name, header_value) pairs.
    /// Flash appears to iterate over an internal hash table to determine
    /// the order of headers sent over the network. We just use an IndexMap
    /// to give us a consistent order - hopefully, no servers depend on
    /// the order of headers.
    headers: IndexMap<String, String>,
}

impl Request {
    /// Construct a GET request.
    pub fn get(url: String) -> Self {
        Self {
            url,
            method: NavigationMethod::Get,
            body: None,
            headers: Default::default(),
        }
    }

    /// Construct a POST request.
    pub fn post(url: String, body: Option<(Vec<u8>, String)>) -> Self {
        Self {
            url,
            method: NavigationMethod::Post,
            body,
            headers: Default::default(),
        }
    }

    /// Construct a request with the given method and data
    #[allow(clippy::self_named_constructors)]
    pub fn request(method: NavigationMethod, url: String, body: Option<(Vec<u8>, String)>) -> Self {
        Self {
            url,
            method,
            body,
            headers: Default::default(),
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

    pub fn set_body(&mut self, body: (Vec<u8>, String)) {
        self.body = Some(body);
    }

    pub fn headers(&self) -> &IndexMap<String, String> {
        &self.headers
    }

    pub fn set_headers(&mut self, headers: IndexMap<String, String>) {
        self.headers = headers;
    }
}

/// A response to a successful fetch request.
pub trait SuccessResponse {
    /// The final URL obtained after any redirects.
    fn url(&self) -> Cow<str>;

    /// Retrieve the contents of the response body.
    ///
    /// This method consumes the response.
    fn body(self: Box<Self>) -> OwnedFuture<Vec<u8>, Error>;

    /// The text encoding listed in the HTTP response header if existing.
    fn text_encoding(&self) -> Option<&'static Encoding>;

    /// The status code of the response.
    fn status(&self) -> u16;

    /// Indicates if the request has been redirected.
    fn redirected(&self) -> bool;

    /// Read the next chunk of the response.
    ///
    /// Repeated calls to `next_chunk` yield further bytes of the response body.
    /// A response that has no data or no more data to yield will instead
    /// yield None.
    ///
    /// The size of yielded chunks is implementation-defined.
    ///
    /// Mixing `next_chunk` and `body` is not supported and may yield errors.
    /// Use one or the other.
    fn next_chunk(&mut self) -> OwnedFuture<Option<Vec<u8>>, Error>;

    /// Estimate the expected length of the response body.
    ///
    /// Returned length may not correspond to the actual length of data
    /// returned from `next_chunk` or `body`. A `None` indicates that the data
    /// is of indefinite length as reported by the source of the response.
    ///
    /// An error may be returned if the source reported corrupted or invalid
    /// length information.
    fn expected_length(&self) -> Result<Option<u64>, Error>;
}

/// A response to a non-successful fetch request.
pub struct ErrorResponse {
    /// The final URL obtained after any redirects.
    pub url: String,

    /// The error that occurred during the request.
    pub error: Error,
}

/// Type alias for pinned, boxed, and owned futures that output a falliable
/// result of type `Result<T, E>`.
pub type OwnedFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + 'static>>;

/// A backend interacting with a browser environment.
pub trait NavigatorBackend: Downcast {
    /// Cause a browser navigation to a given URL.
    ///
    /// The URL given may be any URL scheme a browser can support. This may not
    /// be meaningful for all environments: for example, `javascript:` URLs may
    /// not be executable in a desktop context.
    ///
    /// The `target` parameter, should be treated identically to the `target`
    /// parameter on an HTML `<a>nchor` tag.
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
        url: &str,
        target: &str,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    );

    /// Fetch data and return it some time in the future.
    fn fetch(&self, request: Request) -> OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse>;

    /// Take a URL string and resolve it to the actual URL from which a file
    /// can be fetched. This includes handling of relative links and pre-processing.
    ///
    /// If the URL is local, this equals the URL returned by fetch. Otherwise,
    /// fetch may return a different URL, e.g. considering redirections.
    fn resolve_url(&self, url: &str) -> Result<Url, ParseError>;

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

    /// Handle any Socket connection request
    ///
    /// Use [SocketAction::Connect] to notify AVM that the connection failed or succeeded.
    ///
    /// Use [SocketAction::Close] to close the connection on AVM side.
    ///
    /// Use [SocketAction::Data] to send data to AVM side.
    ///
    /// When the Sender of the Receiver is dropped then this task should end.
    fn connect_socket(
        &mut self,
        host: String,
        port: u16,
        timeout: Duration,
        handle: SocketHandle,
        receiver: Receiver<Vec<u8>>,
        sender: Sender<SocketAction>,
    );
}
impl_downcast!(NavigatorBackend);

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
        self.0.run_until_stalled();
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
                tracing::error!("Asynchronous error occurred: {}", e);
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
                tracing::error!("Asynchronous error occurred: {}", e);
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

    pub fn with_base_path(path: &Path, executor: &NullExecutor) -> Result<Self, std::io::Error> {
        Ok(Self {
            spawner: executor.spawner(),
            relative_base_path: path.canonicalize()?,
        })
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
        _url: &str,
        _target: &str,
        _vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse> {
        fetch_path(self, "NullNavigatorBackend", request.url(), None)
    }

    fn resolve_url(&self, url: &str) -> Result<Url, ParseError> {
        resolve_url_with_relative_base_path(self, self.relative_base_path.clone(), url)
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.spawner.spawn_local(future);
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }

    fn connect_socket(
        &mut self,
        _host: String,
        _port: u16,
        _timeout: Duration,
        handle: SocketHandle,
        _receiver: Receiver<Vec<u8>>,
        sender: Sender<SocketAction>,
    ) {
        sender
            .try_send(SocketAction::Connect(handle, ConnectionState::Failed))
            .expect("working channel send");
    }
}

// The following functions are helper functions used in different
// NavigatorBackend implementations.
// To avoid duplicated code, they are placed here as public functions.

/// Converts a given result into an OwnedFuture and returns it.
pub fn async_return<SuccessType: 'static, ErrorType: 'static>(
    return_value: Result<SuccessType, ErrorType>,
) -> OwnedFuture<SuccessType, ErrorType> {
    Box::pin(async move { return_value })
}

/// This creates and returns the generic ErrorResponse for an invalid URL
/// used in the NavigatorBackend fetch methods.
pub fn create_fetch_error<ErrorType: Display>(
    url: &str,
    error: ErrorType,
) -> Result<Box<dyn SuccessResponse>, ErrorResponse> {
    create_specific_fetch_error("Invalid URL", url, error)
}

/// This creates and returns a specific ErrorResponse with a given reason
/// used in the NavigatorBackend fetch methods.
pub fn create_specific_fetch_error<ErrorType: Display>(
    reason: &str,
    url: &str,
    error: ErrorType,
) -> Result<Box<dyn SuccessResponse>, ErrorResponse> {
    let message = if error.to_string() == "" {
        format!("{reason} {url}")
    } else {
        format!("{reason} {url}: {error}")
    };
    let error = Error::FetchError(message);
    Err(ErrorResponse {
        url: url.to_string(),
        error,
    })
}

// Url doesn't implement from_file_path and to_file_path for WASM targets.
// Therefore, we need to use cfg to make Ruffle compile for all targets.

#[cfg(any(unix, windows, target_os = "redox"))]
fn url_from_file_path(path: &Path) -> Result<Url, ()> {
    Url::from_file_path(path)
}

#[cfg(not(any(unix, windows, target_os = "redox")))]
fn url_from_file_path(_path: &Path) -> Result<Url, ()> {
    Err(())
}

#[cfg(any(unix, windows, target_os = "redox"))]
fn url_to_file_path(url: &Url) -> Result<PathBuf, ()> {
    Url::to_file_path(url)
}

#[cfg(not(any(unix, windows, target_os = "redox")))]
fn url_to_file_path(_path: &Url) -> Result<PathBuf, ()> {
    Err(())
}

// The following functions are implementations used in multiple places.
// To avoid duplicated code, they are placed here as public functions.

/// This is the resolve implementation for the TestNavigatorBackend and the
/// NullNavigatorBackend.
///
/// It resolves the given URL with the given relative base path.
pub fn resolve_url_with_relative_base_path<NavigatorType: NavigatorBackend>(
    navigator: &NavigatorType,
    base_path: PathBuf,
    url: &str,
) -> Result<Url, ParseError> {
    /// This is a helper function used to resolve just the request url.
    /// It is used if the base url and the request url can't be combined.
    fn resolve_request_url<NavigatorType: NavigatorBackend>(
        url: &str,
        navigator: &NavigatorType,
    ) -> Result<Url, ParseError> {
        match Url::parse(url) {
            Ok(parsed_url) => Ok(navigator.pre_process_url(parsed_url)),
            Err(error) => Err(error),
        }
    }

    if let Ok(mut base_url) = url_from_file_path(base_path.as_path()) {
        // Make sure we have a trailing slash, so that joining a request url like 'data.txt'
        // gets appended, rather than replacing the last component.
        base_url.path_segments_mut().unwrap().push("");
        if let Ok(parsed_url) = base_url.join(url) {
            Ok(navigator.pre_process_url(parsed_url))
        } else {
            resolve_request_url(url, navigator)
        }
    } else {
        resolve_request_url(url, navigator)
    }
}

/// This is the fetch implementation for NullNavigatorBackend.
///
/// It tries to fetch the given URL as a local path and read and return
/// its content. It returns an ErrorResponse if the URL is not valid, not
/// local or a local path that can't be read.
pub fn fetch_path<NavigatorType: NavigatorBackend>(
    navigator: &NavigatorType,
    navigator_name: &str,
    url: &str,
    base_path: Option<&Path>,
) -> OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse> {
    struct LocalResponse {
        url: String,
        path: PathBuf,
        open_file: Option<File>,
        status: u16,
        redirected: bool,
    }

    impl SuccessResponse for LocalResponse {
        fn url(&self) -> Cow<str> {
            Cow::Borrowed(&self.url)
        }

        fn body(self: Box<Self>) -> OwnedFuture<Vec<u8>, Error> {
            Box::pin(async move {
                std::fs::read(self.path).map_err(|e| Error::FetchError(e.to_string()))
            })
        }

        fn text_encoding(&self) -> Option<&'static Encoding> {
            None
        }

        fn status(&self) -> u16 {
            self.status
        }

        fn redirected(&self) -> bool {
            self.redirected
        }

        fn next_chunk(&mut self) -> OwnedFuture<Option<Vec<u8>>, Error> {
            if self.open_file.is_none() {
                let result = std::fs::File::open(self.path.clone())
                    .map_err(|e| Error::FetchError(e.to_string()));

                match result {
                    Ok(file) => self.open_file = Some(file),
                    Err(e) => return Box::pin(async move { Err(e) }),
                }
            }

            let file = self.open_file.as_mut().unwrap();
            let mut buf = vec![0; 4096];
            let res = file.read(&mut buf);

            Box::pin(async move {
                match res {
                    Ok(count) if count > 0 => {
                        buf.resize(count, 0);
                        Ok(Some(buf))
                    }
                    Ok(_) => Ok(None),
                    Err(e) => Err(Error::FetchError(e.to_string())),
                }
            })
        }

        fn expected_length(&self) -> Result<Option<u64>, Error> {
            Ok(Some(
                std::fs::File::open(self.path.clone())?.metadata()?.len(),
            ))
        }
    }

    let url = match navigator.resolve_url(url) {
        Ok(url) => url,
        Err(e) => return async_return(create_fetch_error(url, e)),
    };
    let path = if url.scheme() == "file" {
        // Flash supports query parameters with local urls.
        // SwfMovie takes care of exposing those to ActionScript -
        // when we actually load a filesystem url, strip them out.
        let mut filesystem_url = url.clone();
        filesystem_url.set_query(None);

        match url_to_file_path(&filesystem_url) {
            Ok(path) => path,
            Err(_) => {
                return async_return(create_specific_fetch_error(
                    "Unable to create path out of URL",
                    url.as_str(),
                    "",
                ))
            }
        }
    } else if let Some(base_path) = base_path {
        // Turn a url like https://localhost/foo/bar to {base_path}/localhost/foo/bar
        let mut path = base_path.to_path_buf();
        if let Some(host) = url.host_str() {
            path.push(host);
        }
        if let Some(remaining) = url.path().strip_prefix('/') {
            path.push(remaining);
        }
        path
    } else {
        return async_return(create_specific_fetch_error(
            &format!("{navigator_name} can't fetch non-local URL"),
            url.as_str(),
            "",
        ));
    };

    Box::pin(async move {
        let response: Box<dyn SuccessResponse> = Box::new(LocalResponse {
            url: url.to_string(),
            path,
            open_file: None,
            status: 0,
            redirected: false,
        });

        Ok(response)
    })
}

/// Parses and returns the encoding out of an HTTP header content type string
/// if existing.
pub fn get_encoding(content_type: &str) -> Option<&'static Encoding> {
    if let Some((_, encoding_string)) = content_type.split_once("charset=") {
        Encoding::for_label(encoding_string.as_bytes())
    } else {
        None
    }
}
