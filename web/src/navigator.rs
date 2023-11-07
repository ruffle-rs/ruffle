//! Navigator backend for web
use crate::SocketProxy;
use async_channel::Receiver;
use futures_util::{SinkExt, StreamExt};
use gloo_net::websocket::{futures::WebSocket, Message};
use js_sys::{Array, ArrayBuffer, Uint8Array};
use ruffle_core::backend::navigator::{
    async_return, create_fetch_error, create_specific_fetch_error, ErrorResponse, NavigationMethod,
    NavigatorBackend, OpenURLMode, OwnedFuture, Request, SuccessResponse,
};
use ruffle_core::config::NetworkingAccessMode;
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use ruffle_core::socket::{ConnectionState, SocketAction, SocketHandle};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::Registry;
use tracing_wasm::WASMLayer;
use url::{ParseError, Url};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    window, Blob, BlobPropertyBag, HtmlFormElement, HtmlInputElement, Request as WebRequest,
    RequestCredentials, RequestInit, Response as WebResponse,
};

pub struct WebNavigatorBackend {
    log_subscriber: Arc<Layered<WASMLayer, Registry>>,
    allow_script_access: bool,
    allow_networking: NetworkingAccessMode,
    upgrade_to_https: bool,
    base_url: Option<Url>,
    open_url_mode: OpenURLMode,
    socket_proxies: Vec<SocketProxy>,
    credential_allow_list: Vec<String>,
}

#[allow(clippy::too_many_arguments)]
impl WebNavigatorBackend {
    pub fn new(
        allow_script_access: bool,
        allow_networking: NetworkingAccessMode,
        upgrade_to_https: bool,
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
            base_url,
            log_subscriber,
            open_url_mode,
            socket_proxies,
            credential_allow_list,
        }
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

    fn fetch(&self, request: Request) -> OwnedFuture<SuccessResponse, ErrorResponse> {
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
            let mut init = RequestInit::new();

            init.method(&request.method().to_string());
            init.credentials(credentials);

            if let Some((data, mime)) = request.body() {
                let blob = Blob::new_with_buffer_source_sequence_and_options(
                    &Array::from_iter([Uint8Array::from(data.as_slice()).buffer()]),
                    BlobPropertyBag::new().type_(mime),
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

                init.body(Some(&blob));
            }

            let web_request = match WebRequest::new_with_str_and_init(url.as_str(), &init) {
                Ok(web_request) => web_request,
                Err(_) => {
                    return create_specific_fetch_error(
                        "Unable to create request for",
                        url.as_str(),
                        "",
                    )
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

            let body: ArrayBuffer = JsFuture::from(response.array_buffer().map_err(|_| {
                ErrorResponse {
                    url: url.clone(),
                    error: Error::FetchError("Got JS error".to_string()),
                }
            })?)
            .await
            .map_err(|_| ErrorResponse {
                url: url.clone(),
                error: Error::FetchError(
                    "Could not allocate array buffer for response".to_string(),
                ),
            })?
            .dyn_into()
            .map_err(|_| ErrorResponse {
                url: url.clone(),
                error: Error::FetchError("array_buffer result wasn't an ArrayBuffer".to_string()),
            })?;
            let body = Uint8Array::new(&body).to_vec();

            Ok(SuccessResponse {
                url,
                body,
                status,
                redirected,
            })
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
        spawn_local(async move {
            let _subscriber = tracing::subscriber::set_default(subscriber);
            if let Err(e) = future.await {
                tracing::error!("Asynchronous error occurred: {}", e);
            }
        })
    }

    fn pre_process_url(&self, mut url: Url) -> Url {
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
                .send(SocketAction::Connect(handle, ConnectionState::Failed))
                .expect("working channel send");
            return;
        };

        tracing::info!("Connecting to {}", proxy.proxy_url);

        let ws = match WebSocket::open(&proxy.proxy_url) {
            Ok(x) => x,
            Err(e) => {
                tracing::error!("Failed to create WebSocket, reason {:?}", e);
                sender
                    .send(SocketAction::Connect(handle, ConnectionState::Failed))
                    .expect("working channel send");
                return;
            }
        };

        let (mut sink, mut stream) = ws.split();
        sender
            .send(SocketAction::Connect(handle, ConnectionState::Connected))
            .expect("working channel send");

        // Spawn future to handle incoming messages.
        let stream_sender = sender.clone();
        self.spawn_future(Box::pin(async move {
            while let Some(msg) = stream.next().await {
                match msg {
                    Ok(Message::Bytes(buf)) => stream_sender
                        .send(SocketAction::Data(handle, buf))
                        .expect("working channel send"),
                    Ok(_) => tracing::warn!("Server sent unexpected text message"),
                    Err(_) => {
                        stream_sender
                            .send(SocketAction::Close(handle))
                            .expect("working channel send");
                        return Ok(());
                    }
                }
            }

            Ok(())
        }));

        // Spawn future to handle outgoing messages.
        self.spawn_future(Box::pin(async move {
            while let Ok(msg) = receiver.recv().await {
                if let Err(e) = sink.send(Message::Bytes(msg)).await {
                    tracing::warn!("Failed to send message to WebSocket {}", e);
                    sender
                        .send(SocketAction::Close(handle))
                        .expect("working channel send");
                }
            }

            Ok(())
        }));
    }
}
