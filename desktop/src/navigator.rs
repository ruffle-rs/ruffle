//! Navigator backend for web

use crate::custom_event::RuffleEvent;
use isahc::{config::RedirectPolicy, prelude::*, AsyncReadResponseExt, HttpClient, Request};
use ruffle_core::backend::navigator::{
    ConnectOptions, ConnectionEvent, NavigationMethod, NavigatorBackend, OwnedFuture,
    RequestOptions,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::io::{Read, Write};
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{fs, io};
use url::Url;
use winit::event_loop::EventLoopProxy;

enum SocketState {
    TryConnect(ConnectOptions),
    Connected {
        stream: std::net::TcpStream,
        outgoing: VecDeque<Vec<u8>>,
        queue: VecDeque<u8>,
    },
}

/// Implementation of `NavigatorBackend` for non-web environments that can call
/// out to a web browser.
pub struct ExternalNavigatorBackend {
    /// Sink for tasks sent to us through `spawn_future`.
    channel: Sender<OwnedFuture<(), Error>>,

    /// Event sink to trigger a new task poll.
    event_loop: EventLoopProxy<RuffleEvent>,

    /// The url to use for all relative fetches.
    movie_url: Url,

    /// The time that the SWF was launched.
    start_time: Instant,

    // Client to use for network requests
    client: Option<Rc<HttpClient>>,

    sockets: HashMap<u64, SocketState>,

    upgrade_to_https: bool,
}

impl ExternalNavigatorBackend {
    #[allow(dead_code)]
    /// Construct a navigator backend with fetch and async capability.
    pub fn new(
        movie_url: Url,
        channel: Sender<OwnedFuture<(), Error>>,
        event_loop: EventLoopProxy<RuffleEvent>,
        proxy: Option<Url>,
        upgrade_to_https: bool,
    ) -> Self {
        let proxy = proxy.and_then(|url| url.as_str().parse().ok());
        let builder = HttpClient::builder()
            .proxy(proxy)
            .redirect_policy(RedirectPolicy::Follow);

        let client = builder.build().ok().map(Rc::new);

        Self {
            channel,
            event_loop,
            client,
            sockets: HashMap::new(),
            movie_url,
            start_time: Instant::now(),
            upgrade_to_https,
        }
    }
}

impl NavigatorBackend for ExternalNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: String,
        _window_spec: Option<String>,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        //TODO: Should we return a result for failed opens? Does Flash care?

        //NOTE: Flash desktop players / projectors ignore the window parameter,
        //      unless it's a `_layer`, and we shouldn't handle that anyway.
        let mut parsed_url = match Url::parse(&url) {
            Ok(parsed_url) => parsed_url,
            Err(e) => {
                log::error!(
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

        let processed_url = self.pre_process_url(modified_url);

        match webbrowser::open(&processed_url.to_string()) {
            Ok(_output) => {}
            Err(e) => log::error!("Could not open URL {}: {}", processed_url.as_str(), e),
        };
    }

    fn fetch(&self, url: &str, options: RequestOptions) -> OwnedFuture<Vec<u8>, Error> {
        // TODO: honor sandbox type (local-with-filesystem, local-with-network, remote, ...)
        let full_url = match self.movie_url.clone().join(url) {
            Ok(url) => url,
            Err(e) => {
                let msg = format!("Invalid URL {}: {}", url, e);
                return Box::pin(async move { Err(Error::FetchError(msg)) });
            }
        };

        let processed_url = self.pre_process_url(full_url);

        let client = self.client.clone();

        match processed_url.scheme() {
            "file" => Box::pin(async move {
                fs::read(processed_url.to_file_path().unwrap_or_default())
                    .map_err(Error::NetworkError)
            }),
            _ => Box::pin(async move {
                let client = client.ok_or(Error::NetworkUnavailable)?;

                let request = match options.method() {
                    NavigationMethod::Get => Request::get(processed_url.to_string()),
                    NavigationMethod::Post => Request::post(processed_url.to_string()),
                };

                let (body_data, _) = options.body().clone().unwrap_or_default();
                let body = request
                    .body(body_data)
                    .map_err(|e| Error::FetchError(e.to_string()))?;

                let mut response = client
                    .send_async(body)
                    .await
                    .map_err(|e| Error::FetchError(e.to_string()))?;

                if !response.status().is_success() {
                    return Err(Error::FetchError(format!(
                        "HTTP status is not ok, got {}",
                        response.status()
                    )));
                }

                let mut buffer = vec![];
                response
                    .copy_to(&mut buffer)
                    .await
                    .map_err(|e| Error::FetchError(e.to_string()))?;
                Ok(buffer)
            }),
        }
    }

    fn time_since_launch(&mut self) -> Duration {
        Instant::now().duration_since(self.start_time)
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.channel.send(future).expect("working channel send");

        if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
            log::warn!(
                "A task was queued on an event loop that has already ended. It will not be polled."
            );
        }
    }

    fn xmlsocket_connect(&mut self, socket_id: u64, c: ConnectOptions) {
        self.sockets.insert(socket_id, SocketState::TryConnect(c));
    }

    fn xmlsocket_send(&mut self, socket_id: &u64, data: Vec<u8>) {
        if let Some(SocketState::Connected { outgoing, .. }) = self.sockets.get_mut(socket_id) {
            outgoing.push_back(data);
        }
    }

    fn xmlsocket_update(
        &mut self,
        socket_id: &u64,
        buffer: &mut [u8; 1024],
    ) -> Vec<ConnectionEvent> {
        let mut events = Vec::<ConnectionEvent>::new();
        if let Some(st) = self.sockets.get_mut(socket_id) {
            match st {
                SocketState::TryConnect(c) => {
                    let r = std::net::TcpStream::connect(format!("{}:{}", c.host, c.port))
                        .and_then(|s| {
                            s.set_nonblocking(true)?;
                            Ok(s)
                        })
                        .and_then(|s| {
                            s.set_nodelay(true)?;
                            Ok(s)
                        });
                    match r {
                        Ok(stream) => {
                            *st = SocketState::Connected {
                                stream,
                                queue: VecDeque::new(),
                                outgoing: VecDeque::new(),
                            };
                            events.push(ConnectionEvent::ConnectionResult(true));
                        }
                        Err(_) => {
                            events.push(ConnectionEvent::ConnectionResult(false));
                            self.sockets.remove(socket_id);
                        }
                    }
                }
                SocketState::Connected {
                    stream,
                    queue,
                    outgoing,
                } => {
                    let _ = stream.write_all(
                        &outgoing.iter().fold(Vec::<u8>::new(), |mut s, o| {
                            s.extend(o);
                            // every message sent to the server is terminated by a null byte
                            s.push(0);
                            s
                        })[..],
                    );
                    outgoing.clear();
                    match stream.read(buffer) {
                        Ok(l) if l > 0 => {
                            queue.extend(&buffer[0..l]);
                            let mut i = 0;
                            while i < queue.len() {
                                if i > 0 && queue[i] == 0x00 {
                                    let message = queue.drain(0..i).collect();
                                    // remove the null byte
                                    queue.pop_front();
                                    events.push(ConnectionEvent::Data(message));
                                    // restart to the start
                                    i = 0;
                                }
                                i += 1;
                            }
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                        Err(_) | Ok(_) => {
                            self.sockets.remove(socket_id);
                            events.push(ConnectionEvent::Closed);
                        }
                    }
                }
            }
        }
        events
    }

    fn resolve_relative_url<'a>(&mut self, url: &'a str) -> Cow<'a, str> {
        let relative = self.movie_url.join(url);
        if let Ok(relative) = relative {
            String::from(relative).into()
        } else {
            url.into()
        }
    }

    fn pre_process_url(&self, mut url: Url) -> Url {
        if self.upgrade_to_https && url.scheme() == "http" && url.set_scheme("https").is_err() {
            log::error!("Url::set_scheme failed on: {}", url);
        }
        url
    }
}
