//! Navigator backend for web

use crate::executor::FutureSpawner;
use async_channel::{Receiver, Sender, TryRecvError};
use async_io::Timer;
use async_net::TcpStream;
use futures::future::select;
use futures::{AsyncReadExt, AsyncWriteExt};
use futures_lite::FutureExt;
use isahc::http::{HeaderName, HeaderValue};
use isahc::{
    config::RedirectPolicy, prelude::*, AsyncBody, HttpClient, Request as IsahcRequest,
    Response as IsahcResponse,
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
use std::fs::File;
use std::io::ErrorKind;
use std::io::{self, Read};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::warn;
use url::{ParseError, Url};

/// Implementation of `NavigatorBackend` for non-web environments that can call
/// out to a web browser.
pub struct ExternalNavigatorBackend<F: FutureSpawner> {
    /// Sink for tasks sent to us through `spawn_future`.
    future_spawner: F,

    /// The url to use for all relative fetches.
    base_url: Url,

    // Client to use for network requests
    client: Option<Rc<HttpClient>>,

    socket_allowed: HashSet<String>,

    socket_mode: SocketMode,

    upgrade_to_https: bool,

    open_url_mode: OpenURLMode,
}

impl<F: FutureSpawner> ExternalNavigatorBackend<F> {
    /// Construct a navigator backend with fetch and async capability.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mut base_url: Url,
        future_spawner: F,
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
            future_spawner,
            client,
            base_url,
            upgrade_to_https,
            open_url_mode,
            socket_allowed,
            socket_mode,
        }
    }
}

impl<F: FutureSpawner> NavigatorBackend for ExternalNavigatorBackend<F> {
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

