//! Navigator backend for web
use crate::SocketProxy;
use async_channel::{Receiver, Sender};
use futures_util::future::Either;
use futures_util::{future, SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use js_sys::{Array, Promise, RegExp, Uint8Array};
use ruffle_core::backend::navigator::{
    async_return, create_fetch_error, create_specific_fetch_error, get_encoding, ErrorResponse,
    NavigationMethod, NavigatorBackend, OpenURLMode, OwnedFuture, Request, SuccessResponse,
};
use ruffle_core::config::NetworkingAccessMode;
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use ruffle_core::socket::{ConnectionState, SocketAction, SocketHandle};
use ruffle_core::swf::Encoding;
use ruffle_core::Player;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::Registry;
use tracing_wasm::WASMLayer;
use url::{ParseError, Url};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use wasm_streams::readable::ReadableStream;
use web_sys::{
    window, Blob, BlobPropertyBag, HtmlFormElement, HtmlInputElement, Request as WebRequest,
    RequestCredentials, RequestInit, Response as WebResponse,
};

pub struct WebNavigatorBackend {
    log_subscriber: Arc<Layered<WASMLayer, Registry>>,
    allow_script_access: bool,
    allow_networking: NetworkingAccessMode,
    upgrade_to_https: bool,
    url_rewrite_rules: Vec<(RegExp, String)>,
    base_url: Option<Url>,
    open_url_mode: OpenURLMode,
    socket_proxies: Vec<SocketProxy>,
    credential_allow_list: Vec<String>,
    player: Weak<Mutex<Player>>,
}

#[allow(clippy::too_many_arguments)]
impl WebNavigatorBackend {
    pub fn new(
        allow_script_access: bool,
        allow_networking: NetworkingAccessMode,
        upgrade_to_https: bool,
        url_rewrite_rules: Vec<(RegExp, String)>,
        base_url: Option<String>,
        log_subscriber: Arc<Layered<WASMLayer, Registry>>,
        open_url_mode: OpenURLMode,
        socket_proxies: Vec<SocketProxy>,
        credential_allow_list: Vec<String>,
    ) -> Self {
        let window = web_sys::window().expect("window()");

        // Upgrade to HTTPS takes effect if the current page is hosted on HTTPS.
        let upgrade_to_https =
            upgrade_to_https && window.location().protocol().expect("protocol()") == "https:";

        // Retrieve and parse `document.baseURI`.
        let document_base_uri = || {
            let document = window.document().expect("document()");
            if let Ok(Some(base_uri)) = document.base_uri() {
                return Url::parse(&base_uri).ok();
            }

            None
        };

        let base_url = if let Some(mut base_url) = base_url {
            // Adding trailing slash so `Url::parse` will not drop the last part.
            if !base_url.ends_with('/') {
                base_url.push('/');
            }

            Url::parse(&base_url)
                .ok()
                .or_else(|| document_base_uri().and_then(|base_uri| base_uri.join(&base_url).ok()))
        } else {
            document_base_uri()
        };

        if base_url.is_none() {
            tracing::error!("Could not get base URL for base directory inference.");
        }

        Self {
            allow_script_access,
            allow_networking,
            upgrade_to_https,
            url_rewrite_rules,
            base_url,
            log_subscriber,
            open_url_mode,
            socket_proxies,
            credential_allow_list,
            player: Weak::new(),
        }
    }

    /// We need to set the player after construction because the player is created after the navigator.
    pub fn set_player(&mut self, player: Weak<Mutex<Player>>) {
        self.player = player;
    }

    /// Try to rewrite the URL using URL rewrite rules.
    fn rewrite_url(&self, url: &Url) -> Option<Url> {
        let url_string: js_sys::JsString = url.to_string().into();
        for (regexp, replacement) in &self.url_rewrite_rules {
            if !url_string.search(regexp) >= 0 {
                continue;
            }

            tracing::info!(
                "URL rewrite rule triggered ({:?} -> {}) for URL {}",
                regexp,
                replacement,
                url
            );

            let replaced = url_string.replace_by_pattern(regexp, replacement);
            let replaced = replaced.as_string()?;
            match Url::parse(&replaced) {
                Ok(new_url) => {
                    return Some(new_url);
                }
                // Handle relative rewrite URLs
                Err(ParseError::RelativeUrlWithoutBase) if self.base_url.is_some() => {
                    let base_url = self.base_url.as_ref().expect("condition");
                    match base_url.join(&replaced) {
                        Ok(new_url) => {
                            return Some(new_url);
                        }
                        Err(err) => {
                            tracing::error!(
                                "Rewritten URL (relative) is not valid: {}, {}",
                                replaced,
                                err
                            );
                        }
                    }
                }
                Err(err) => {
                    tracing::error!(
                        "Rewritten URL (absolute) is not valid: {}, {}",
                        replaced,
                        err
                    );
                }
            }
            break;
        }
        None
    }
}

impl NavigatorBackend for WebNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: &str,
        target: &str,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        // If the URL is empty, ignore the request.
        if url.is_empty() {
            return;
        }

        let url = match self.resolve_url(url) {
            Ok(url) => {
                if url.scheme() == "file" {
                    tracing::error!(
                        "Can't open the local URL {} on WASM target",
                        url.to_string()
                    );
                    return;
                } else {
                    url
                }
            }
            Err(e) => {
                tracing::error!(
                    "Could not parse URL because of {}, the corrupt URL was: {}",
                    e,
                    url
                );
                return;
            }
        };

        // If `allowNetworking` is set to `internal` or `none`, block all `navigate_to_url` calls.
        if self.allow_networking != NetworkingAccessMode::All {
            tracing::warn!("SWF tried to open a URL, but opening URLs is not allowed");
            return;
        }

        // If `allowScriptAccess` is disabled, reject the `javascript:` scheme.
        // Also reject any attempt to open a URL when `target` is a keyword that affects the current tab.
        if !self.allow_script_access {
            if url.scheme() == "javascript" {
                tracing::warn!("SWF tried to run a script, but script access is not allowed");
                return;
            } else {
                match target.to_lowercase().as_str() {
                    "_parent" | "_self" | "_top" | "" => {
                        tracing::warn!("SWF tried to open a URL, but opening URLs in the current tab is prevented by script access");
                        return;
                    }
                    _ => (),
                }
            }
        }

        let window = window().expect("window()");

        if url.scheme() != "javascript" {
            if self.open_url_mode == OpenURLMode::Confirm {
                let message = format!("The SWF file wants to open the website {}", &url);
                // TODO: Add a checkbox with a GUI toolkit
                let confirm = window
                    .confirm_with_message(&message)
                    .expect("confirm_with_message()");
                if !confirm {
                    tracing::info!(
                        "SWF tried to open a website, but the user declined the request"
                    );
                    return;
                }
            } else if self.open_url_mode == OpenURLMode::Deny {
                tracing::warn!("SWF tried to open a website, but opening a website is not allowed");
                return;
            }
            // If the user confirmed or if in `Allow` mode, open the website.
        }

        // TODO: Should we return a result for failed opens? Does Flash care?
        match vars_method {
            Some((navmethod, formvars)) => {
                let document = window.document().expect("document()");
                let body = match document.body() {
                    Some(body) => body,
                    None => return,
                };

                let form: HtmlFormElement = document
                    .create_element("form")
                    .expect("create_element() must succeed")
                    .dyn_into()
                    .expect("create_element(\"form\") didn't give us a form");

                form.set_method(&navmethod.to_string());
                form.set_action(url.as_str());

                if !target.is_empty() {
                    form.set_target(target);
                }

                for (key, value) in formvars {
                    let hidden: HtmlInputElement = document
                        .create_element("input")
                        .expect("create_element() must succeed")
                        .dyn_into()
                        .expect("create_element(\"input\") didn't give us an input");

                    hidden.set_type("hidden");
                    hidden.set_name(&key);
                    hidden.set_value(&value);

                    let _ = form.append_child(&hidden);
                }

                let _ = body.append_child(&form);
                let _ = form.submit();
            }
            None => {
                if target.is_empty() {
                    let _ = window.location().assign(url.as_str());
                } else {
                    let _ = window.open_with_url_and_target(url.as_str(), target);
                }
            }
        };
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse> {
        let url = match self.resolve_url(request.url()) {
            Ok(url) => {
                if url.scheme() == "file" {
                    return async_return(create_specific_fetch_error(
                        "WASM target can't fetch local URL",
                        url.as_str(),
                        "",
                    ));
                } else {
                    url
                }
            }
            Err(e) => {
                return async_return(create_fetch_error(request.url(), e));
            }
        };

        let credentials = if let Some(host) = url.host_str() {
            if self
                .credential_allow_list
                .iter()
                .any(|allowed| allowed == &format!("{}://{}", url.scheme(), host))
            {
                RequestCredentials::Include
            } else {
                RequestCredentials::SameOrigin
            }
        } else {
            RequestCredentials::SameOrigin
        };

        Box::pin(async move {
            let init = RequestInit::new();

            init.set_method(&request.method().to_string());
            init.set_credentials(credentials);

            if let Some((data, mime)) = request.body() {
                let options = BlobPropertyBag::new();
                options.set_type(mime);
                let blob = Blob::new_with_buffer_source_sequence_and_options(
                    &Array::from_iter([Uint8Array::from(data.as_slice()).buffer()]),
                    &options,
                )
                .map_err(|_| ErrorResponse {
                    url: url.to_string(),
                    error: Error::FetchError("Got JS error".to_string()),
                })?
                .dyn_into()
                .map_err(|_| ErrorResponse {
                    url: url.to_string(),
                    error: Error::FetchError("Got JS error".to_string()),
                })?;

                init.set_body(&blob);
            }

            let web_request = match WebRequest::new_with_str_and_init(url.as_str(), &init) {
                Ok(web_request) => web_request,
                Err(_) => {
                    return create_specific_fetch_error(
                        "Unable to create request for",
                        url.as_str(),
                        "",
                    );
                }
            };

            let headers = web_request.headers();

            for (header_name, header_val) in request.headers() {
                headers
                    .set(header_name, header_val)
                    .map_err(|_| ErrorResponse {
                        url: url.to_string(),
                        error: Error::FetchError("Got JS error".to_string()),
                    })?;
            }

            let window = web_sys::window().expect("window()");
            let fetchval = JsFuture::from(window.fetch_with_request(&web_request))
                .await
                .map_err(|_| ErrorResponse {
                    url: url.to_string(),
                    error: Error::FetchError("Got JS error".to_string()),
                })?;

            let response: WebResponse = fetchval.dyn_into().map_err(|_| ErrorResponse {
                url: url.to_string(),
                error: Error::FetchError("Fetch result wasn't a WebResponse".to_string()),
            })?;
            let url = response.url();
            let status = response.status();
            let redirected = response.redirected();
            if !response.ok() {
                let error = Error::HttpNotOk(
                    format!("HTTP status is not ok, got {}", response.status_text()),
                    status,
                    redirected,
                    0,
                );
                return Err(ErrorResponse { url, error });
            }

            let wrapper: Box<dyn SuccessResponse> = Box::new(WebResponseWrapper {
                response,
                body_stream: None,
            });

            Ok(wrapper)
        })
    }

    fn resolve_url(&self, url: &str) -> Result<Url, ParseError> {
        if let Some(base_url) = &self.base_url {
            match base_url.join(url) {
                Ok(full_url) => Ok(self.pre_process_url(full_url)),
                Err(error) => Err(error),
            }
        } else {
            match Url::parse(url) {
                Ok(parsed_url) => Ok(self.pre_process_url(parsed_url)),
                Err(error) => Err(error),
            }
        }
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        let subscriber = self.log_subscriber.clone();
        let player = self.player.clone();

        spawn_local(async move {
            let _subscriber = tracing::subscriber::set_default(subscriber.clone());
            if player
                .upgrade()
                .expect("Called spawn_future after player was dropped")
                .try_lock()
                .is_err()
            {
                // The player is locked - this can occur due to 'wasm-bindgen-futures' using
                // 'queueMicroTask', which may result in one of our future's getting polled
                // while we're still inside of our 'requestAnimationFrame' callback (e.g.
                // when we call into javascript).
                //
                // When this happens, we 'reschedule' this future by waiting for a 'setTimeout'
                // callback to be resolved. This will cause our future to get woken up from
                // inside the 'setTimeout' JavaScript task (which is a new top-level call stack),
                // outside of the 'requestAnimationFrame' callback, which will allow us to lock
                // the Player.
                let promise = Promise::new(&mut |resolve, _reject| {
                    web_sys::window()
                        .expect("window")
                        .set_timeout_with_callback(&resolve)
                        .expect("Failed to call setTimeout with dummy promise");
                });
                let _ = JsFuture::from(promise).await;
            }
            if let Err(e) = future.await {
                tracing::error!("Asynchronous error occurred: {}", e);
            }
        })
    }

    fn pre_process_url(&self, mut url: Url) -> Url {
        if let Some(rewritten_url) = self.rewrite_url(&url) {
            url = rewritten_url;
        }

        if self.upgrade_to_https && url.scheme() == "http" && url.set_scheme("https").is_err() {
            tracing::error!("Url::set_scheme failed on: {}", url);
        }

        url
    }

    fn connect_socket(
        &mut self,
        host: String,
        port: u16,
        // NOTE: WebSocket does not allow specifying a timeout, so this goes unused.
        _timeout: Duration,
        handle: SocketHandle,
        receiver: Receiver<Vec<u8>>,
        sender: Sender<SocketAction>,
    ) {
        let Some(proxy) = self
            .socket_proxies
            .iter()
            .find(|x| x.host == host && x.port == port)
        else {
            tracing::warn!("Missing WebSocket proxy for host {}, port {}", host, port);
            sender
                .try_send(SocketAction::Connect(handle, ConnectionState::Failed))
                .expect("working channel send");
            return;
        };

        tracing::info!("Connecting to {}", proxy.proxy_url);

        let ws = match WebSocket::open(&proxy.proxy_url) {
            Ok(x) => x,
            Err(e) => {
                tracing::error!("Failed to create WebSocket, reason {:?}", e);
                sender
                    .try_send(SocketAction::Connect(handle, ConnectionState::Failed))
                    .expect("working channel send");
                return;
            }
        };

        let (mut ws_write, mut ws_read) = ws.split();
        sender
            .try_send(SocketAction::Connect(handle, ConnectionState::Connected))
            .expect("working channel send");

        self.spawn_future(Box::pin(async move {
            loop {
                match future::select(ws_read.next(), std::pin::pin!(receiver.recv())).await {
                    // Handle incoming messages.
                    Either::Left((Some(msg), _)) => match msg {
                        Ok(Message::Bytes(buf)) => sender
                            .try_send(SocketAction::Data(handle, buf))
                            .expect("working channel send"),
                        Ok(_) => tracing::warn!("Server sent an unexpected text message"),
                        Err(_) => {
                            sender
                                .try_send(SocketAction::Close(handle))
                                .expect("working channel send");
                            break;
                        }
                    },
                    // Handle outgoing messages.
                    Either::Right((Ok(msg), _)) => {
                        if let Err(e) = ws_write.send(Message::Bytes(msg)).await {
                            tracing::warn!("Failed to send message to WebSocket {}", e);
                            sender
                                .try_send(SocketAction::Close(handle))
                                .expect("working channel send");
                        }
                    }
                    // The connection was closed.
                    _ => break,
                };
            }

            let ws = ws_write
                .reunite(ws_read)
                .expect("both originate from the same websocket");
            let _ = ws.close(None, None);

            Ok(())
        }));
    }
}

