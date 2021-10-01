//! Browser-related platform functions

use crate::loader::Error;
use crate::string::WStr;
use indexmap::IndexMap;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::fs;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::ptr::null;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::time::Duration;
use swf::avm1::types::SendVarsMethod;
use url::{ParseError, Url};

/// Attempt to convert a relative filesystem path into an absolute `file:///`
/// URL.
///
/// If the relative path is an absolute path, the base will not be used, but it
/// will still be parsed into a `Url`.
///
/// This is the desktop version of this function, which actually carries out
/// the above instructions. On non-Unix, non-Windows, non-Redox environments,
/// this function always yields an error.
#[cfg(any(unix, windows, target_os = "redox"))]
pub fn url_from_relative_path<P: AsRef<Path>>(base: P, relative: &str) -> Result<Url, ParseError> {
    let parsed = Url::from_file_path(relative);
    if let Err(()) = parsed {
        let base =
            Url::from_directory_path(base).map_err(|_| ParseError::RelativeUrlWithoutBase)?;

        return base.join(relative);
    }

    Ok(parsed.unwrap())
}

/// Attempt to convert a relative filesystem path into an absolute `file:///`
/// URL.
///
/// If the relative path is an absolute path, the base will not be used, but it
/// will still be parsed into a `Url`.
///
/// This is the web version of this function, which always yields an error. On
/// Unix, Windows, or Redox, this function actually carries out the above
/// instructions.
#[cfg(not(any(unix, windows, target_os = "redox")))]
pub fn url_from_relative_path<P: AsRef<Path>>(
    _base: P,
    _relative: &str,
) -> Result<Url, ParseError> {
    Err(ParseError::RelativeUrlWithoutBase)
}

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

    pub fn from_method_str(method: WStr<'_>) -> Option<Self> {
        if method == b"GET" {
            Some(Self::Get)
        } else if method == b"POST" {
            Some(Self::Post)
        } else {
            None
        }
    }
}

/// Represents request options to be sent as part of a fetch.
pub struct RequestOptions {
    /// The HTTP method to be used to make the request.
    method: NavigationMethod,

    /// The contents of the request body, if the request's HTTP method supports
    /// having a body.
    ///
    /// The body consists of data and a mime type.
    body: Option<(Vec<u8>, String)>,
}

impl RequestOptions {
    /// Construct request options for a GET request.
    pub fn get() -> Self {
        Self {
            method: NavigationMethod::Get,
            body: None,
        }
    }

    /// Construct request options for a POST request.
    pub fn post(body: Option<(Vec<u8>, String)>) -> Self {
        Self {
            method: NavigationMethod::Post,
            body,
        }
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

    /// Fetch data at a given URL and return it some time in the future.
    fn fetch(&self, url: &str, request_options: RequestOptions) -> OwnedFuture<Vec<u8>, Error>;

    /// Get the amount of time since the SWF was launched.
    /// Used by the `getTimer` ActionScript call.
    fn time_since_launch(&mut self) -> Duration;

    /// Arrange for a future to be run at some point in the... well, future.
    ///
    /// This function must be called to ensure a future is actually computed.
    /// The future must output an empty value and not hold any stack references
    /// which would cause it to become invalidated.
    ///
    /// TODO: For some reason, `wasm_bindgen_futures` wants unpinnable futures.
    /// This seems highly limiting.
    fn spawn_future(&mut self, future: OwnedFuture<(), Error>);

    /// Resolve a relative URL.
    ///
    /// This function must not change URLs which are already protocol, domain,
    /// and path absolute. For URLs that are relative, the implementer of
    /// this function may opt to convert them to absolute using an implementor
    /// defined base. For a web browser, the most obvious base would be the
    /// current document's base URL, while the most obvious base for a desktop
    /// client would be the file-URL form of the current path.
    fn resolve_relative_url<'a>(&self, url: &'a str) -> Cow<'a, str>;

    /// Handle any context specific pre-processing
    ///
    /// Changing http -> https for example. This function may alter any part of the
    /// URL (generally only if configured to do so by the user).
    fn pre_process_url(&self, url: Url) -> Url;
}

/// A null implementation of an event loop that only supports blocking.
pub struct NullExecutor {
    /// The list of outstanding futures spawned on this executor.
    futures_queue: VecDeque<OwnedFuture<(), Error>>,

    /// The source of any additional futures.
    channel: Receiver<OwnedFuture<(), Error>>,
}

