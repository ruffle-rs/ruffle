//! Navigator backend for web

use crate::custom_event::RuffleEvent;
use async_channel::{Receiver, TryRecvError};
use async_io::Timer;
use async_net::TcpStream;
use futures::future::select;
use futures::{AsyncReadExt, AsyncWriteExt};
use futures_lite::FutureExt;
use isahc::http::{HeaderName, HeaderValue};
use isahc::{
    config::RedirectPolicy, prelude::*, AsyncReadResponseExt, HttpClient, Request as IsahcRequest,
};
use rfd::{AsyncMessageDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use ruffle_core::backend::navigator::{
    async_return, create_fetch_error, create_specific_fetch_error, ErrorResponse, NavigationMethod,
    NavigatorBackend, OpenURLMode, OwnedFuture, Request, SocketMode, SuccessResponse,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use ruffle_core::socket::{ConnectionState, SocketAction, SocketHandle};
use std::collections::HashSet;
use std::io;
use std::io::ErrorKind;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::mpsc::Sender;
use std::time::Duration;
use tracing::warn;
use url::{ParseError, Url};
use winit::event_loop::EventLoopProxy;

/// Implementation of `NavigatorBackend` for non-web environments that can call
/// out to a web browser.
pub struct ExternalNavigatorBackend {
    /// Sink for tasks sent to us through `spawn_future`.
    channel: Sender<OwnedFuture<(), Error>>,

    /// Event sink to trigger a new task poll.
    event_loop: EventLoopProxy<RuffleEvent>,

    /// The url to use for all relative fetches.
    base_url: Url,

    // Client to use for network requests
    client: Option<Rc<HttpClient>>,

    socket_allowed: HashSet<String>,

    socket_mode: SocketMode,

    upgrade_to_https: bool,

    open_url_mode: OpenURLMode,
}

impl ExternalNavigatorBackend {
    /// Construct a navigator backend with fetch and async capability.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mut base_url: Url,
        channel: Sender<OwnedFuture<(), Error>>,
        event_loop: EventLoopProxy<RuffleEvent>,
        proxy: Option<Url>,
        upgrade_to_https: bool,
        open_url_mode: OpenURLMode,
        socket_allowed: HashSet<String>,
        socket_mode: SocketMode,
    ) -> Self {
        let proxy = proxy.and_then(|url| url.as_str().parse().ok());
        let builder = HttpClient::builder()
            .proxy(proxy)
            .cookies()
            .redirect_policy(RedirectPolicy::Follow);

        let client = builder.build().ok().map(Rc::new);

        // Force replace the last segment with empty. //

        if let Ok(mut base_url) = base_url.path_segments_mut() {
            base_url.pop().pop_if_empty().push("");
        }

        Self {
            channel,
            event_loop,
            client,
            base_url,
            upgrade_to_https,
            open_url_mode,
            socket_allowed,
            socket_mode,
        }
    }
}

impl NavigatorBackend for ExternalNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: &str,
        _target: &str,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        //TODO: Should we return a result for failed opens? Does Flash care?

        //NOTE: Flash desktop players / projectors ignore the window parameter,
        //      unless it's a `_layer`, and we shouldn't handle that anyway.
        let mut parsed_url = match self.resolve_url(url) {
            Ok(parsed_url) => parsed_url,
            Err(e) => {
                tracing::error!(
                    "Could not parse URL because of {}, the corrupt URL was: {}",
                    e,
                    url
                );
                return;
            }
        };

        let modified_url = match vars_method {
            Some((_, query_pairs)) => {
                {
                    //lifetime limiter because we don't have NLL yet
                    let mut modifier = parsed_url.query_pairs_mut();

                    for (k, v) in query_pairs.iter() {
                        modifier.append_pair(k, v);
                    }
                }

                parsed_url
            }
            None => parsed_url,
        };

        if modified_url.scheme() == "javascript" {
            tracing::warn!(
                "SWF tried to run a script on desktop, but javascript calls are not allowed"
            );
            return;
        }

        if self.open_url_mode == OpenURLMode::Confirm {
            let message = format!("The SWF file wants to open the website {}", modified_url);
            // TODO: Add a checkbox with a GUI toolkit
            let confirm = MessageDialog::new()
                .set_title("Open website?")
                .set_level(MessageLevel::Info)
                .set_description(message)
                .set_buttons(MessageButtons::OkCancel)
                .show()
                == MessageDialogResult::Ok;
            if !confirm {
                tracing::info!("SWF tried to open a website, but the user declined the request");
                return;
            }
        } else if self.open_url_mode == OpenURLMode::Deny {
            tracing::warn!("SWF tried to open a website, but opening a website is not allowed");
            return;
        }

        // If the user confirmed or if in Allow mode, open the website

        // TODO: This opens local files in the browser while flash opens them
        // in the default program for the respective filetype.
        // This especially includes mailto links. Ruffle opens the browser which opens
        // the preferred program while flash opens the preferred program directly.
        match webbrowser::open(modified_url.as_ref()) {
            Ok(_output) => {}
            Err(e) => tracing::error!("Could not open URL {}: {}", modified_url.as_str(), e),
        };
    }

    fn fetch(&self, request: Request) -> OwnedFuture<SuccessResponse, ErrorResponse> {
        // TODO: honor sandbox type (local-with-filesystem, local-with-network, remote, ...)
        let mut processed_url = match self.resolve_url(request.url()) {
            Ok(url) => url,
            Err(e) => {
                return async_return(create_fetch_error(request.url(), e));
            }
        };

        let client = self.client.clone();

        match processed_url.scheme() {
            "file" => Box::pin(async move {
                // We send the original url (including query parameters)
                // back to ruffle_core in the `Response`
                let response_url = processed_url.clone();
                // Flash supports query parameters with local urls.
                // SwfMovie takes care of exposing those to ActionScript -
                // when we actually load a filesystem url, strip them out.
                processed_url.set_query(None);

                let path = match processed_url.to_file_path() {
                    Ok(path) => path,
                    Err(_) => {
                        return create_specific_fetch_error(
                            "Unable to create path out of URL",
                            response_url.as_str(),
                            "",
                        )
                    }
                };

                let contents = std::fs::read(&path).or_else(|e| {
                    if cfg!(feature = "sandbox") {
                        use rfd::FileDialog;

                        if e.kind() == ErrorKind::PermissionDenied {
                            let attempt_sandbox_open = MessageDialog::new()
                                .set_level(MessageLevel::Warning)
                                .set_description(format!("The current movie is attempting to read files stored in {}.\n\nTo allow it to do so, click Yes, and then Open to grant read access to that directory.\n\nOtherwise, click No to deny access.", path.parent().unwrap_or(&path).to_string_lossy()))
                                .set_buttons(MessageButtons::YesNo)
                                .show() == MessageDialogResult::Yes;

                            if attempt_sandbox_open {
                                FileDialog::new().set_directory(&path).pick_folder();

                                return std::fs::read(&path);
                            }
                        }
                    }

                    Err(e)
                });

                let body = match contents {
                    Ok(body) => body,
                    Err(e) => {
                        return create_specific_fetch_error(
                            "Can't open file",
                            response_url.as_str(),
                            e,
                        )
                    }
                };

                Ok(SuccessResponse {
                    url: response_url.to_string(),
                    body,
                    status: 0,
                    redirected: false,
                })
            }),
            _ => Box::pin(async move {
                let client = client.ok_or_else(|| ErrorResponse {
                    url: processed_url.to_string(),
                    error: Error::FetchError("Network unavailable".to_string()),
                })?;

                let mut isahc_request = match request.method() {
                    NavigationMethod::Get => IsahcRequest::get(processed_url.to_string()),
                    NavigationMethod::Post => IsahcRequest::post(processed_url.to_string()),
                };
                let (body_data, mime) = request.body().clone().unwrap_or_default();
                if let Some(headers) = isahc_request.headers_mut() {
                    for (name, val) in request.headers().iter() {
                        headers.insert(
                            HeaderName::from_str(name).map_err(|e| ErrorResponse {
                                url: processed_url.to_string(),
                                error: Error::FetchError(e.to_string()),
                            })?,
                            HeaderValue::from_str(val).map_err(|e| ErrorResponse {
                                url: processed_url.to_string(),
                                error: Error::FetchError(e.to_string()),
                            })?,
                        );
                    }
                    headers.insert(
                        "Content-Type",
                        HeaderValue::from_str(&mime).map_err(|e| ErrorResponse {
                            url: processed_url.to_string(),
                            error: Error::FetchError(e.to_string()),
                        })?,
                    );
                }

                let body = isahc_request.body(body_data).map_err(|e| ErrorResponse {
                    url: processed_url.to_string(),
                    error: Error::FetchError(e.to_string()),
                })?;

                let mut response = client.send_async(body).await.map_err(|e| {
                    let inner = match e.kind() {
                        isahc::error::ErrorKind::NameResolution => {
                            Error::InvalidDomain(processed_url.to_string())
                        }
                        _ => Error::FetchError(e.to_string()),
                    };
                    ErrorResponse {
                        url: processed_url.to_string(),
                        error: inner,
                    }
                })?;

                let url = if let Some(uri) = response.effective_uri() {
                    uri.to_string()
                } else {
                    processed_url.into()
                };

                let status = response.status().as_u16();
                let redirected = response.effective_uri().is_some();
                if !response.status().is_success() {
                    let error = Error::HttpNotOk(
                        format!("HTTP status is not ok, got {}", response.status()),
                        status,
                        redirected,
                        response.body().len().unwrap_or(0),
                    );
                    return Err(ErrorResponse { url, error });
                }

                let mut body = vec![];
                response
                    .copy_to(&mut body)
                    .await
                    .map_err(|e| ErrorResponse {
                        url: url.clone(),
                        error: Error::FetchError(e.to_string()),
                    })?;

                Ok(SuccessResponse {
                    url,
                    body,
                    status,
                    redirected,
                })
            }),
        }
    }

    fn resolve_url(&self, url: &str) -> Result<Url, ParseError> {
        match self.base_url.join(url) {
            Ok(url) => Ok(self.pre_process_url(url)),
            Err(error) => Err(error),
        }
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.channel.send(future).expect("working channel send");

        if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
            tracing::warn!(
                "A task was queued on an event loop that has already ended. It will not be polled."
            );
        }
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
        timeout: Duration,
        handle: SocketHandle,
        receiver: Receiver<Vec<u8>>,
        sender: Sender<SocketAction>,
    ) {
        let addr = format!("{}:{}", host, port);
        let is_allowed = self.socket_allowed.contains(&addr);
        let socket_mode = self.socket_mode;

        let future = Box::pin(async move {
            match (is_allowed, socket_mode) {
                (false, SocketMode::Allow) | (true, _) => {} // the process is allowed to continue. just dont do anything.
                (false, SocketMode::Deny) => {
                    // Just fail the connection.
                    sender
                        .send(SocketAction::Connect(handle, ConnectionState::Failed))
                        .expect("working channel send");

                    tracing::warn!(
                        "SWF tried to open a socket, but opening a socket is not allowed"
                    );

                    return Ok(());
                }
                (false, SocketMode::Ask) => {
                    let attempt_sandbox_connect = AsyncMessageDialog::new().set_level(MessageLevel::Warning).set_description(format!("The current movie is attempting to connect to {:?} (port {}).\n\nTo allow it to do so, click Yes to grant network access to that host.\n\nOtherwise, click No to deny access.", host, port)).set_buttons(MessageButtons::YesNo)
                    .show()
                    .await == MessageDialogResult::Yes;

                    if !attempt_sandbox_connect {
                        // fail the connection.
                        sender
                            .send(SocketAction::Connect(handle, ConnectionState::Failed))
                            .expect("working channel send");

                        return Ok(());
                    }
                }
            }

            let host2 = host.clone();

            let timeout = async {
                Timer::after(timeout).await;
                Result::<TcpStream, io::Error>::Err(io::Error::new(ErrorKind::TimedOut, ""))
            };

            let stream = match TcpStream::connect((host, port)).or(timeout).await {
                Err(e) if e.kind() == ErrorKind::TimedOut => {
                    warn!("Connection to {}:{} timed out", host2, port);
                    sender
                        .send(SocketAction::Connect(handle, ConnectionState::TimedOut))
                        .expect("working channel send");
                    return Ok(());
                }
                Ok(stream) => {
                    sender
                        .send(SocketAction::Connect(handle, ConnectionState::Connected))
                        .expect("working channel send");

                    stream
                }
                Err(err) => {
                    warn!("Failed to connect to {}:{}, error: {}", host2, port, err);
                    sender
                        .send(SocketAction::Connect(handle, ConnectionState::Failed))
                        .expect("working channel send");
                    return Ok(());
                }
            };

            let sender = sender;
            //NOTE: We clone the sender here as we cant share it between async tasks.
            let sender2 = sender.clone();
            let (mut read, mut write) = stream.split();

            let read = std::pin::pin!(async move {
                loop {
                    let mut buffer = [0; 4096];

                    match read.read(&mut buffer).await {
                        Err(e) if e.kind() == ErrorKind::TimedOut => {} // try again later.
                        Err(_) | Ok(0) => {
                            sender
                                .send(SocketAction::Close(handle))
                                .expect("working channel send");
                            drop(read);
                            break;
                        }
                        Ok(read) => {
                            let buffer = buffer.into_iter().take(read).collect::<Vec<_>>();

                            sender
                                .send(SocketAction::Data(handle, buffer))
                                .expect("working channel send");
                        }
                    };
                }
            });

            let write = std::pin::pin!(async move {
                let mut pending_write = vec![];

                loop {
                    loop {
                        match receiver.try_recv() {
                            Ok(val) => {
                                pending_write.extend(val);
                            }
                            Err(TryRecvError::Closed) => {
                                //NOTE: Channel sender has been dropped.
                                //      This means we have to close the connection.
                                drop(write);
                                return;
                            }
                            Err(_) => break,
                        }
                    }

                    if !pending_write.is_empty() {
                        match write.write(&pending_write).await {
                            Err(e) if e.kind() == ErrorKind::TimedOut => {} // try again later.
                            Err(_) => {
                                sender2
                                    .send(SocketAction::Close(handle))
                                    .expect("working channel send");
                                drop(write);
                                return;
                            }
                            Ok(written) => {
                                let _ = pending_write.drain(..written);
                            }
                        }
                    } else {
                        //NOTE: We wait here as if the buffer is empty the syscall (at least on linux),
                        //      will return immediately, and because of that we get stuck in a infinite loop
                        //      as we never yield to the executor.
                        Timer::after(Duration::from_millis(10)).await;
                    }
                }
            });

            //NOTE: If one future exits, this will take the other one down too.
            select(read, write).await;

            Ok(())
        });

        self.spawn_future(future);
    }
}