struct WebResponseWrapper {
    response: WebResponse,
    body_stream: Option<Rc<RefCell<ReadableStream>>>,
}

impl SuccessResponse for WebResponseWrapper {
    fn url(&self) -> Cow<str> {
        Cow::Owned(self.response.url())
    }

    fn body(self: Box<Self>) -> OwnedFuture<Vec<u8>, Error> {
        Box::pin(async move {
            let body = JsFuture::from(
                self.response
                    .array_buffer()
                    .map_err(|_| Error::FetchError("Got JS error".to_string()))?,
            )
            .await
            .map_err(|_| {
                Error::FetchError("Could not allocate array buffer for response".to_string())
            })?
            .dyn_into()
            .map_err(|_| {
                Error::FetchError("array_buffer result wasn't an ArrayBuffer".to_string())
            })?;
            let body = Uint8Array::new(&body).to_vec();

            Ok(body)
        })
    }

    fn text_encoding(&self) -> Option<&'static Encoding> {
        if let Ok(Some(content_type)) = self.response.headers().get("Content-Type") {
            get_encoding(&content_type)
        } else {
            None
        }
    }

    fn status(&self) -> u16 {
        self.response.status()
    }

    fn redirected(&self) -> bool {
        self.response.redirected()
    }

    #[allow(clippy::await_holding_refcell_ref)]
    fn next_chunk(&mut self) -> OwnedFuture<Option<Vec<u8>>, Error> {
        if self.body_stream.is_none() {
            let body = self.response.body();
            if body.is_none() {
                return Box::pin(async move { Ok(None) });
            }

            self.body_stream = Some(Rc::new(RefCell::new(ReadableStream::from_raw(
                body.expect("body").unchecked_into(),
            ))));
        }

        let body_stream = self.body_stream.clone().expect("web body stream");
        Box::pin(async move {
            let read_lock = body_stream.try_borrow_mut();
            if read_lock.is_err() {
                return Err(Error::FetchError(
                    "Concurrent read operations on the same stream are not supported.".to_string(),
                ));
            }

            let mut read_lock = read_lock.expect("web response reader");
            let mut body_reader = read_lock.get_reader();

            let chunk = body_reader.read();
            match chunk.await {
                Ok(Some(chunk)) => Ok(Some(Uint8Array::new(&chunk).to_vec())),
                Ok(None) => Ok(None),
                Err(_) => Err(Error::FetchError("Cannot read next chunk".to_string())), //TODO: JsValue to string?!
            }
        })
    }

    fn expected_length(&self) -> Result<Option<u64>, Error> {
        let length = self
            .response
            .headers()
            .get("Content-Length")
            .map_err(|js_err| {
                Error::FetchError(
                    (js_err + JsValue::from(""))
                        .as_string()
                        .expect("JavaScript String addition to yield String"),
                )
            })?;

        if let Some(length) = length {
            Ok(Some(length.parse::<u64>()?))
        } else {
            Ok(None)
        }
    }
}