unsafe fn do_nothing(_data: *const ()) {}

unsafe fn clone(_data: *const ()) -> RawWaker {
    NullExecutor::raw_waker()
}

const NULL_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, do_nothing, do_nothing, do_nothing);

impl NullExecutor {
    /// Construct a new executor.
    ///
    /// The sender yielded as part of construction should be given to a
    /// `NullNavigatorBackend` so that it can spawn futures on this executor.
    pub fn new() -> (Self, Sender<OwnedFuture<(), Error>>) {
        let (send, recv) = channel();

        (
            Self {
                futures_queue: VecDeque::new(),
                channel: recv,
            },
            send,
        )
    }

    /// Construct a do-nothing raw waker.
    ///
    /// The RawWaker, because the RawWaker
    /// interface normally deals with unchecked pointers. We instead just hand
    /// it a null pointer and do nothing with it, which is trivially sound.
    fn raw_waker() -> RawWaker {
        RawWaker::new(null(), &NULL_VTABLE)
    }

    /// Copy all outstanding futures into the local queue.
    fn flush_channel(&mut self) {
        for future in self.channel.try_iter() {
            self.futures_queue.push_back(future);
        }
    }

    /// Poll all in-progress futures.
    ///
    /// If any task in the executor yields an error, then this function will
    /// stop polling futures and return that error. Otherwise, it will yield
    /// `Ok`, indicating that no errors occurred. More work may still be
    /// available,
    pub fn poll_all(&mut self) -> Result<(), Error> {
        self.flush_channel();

        let mut unfinished_futures = VecDeque::new();
        let mut result = Ok(());

        while let Some(mut future) = self.futures_queue.pop_front() {
            let waker = unsafe { Waker::from_raw(Self::raw_waker()) };
            let mut context = Context::from_waker(&waker);

            match future.as_mut().poll(&mut context) {
                Poll::Ready(v) if v.is_err() => {
                    result = v;
                    break;
                }
                Poll::Ready(_) => continue,
                Poll::Pending => unfinished_futures.push_back(future),
            }
        }

        for future in unfinished_futures {
            self.futures_queue.push_back(future);
        }

        result
    }

    /// Check if work remains in the executor.
    pub fn has_work(&mut self) -> bool {
        self.flush_channel();

        !self.futures_queue.is_empty()
    }

    /// Block until all futures complete or an error occurs.
    pub fn block_all(&mut self) -> Result<(), Error> {
        while self.has_work() {
            self.poll_all()?;
        }

        Ok(())
    }
}

/// A null implementation for platforms that do not live in a web browser.
///
/// The NullNavigatorBackend includes a trivial executor that holds owned
/// futures and runs them to completion, blockingly.
pub struct NullNavigatorBackend {
    /// The channel upon which all spawned futures will be sent.
    channel: Option<Sender<OwnedFuture<(), Error>>>,

    /// The base path for all relative fetches.
    relative_base_path: PathBuf,
}

impl NullNavigatorBackend {
    /// Construct a default navigator backend with no async or fetch
    /// capability.
    pub fn new() -> Self {
        NullNavigatorBackend {
            channel: None,
            relative_base_path: PathBuf::new(),
        }
    }

    /// Construct a navigator backend with fetch and async capability.
    pub fn with_base_path<P: AsRef<Path>>(
        path: P,
        channel: Sender<OwnedFuture<(), Error>>,
    ) -> Self {
        let mut relative_base_path = PathBuf::new();

        relative_base_path.push(path);

        NullNavigatorBackend {
            channel: Some(channel),
            relative_base_path,
        }
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

    fn fetch(&self, url: &str, _opts: RequestOptions) -> OwnedFuture<Vec<u8>, Error> {
        let mut path = self.relative_base_path.clone();
        path.push(url);

        Box::pin(async move { fs::read(path).map_err(Error::NetworkError) })
    }

    fn time_since_launch(&mut self) -> Duration {
        Duration::from_millis(0)
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        if let Some(channel) = self.channel.as_ref() {
            channel.send(future).unwrap();
        }
    }

    fn resolve_relative_url<'a>(&self, url: &'a str) -> Cow<'a, str> {
        let relative = url_from_relative_path(&self.relative_base_path, url);
        if let Ok(relative) = relative {
            String::from(relative).into()
        } else {
            url.into()
        }
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }
}
