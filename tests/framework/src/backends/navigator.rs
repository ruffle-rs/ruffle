use crate::backends::TestLogBackend;
use crate::util::read_bytes;
use async_channel::{Receiver, Sender};
use percent_encoding::percent_decode_str;
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{
    async_return, create_fetch_error, ErrorResponse, NavigationMethod, NavigatorBackend,
    NullExecutor, NullSpawner, OwnedFuture, Request, SuccessResponse,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use ruffle_core::socket::{ConnectionState, SocketAction, SocketHandle};
use ruffle_core::swf::Encoding;
use ruffle_socket_format::SocketEvent;
use std::borrow::Cow;
use std::time::Duration;
use url::{ParseError, Url};
use vfs::VfsPath;

struct TestResponse {
    url: String,
    body: Vec<u8>,
    chunk_gotten: bool,
    status: u16,
    redirected: bool,
}

impl SuccessResponse for TestResponse {
    fn url(&self) -> Cow<str> {
        Cow::Borrowed(&self.url)
    }

    fn body(self: Box<Self>) -> OwnedFuture<Vec<u8>, Error> {
        Box::pin(async move { Ok(self.body) })
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
        if !self.chunk_gotten {
            self.chunk_gotten = true;
            let body = self.body.clone();
            Box::pin(async move { Ok(Some(body)) })
        } else {
            Box::pin(async move { Ok(None) })
        }
    }

    fn expected_length(&self) -> Result<Option<u64>, Error> {
        Ok(Some(self.body.len() as u64))
    }
}

/// A `NavigatorBackend` used by tests that supports logging fetch requests.
///
/// This can be used by tests that fetch data to verify that the request is correct.
///
/// Attempting to fetch URLs containing the following "hints" will cause a simulated response:
/// * "?debug-success" -> Simulates a successful fetch, with body "Hello, World!"
/// * "?debug-error-statuscode" -> Simulates a failed fetch due to a unsuccessful status
/// * "?debug-error-dns" -> Simulates a failed fetch due to a dns resolution error
///
/// These are formatted as query params, rather than domains/whole URLs, so that real/real-invalid
/// URLs can be used in Flash Player when writing tests
pub struct TestNavigatorBackend {
    spawner: NullSpawner,
    relative_base_path: VfsPath,
    socket_events: Option<Vec<SocketEvent>>,
    log: Option<TestLogBackend>,
}

impl TestNavigatorBackend {
    pub fn new(
        path: VfsPath,
        executor: &NullExecutor,
        socket_events: Option<Vec<SocketEvent>>,
        log: Option<TestLogBackend>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            spawner: executor.spawner(),
            relative_base_path: path,
            socket_events,
            log,
        })
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

    fn fetch(&self, request: Request) -> OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse> {
        if request.url().contains("?debug-success") {
            return Box::pin(async move {
                let response: Box<dyn SuccessResponse> = Box::new(TestResponse {
                    url: request.url().to_string(),
                    body: b"Hello, World!".to_vec(),
                    chunk_gotten: false,
                    status: 200,
                    redirected: false,
                });

                Ok(response)
            });
        }

        if request.url().contains("?debug-error-statuscode") {
            return Box::pin(async move {
                Err(ErrorResponse {
                    url: request.url().to_string(),
                    error: Error::HttpNotOk(request.url().to_string(), 0, false, 0),
                })
            });
        }

        if request.url().contains("?debug-error-dns") {
            return Box::pin(async move {
                Err(ErrorResponse {
                    url: request.url().to_string(),
                    error: Error::InvalidDomain(request.url().to_string()),
                })
            });
        }

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
                    log.avm_trace(&format!("  Body: {:02X?}", body));
                }
            }
        }

        let url = match self.resolve_url(request.url()) {
            Ok(url) => url,
            Err(e) => return async_return(create_fetch_error(request.url(), e)),
        };

        let base_path = self.relative_base_path.clone();

        Box::pin(async move {
            let path = if url.scheme() == "file" {
                // Flash supports query parameters with local urls.
                // SwfMovie takes care of exposing those to ActionScript -
                // when we actually load a filesystem url, strip them out.
                let mut filesystem_url = url.clone();
                filesystem_url.set_query(None);

                base_path
                    .join(
                        percent_decode_str(filesystem_url.path())
                            .decode_utf8()
                            .map_err(|e| ErrorResponse {
                                url: url.to_string(),
                                error: Error::FetchError(e.to_string()),
                            })?,
                    )
                    .map_err(|e| ErrorResponse {
                        url: url.to_string(),
                        error: Error::FetchError(e.to_string()),
                    })?
            } else {
                // Turn a url like https://localhost/foo/bar to {base_path}/localhost/foo/bar
                let mut path = base_path.clone();
                if let Some(host) = url.host_str() {
                    path = path.join(host).map_err(|e| ErrorResponse {
                        url: url.to_string(),
                        error: Error::FetchError(e.to_string()),
                    })?;
                }
                if let Some(remaining) = url.path().strip_prefix('/') {
                    path = path
                        .join(percent_decode_str(remaining).decode_utf8().map_err(|e| {
                            ErrorResponse {
                                url: url.to_string(),
                                error: Error::FetchError(e.to_string()),
                            }
                        })?)
                        .map_err(|e| ErrorResponse {
                            url: url.to_string(),
                            error: Error::FetchError(e.to_string()),
                        })?;
                }
                path
            };

            let body = read_bytes(&path).map_err(|error| ErrorResponse {
                url: url.to_string(),
                error: Error::FetchError(error.to_string()),
            })?;

            let response: Box<dyn SuccessResponse> = Box::new(TestResponse {
                url: url.to_string(),
                body,
                chunk_gotten: false,
                status: 0,
                redirected: false,
            });

            Ok(response)
        })
    }

    fn resolve_url(&self, url: &str) -> Result<Url, ParseError> {
        let mut base_url = Url::parse("file:///")?;

        // Make sure we have a trailing slash, so that joining a request url like 'data.txt'
        // gets appended, rather than replacing the last component.
        base_url.path_segments_mut().unwrap().push("");
        if let Ok(parsed_url) = base_url.join(url) {
            Ok(self.pre_process_url(parsed_url))
        } else {
            match Url::parse(url) {
                Ok(parsed_url) => Ok(self.pre_process_url(parsed_url)),
                Err(error) => Err(error),
            }
        }
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.spawner.spawn_local(future);
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }

    fn connect_socket(
        &mut self,
        host: String,
        port: u16,
        _timeout: Duration,
        handle: SocketHandle,
        receiver: Receiver<Vec<u8>>,
        sender: Sender<SocketAction>,
    ) {
        if let Some(log) = &self.log {
            log.avm_trace("Navigator::connect_socket");
            log.avm_trace(&format!("    Host: {}; Port: {}", host, port));
        }

        if let Some(events) = self.socket_events.clone() {
            self.spawn_future(Box::pin(async move {
                sender
                    .try_send(SocketAction::Connect(handle, ConnectionState::Connected))
                    .expect("working channel send");

                for event in events {
                    match event {
                        SocketEvent::Disconnect => {
                            sender
                                .try_send(SocketAction::Close(handle))
                                .expect("working channel send");
                        }
                        SocketEvent::WaitForDisconnect => {
                            match receiver.recv().await {
                                Err(_) => break,
                                Ok(_) => panic!("Expected client to disconnect, data was sent instead"),
                            }
                        }
                        SocketEvent::Receive { expected } => {
                            match receiver.recv().await {
                                Ok(val) => {
                                    if expected != val {
                                        panic!("Received data did not match expected data\nExpected: {:?}\nActual: {:?}", expected, val);
                                    }
                                }
                                Err(_) => panic!("Expected client to send data, but connection was closed instead"),
                            }
                        }
                        SocketEvent::Send { payload } => {
                            sender.try_send(SocketAction::Data(handle, payload)).expect("working channel send");
                        }
                    }
                }

                Ok(())
            }));
        }
    }
}