    fn fetch(&self, request: Request) -> OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse> {
        enum DesktopResponseBody {
            /// The response's body comes from a file.
            File(File),

            /// The response's body comes from the network.
            ///
            /// This has to be stored in shared ownership so that we can return
            /// owned futures. A synchronous lock is used here as we do not
            /// expect contention on this lock.
            Network(Arc<Mutex<IsahcResponse<AsyncBody>>>),
        }

        struct DesktopResponse {
            url: String,
            response_body: DesktopResponseBody,
            status: u16,
            redirected: bool,
        }

        impl SuccessResponse for DesktopResponse {
            fn url(&self) -> std::borrow::Cow<str> {
                std::borrow::Cow::Borrowed(&self.url)
            }

            #[allow(clippy::await_holding_lock)]
            fn body(self: Box<Self>) -> OwnedFuture<Vec<u8>, Error> {
                match self.response_body {
                    DesktopResponseBody::File(mut file) => Box::pin(async move {
                        let mut body = vec![];
                        file.read_to_end(&mut body)
                            .map_err(|e| Error::FetchError(e.to_string()))?;

                        Ok(body)
                    }),
                    DesktopResponseBody::Network(response) => Box::pin(async move {
                        let mut body = vec![];
                        response
                            .lock()
                            .expect("working lock during fetch body read")
                            .copy_to(&mut body)
                            .await
                            .map_err(|e| Error::FetchError(e.to_string()))?;

                        Ok(body)
                    }),
                }
            }

            fn status(&self) -> u16 {
                self.status
            }

            fn redirected(&self) -> bool {
                self.redirected
            }

            #[allow(clippy::await_holding_lock)]
            fn next_chunk(&mut self) -> OwnedFuture<Option<Vec<u8>>, Error> {
                match &mut self.response_body {
                    DesktopResponseBody::File(file) => {
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
                    DesktopResponseBody::Network(response) => {
                        let response = response.clone();

                        Box::pin(async move {
                            let mut buf = vec![0; 4096];
                            let lock = response.try_lock();
                            if matches!(lock, Err(std::sync::TryLockError::WouldBlock)) {
                                return Err(Error::FetchError(
                                    "Concurrent read operations on the same stream are not supported."
                                        .to_string(),
                                ));
                            }

                            let result = lock
                                .expect("desktop network lock")
                                .body_mut()
                                .read(&mut buf)
                                .await;

                            match result {
                                Ok(count) if count > 0 => {
                                    buf.resize(count, 0);
                                    Ok(Some(buf))
                                }
                                Ok(_) => Ok(None),
                                Err(e) => Err(Error::FetchError(e.to_string())),
                            }
                        })
                    }
                }
            }

            fn expected_length(&self) -> Result<Option<u64>, Error> {
                match &self.response_body {
                    DesktopResponseBody::File(file) => Ok(Some(file.metadata()?.len())),
                    DesktopResponseBody::Network(response) => {
                        let response = response.lock().expect("no recursive locks");
                        let content_length = response.headers().get("Content-Length");

                        if let Some(len) = content_length {
                            Ok(Some(
                                len.to_str()
                                    .map_err(|_| Error::InvalidHeaderValue)?
                                    .parse::<u64>()?,
                            ))
                        } else {
                            Ok(None)
                        }
                    }
                }
            }
        }

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
                        );
                    }
                };

                let contents = std::fs::File::open(&path).or_else(|e| {
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

                                return std::fs::File::open(&path);
                            }
                        }
                    }

                    Err(e)
                });

                let file = match contents {
                    Ok(file) => file,
                    Err(e) => {
                        return create_specific_fetch_error(
                            "Can't open file",
                            response_url.as_str(),
                            e,
                        );
                    }
                };

                let response: Box<dyn SuccessResponse> = Box::new(DesktopResponse {
                    url: response_url.to_string(),
                    response_body: DesktopResponseBody::File(file),
                    status: 0,
                    redirected: false,
                });

                Ok(response)
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

                let response = client.send_async(body).await.map_err(|e| {
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

                let response: Box<dyn SuccessResponse> = Box::new(DesktopResponse {
                    url,
                    response_body: DesktopResponseBody::Network(Arc::new(Mutex::new(response))),
                    status,
                    redirected,
                });
                Ok(response)
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
        self.future_spawner.spawn(future);
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
                        .try_send(SocketAction::Connect(handle, ConnectionState::Failed))
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
                            .try_send(SocketAction::Connect(handle, ConnectionState::Failed))
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
                        .try_send(SocketAction::Connect(handle, ConnectionState::TimedOut))
                        .expect("working channel send");
                    return Ok(());
                }
                Ok(stream) => {
                    sender
                        .try_send(SocketAction::Connect(handle, ConnectionState::Connected))
                        .expect("working channel send");

                    stream
                }
                Err(err) => {
                    warn!("Failed to connect to {}:{}, error: {}", host2, port, err);
                    sender
                        .try_send(SocketAction::Connect(handle, ConnectionState::Failed))
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
                                .try_send(SocketAction::Close(handle))
                                .expect("working channel send");
                            drop(read);
                            break;
                        }
                        Ok(read) => {
                            let buffer = buffer.into_iter().take(read).collect::<Vec<_>>();

                            sender
                                .try_send(SocketAction::Data(handle, buffer))
                                .expect("working channel send");
                        }
                    };
                }
            });

            let write = std::pin::pin!(async move {
                let mut pending_write = vec![];

                loop {
                    let close_connection = loop {
                        match receiver.try_recv() {
                            Ok(val) => {
                                pending_write.extend(val);
                            }
                            Err(TryRecvError::Empty) => break false,
                            Err(TryRecvError::Closed) => {
                                //NOTE: Channel sender has been dropped.
                                //      This means we have to close the connection,
                                //      but not here, as we might have a pending write.
                                break true;
                            }
                        }
                    };

                    if !pending_write.is_empty() {
                        match write.write(&pending_write).await {
                            Err(e) if e.kind() == ErrorKind::TimedOut => {} // try again later.
                            Err(_) => {
                                sender2
                                    .try_send(SocketAction::Close(handle))
                                    .expect("working channel send");
                                drop(write);
                                return;
                            }
                            Ok(written) => {
                                let _ = pending_write.drain(..written);
                            }
                        }
                    } else if close_connection {
                        drop(write);
                        return;
                    } else {
                        // Receiver is empty and there's no pending data,
                        // we may block here and wait for new data.
                        match receiver.recv().await {
                            Ok(val) => {
                                pending_write.extend(val);
                            }
                            Err(_) => {
                                // Ignore the error here, it will be
                                // reported again in try_recv.
                            }
                        }
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use async_net::TcpListener;
    use ruffle_core::socket::SocketAction::{Close, Connect, Data};
    use std::net::SocketAddr;
    use tokio::task;

    use super::*;

    const TIMEOUT_ZERO: Duration = Duration::ZERO;
    // The timeout has to be large enough to allow "instantaneous" actions
    // and local IO to execute, but small enough to fail tests quickly.
    const TIMEOUT: Duration = Duration::from_secs(1);

    struct TestFutureSpawner;

    impl FutureSpawner for TestFutureSpawner {
        fn spawn(&self, future: OwnedFuture<(), Error>) {
            task::spawn_local(future);
        }
    }

    macro_rules! async_timeout {
        () => {
            async {
                Timer::after(TIMEOUT).await;
                panic!("An action which should complete timed out")
            }
        };
    }

    macro_rules! async_test {
        (
            async fn $test_name:ident() $content:block
        ) => {
            #[tokio::test(flavor = "current_thread")]
            async fn $test_name() {
                task::LocalSet::new().run_until(async move $content).await;
            }
        }
    }

    macro_rules! dummy_handle {
        () => {
            SocketHandle::default()
        };
    }

    macro_rules! assert_next_socket_actions {
        ($receiver:expr;) => {
            // no more actions
        };
        ($receiver:expr; $action:expr, $($more:expr,)*) => {
            assert_eq!($receiver.recv().or(async_timeout!()).await.expect("receive action"), $action);
            assert_next_socket_actions!($receiver; $($more,)*);
        };
    }

    fn new_test_backend(socket_allow: bool) -> ExternalNavigatorBackend<TestFutureSpawner> {
        ExternalNavigatorBackend::new(
            Url::parse("https://example.com/path/").unwrap(),
            TestFutureSpawner,
            None,
            false,
            OpenURLMode::Allow,
            Default::default(),
            if socket_allow {
                SocketMode::Allow
            } else {
                SocketMode::Deny
            },
        )
    }

    async fn start_test_server() -> (task::JoinHandle<TcpStream>, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let accept_task = task::spawn_local(async move {
            let (socket, _) = listener.accept().or(async_timeout!()).await.unwrap();
            socket
        });
        (accept_task, addr)
    }

    fn connect_test_socket(
        addr: SocketAddr,
        timeout: Duration,
        socket_allow: bool,
    ) -> (Sender<Vec<u8>>, Receiver<SocketAction>) {
        let mut backend = new_test_backend(socket_allow);

        let (write, receiver) = async_channel::unbounded();
        let (sender, read) = async_channel::unbounded();

        backend.connect_socket(
            addr.ip().to_string(),
            addr.port(),
            timeout,
            dummy_handle!(),
            receiver,
            sender,
        );

        (write, read)
    }

    async fn write_server(server_socket: &mut TcpStream, data: &str) {
        server_socket
            .write(data.as_bytes())
            .or(async_timeout!())
            .await
            .expect("server write");
    }

    async fn read_server(server_socket: &mut TcpStream) -> String {
        let mut buffer = [0; 4096];

        let read = match server_socket.read(&mut buffer).await {
            Err(e) => {
                panic!("server read error: {}", e);
            }
            Ok(read) => read,
        };

        let buffer = buffer.into_iter().take(read).collect::<Vec<_>>();
        String::from_utf8(buffer).unwrap()
    }

    async fn write_client(client_write: &Sender<Vec<u8>>, data: &str) {
        client_write
            .send(data.as_bytes().to_vec())
            .or(async_timeout!())
            .await
            .expect("client write");
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_timeout() {
        let (_accept_task, addr) = start_test_server().await;
        let (_client_write, client_read) = connect_test_socket(addr, TIMEOUT_ZERO, true);
        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::TimedOut),
        );
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_connect() {
        let (accept_task, addr) = start_test_server().await;
        let (_client_write, client_read) = connect_test_socket(addr, TIMEOUT, true);
        let _server_socket = accept_task.await.unwrap();
        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::Connected),
        );
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_deny() {
        let (_accept_task, addr) = start_test_server().await;
        let (_client_write, client_read) = connect_test_socket(addr, TIMEOUT, false);

        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::Failed),
        );
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_fail() {
        let addr = SocketAddr::from_str("[100::]:42").expect("black hole address");
        let (_client_write, client_read) = connect_test_socket(addr, TIMEOUT, true);

        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::Failed),
        );
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_server_close() {
        let (accept_task, addr) = start_test_server().await;
        let (_client_write, client_read) = connect_test_socket(addr, TIMEOUT, true);

        let server_socket = accept_task.await.unwrap();
        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::Connected),
        );

        drop(server_socket);

        assert_next_socket_actions!(
            client_read;
            Close(dummy_handle!()),
        );
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_client_close() {
        let (accept_task, addr) = start_test_server().await;
        let (client_write, client_read) = connect_test_socket(addr, TIMEOUT, true);

        let mut server_socket = accept_task.await.unwrap();
        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::Connected),
        );

        drop(client_write);

        assert_eq!(read_server(&mut server_socket).await, "");
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_basic_communication() {
        let (accept_task, addr) = start_test_server().await;
        let (client_write, client_read) = connect_test_socket(addr, TIMEOUT, true);

        let mut server_socket = accept_task.await.unwrap();
        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::Connected),
        );

        write_server(&mut server_socket, "Hello ").await;
        write_server(&mut server_socket, "World!").await;

        assert_next_socket_actions!(
            client_read;
            Data(dummy_handle!(), "Hello World!".as_bytes().to_vec()),
        );

        write_client(&client_write, "Hello from").await;
        write_client(&client_write, " client").await;

        assert_eq!(read_server(&mut server_socket).await, "Hello from client");

        write_server(&mut server_socket, "from server 2").await;
        write_client(&client_write, "from client 2").await;

        assert_next_socket_actions!(
            client_read;
            Data(dummy_handle!(), "from server 2".as_bytes().to_vec()),
        );
        assert_eq!(read_server(&mut server_socket).await, "from client 2");
    }

    #[macro_rules_attribute::apply(async_test)]
    async fn test_socket_flush_before_close() {
        let (accept_task, addr) = start_test_server().await;
        let (client_write, client_read) = connect_test_socket(addr, TIMEOUT, true);

        let mut server_socket = accept_task.await.unwrap();
        assert_next_socket_actions!(
            client_read;
            Connect(dummy_handle!(), ConnectionState::Connected),
        );

        write_client(&client_write, "Sending some").await;
        write_client(&client_write, " data").await;
        client_write.close();

        assert_eq!(read_server(&mut server_socket).await, "Sending some data");
    }
}
