use crate::util::runner::TestLogBackend;
use async_channel::Receiver;
use ruffle_core::backend::log::LogBackend;
use ruffle_core::backend::navigator::{
    fetch_path, resolve_url_with_relative_base_path, ErrorResponse, NavigationMethod,
    NavigatorBackend, NullExecutor, NullSpawner, OwnedFuture, Request, SuccessResponse,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use ruffle_core::socket::{ConnectionState, SocketAction, SocketHandle};
use ruffle_socket_format::SocketEvent;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;
use url::{ParseError, Url};

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
    relative_base_path: PathBuf,
    socket_events: Option<Vec<SocketEvent>>,
    log: Option<TestLogBackend>,
}

impl TestNavigatorBackend {
    pub fn new(
        path: &Path,
        executor: &NullExecutor,
        socket_events: Option<Vec<SocketEvent>>,
        log: Option<TestLogBackend>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            spawner: executor.spawner(),
            relative_base_path: path.canonicalize()?,
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

    fn fetch(&self, request: Request) -> OwnedFuture<SuccessResponse, ErrorResponse> {
        if request.url().contains("?debug-success") {
            return Box::pin(async move {
                Ok(SuccessResponse {
                    url: request.url().to_string(),
                    body: b"Hello, World!".to_vec(),
                    status: 200,
                    redirected: false,
                })
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

        fetch_path(
            self,
            "TestNavigatorBackend",
            request.url(),
            Some(&self.relative_base_path),
        )
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
                                .send(SocketAction::Connect(handle, ConnectionState::Connected))
                                .expect("working channel send");

                for event in events {
                    match event {
                        SocketEvent::Disconnect => {
                            sender
                                .send(SocketAction::Close(handle))
                                .expect("working channel send");
                        },
                        SocketEvent::WaitForDisconnect => {
                            match receiver.recv().await {
                                Err(_) => break,
                                Ok(_) => panic!("Expected client to disconnect, data was sent instead"),
                            }
                        },
                        SocketEvent::Receive { expected } => {
                            match receiver.recv().await {
                                Ok(val) => {
                                    if expected != val {
                                        panic!("Received data did not match expected data\nExpected: {:?}\nActual: {:?}", expected, val);
                                    }
                                }
                                Err(_) => panic!("Expected client to send data, but connection was closed instead"),
                            }
                        },
                        SocketEvent::Send { payload } => {
                            sender.send(SocketAction::Data(handle, payload)).expect("working channel send");
                        }
                    }
                }

                Ok(())
            }));
        }
    }
}
