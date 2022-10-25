//! Navigator backend for web

use crate::custom_event::RuffleEvent;
use isahc::{
    config::RedirectPolicy, prelude::*, AsyncReadResponseExt, HttpClient, Request as IsahcRequest,
};
use ruffle_core::backend::navigator::{
    NavigationMethod, NavigatorBackend, OwnedFuture, Request, Response,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::rc::Rc;
use std::sync::mpsc::Sender;
use url::Url;
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

    upgrade_to_https: bool,
}

impl ExternalNavigatorBackend {
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
        let mut base_url = movie_url;

        // Force replace the last segment with empty. //

        base_url
            .path_segments_mut()
            .unwrap()
            .pop_if_empty()
            .pop()
            .push("");

        Self {
            channel,
            event_loop,
            client,
            base_url,
            upgrade_to_https,
        }
    }
}

impl NavigatorBackend for ExternalNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: String,
        _target: String,
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

        match webbrowser::open(processed_url.as_ref()) {
            Ok(_output) => {}
            Err(e) => log::error!("Could not open URL {}: {}", processed_url.as_str(), e),
        };
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error> {
        // TODO: honor sandbox type (local-with-filesystem, local-with-network, remote, ...)
        let full_url = match self.base_url.join(request.url()) {
            Ok(url) => url,
            Err(e) => {
                let msg = format!("Invalid URL {}: {}", request.url(), e);
                return Box::pin(async move { Err(Error::FetchError(msg)) });
            }
        };

        let processed_url = self.pre_process_url(full_url);

        let client = self.client.clone();

        match processed_url.scheme() {
            "file" => Box::pin(async move {
                let path = processed_url.to_file_path().unwrap_or_default();

                let url = processed_url.into();

                let body = std::fs::read(&path).or_else(|e| {
                    if cfg!(feature = "sandbox") {
                        use rfd::{FileDialog, MessageButtons, MessageDialog, MessageLevel};
                        use std::io::ErrorKind;

                        if e.kind() == ErrorKind::PermissionDenied {
                            let attempt_sandbox_open = MessageDialog::new()
                                .set_level(MessageLevel::Warning)
                                .set_description(&format!("The current movie is attempting to read files stored in {}.\n\nTo allow it to do so, click Yes, and then Open to grant read access to that directory.\n\nOtherwise, click No to deny access.", path.parent().unwrap().to_string_lossy()))
                                .set_buttons(MessageButtons::YesNo)
                                .show();

                            if attempt_sandbox_open {
                                FileDialog::new().set_directory(&path).pick_folder();

                                return std::fs::read(&path);
                            }
                        }
                    }

                    Err(e)
                }).map_err(|e| Error::FetchError(e.to_string()))?;

                Ok(Response { url, body })
            }),
            _ => Box::pin(async move {
                let client =
                    client.ok_or_else(|| Error::FetchError("Network unavailable".to_string()))?;

                let isahc_request = match request.method() {
                    NavigationMethod::Get => IsahcRequest::get(processed_url.to_string()),
                    NavigationMethod::Post => IsahcRequest::post(processed_url.to_string()),
                };

                let (body_data, _) = request.body().clone().unwrap_or_default();
                let body = isahc_request
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

                let url = if let Some(uri) = response.effective_uri() {
                    uri.to_string()
                } else {
                    processed_url.into()
                };

                let mut body = vec![];
                response
                    .copy_to(&mut body)
                    .await
                    .map_err(|e| Error::FetchError(e.to_string()))?;

                Ok(Response { url, body })
            }),
        }
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.channel.send(future).expect("working channel send");

        if self.event_loop.send_event(RuffleEvent::TaskPoll).is_err() {
            log::warn!(
                "A task was queued on an event loop that has already ended. It will not be polled."
            );
        }
    }

    fn pre_process_url(&self, mut url: Url) -> Url {
        if self.upgrade_to_https && url.scheme() == "http" && url.set_scheme("https").is_err() {
            log::error!("Url::set_scheme failed on: {}", url);
        }
        url
    }
}
