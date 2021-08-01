//! Navigator backend for web

use crate::custom_event::RuffleEvent;
use isahc::{config::RedirectPolicy, prelude::*, AsyncReadResponseExt, HttpClient, Request};
use ruffle_core::backend::navigator::{
    NavigationMethod, NavigatorBackend, OwnedFuture, RequestOptions,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::borrow::Cow;
use std::convert::Infallible;
use std::fs;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use url::Url;
use winit::event_loop::EventLoopProxy;

pub type BackgroundTaskHandle = (Waker, Arc<Mutex<BackgroundFutureState>>);

/// The internal shared state of a single BackgroundFuture.
pub struct BackgroundFutureState {
    /// Whether or not the event loop has resolved.
    pub event_loop_resolved: bool,

    /// What to send wakers into.
    waker_sender: Sender<BackgroundTaskHandle>,
}

/// A future that suspends it's awaited task until the event loop clears out.
pub struct BackgroundFuture(Arc<Mutex<BackgroundFutureState>>);

impl BackgroundFuture {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(waker_sender: Sender<BackgroundTaskHandle>) -> OwnedFuture<(), Infallible> {
        Box::pin(Self(Arc::new(Mutex::new(BackgroundFutureState {
            event_loop_resolved: false,
            waker_sender,
        }))))
    }
}

impl Future for BackgroundFuture {
    type Output = Result<(), Infallible>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let guard = self.0.lock().unwrap();
        if guard.event_loop_resolved {
            Poll::Ready(Ok(()))
        } else {
            guard
                .waker_sender
                .send((cx.waker().clone(), self.0.clone()))
                .unwrap();

            Poll::Pending
        }
    }
}

/// Implementation of `NavigatorBackend` for non-web environments that can call
/// out to a web browser.
pub struct ExternalNavigatorBackend {
    /// Sink for tasks sent to us through `spawn_future`.
    future_channel: Sender<OwnedFuture<(), Error>>,

    /// Sink for suspended task notifications.
    suspend_channel: Sender<BackgroundTaskHandle>,

    /// Event sink to trigger a new task poll.
    event_loop: EventLoopProxy<RuffleEvent>,

    /// The url to use for all relative fetches.
    movie_url: Url,

    /// The time that the SWF was launched.
    start_time: Instant,

    // Client to use for network requests
    client: Option<Rc<HttpClient>>,

    upgrade_to_https: bool,
}

impl ExternalNavigatorBackend {
    #[allow(dead_code)]
    /// Construct a navigator backend with fetch and async capability.
    pub fn new(
        movie_url: Url,
        future_channel: Sender<OwnedFuture<(), Error>>,
        suspend_channel: Sender<BackgroundTaskHandle>,
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
            future_channel,
            suspend_channel,
            event_loop,
            client,
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
        self.future_channel
            .send(future)
            .expect("working channel send");

        if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
            log::warn!(
                "A task was queued on an event loop that has already ended. It will not be polled."
            );
        }
    }

    fn background(&self) -> OwnedFuture<(), Infallible> {
        BackgroundFuture::new(self.suspend_channel.clone())
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
