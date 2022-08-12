//! Navigator backend for web
use js_sys::{Array, ArrayBuffer, Uint8Array};
use ruffle_core::backend::navigator::{
    NavigationMethod, NavigatorBackend, OwnedFuture, Request, Response,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use std::borrow::Cow;
use url::Url;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{
    window, Blob, BlobPropertyBag, Request as WebRequest, RequestInit, Response as WebResponse,
};

pub struct WebNavigatorBackend {
    allow_script_access: bool,
    upgrade_to_https: bool,
    base_url: Option<Url>,
}

impl WebNavigatorBackend {
    pub fn new(
        allow_script_access: bool,
        upgrade_to_https: bool,
        base_url: Option<String>,
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
            log::error!("Could not get base URL for base directory inference.");
        }

        Self {
            allow_script_access,
            upgrade_to_https,
            base_url,
        }
    }

    fn resolve_url<'a>(&self, url: &'a str) -> Cow<'a, str> {
        if let Some(base_url) = &self.base_url {
            if let Ok(url) = base_url.join(url) {
                return self.pre_process_url(url).to_string().into();
            }
        }

        url.into()
    }
}

impl NavigatorBackend for WebNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: String,
        target: String,
        vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        // If the URL is empty, ignore the request.
        if url.is_empty() {
            return;
        }

        let url = self.resolve_url(&url);

        // If `allowScriptAccess` is disabled, reject the `javascript:` scheme.
        if let Ok(url) = Url::parse(&url) {
            if !self.allow_script_access && url.scheme() == "javascript" {
                log::warn!("SWF tried to run a script, but script access is not allowed");
                return;
            }
        }

        // TODO: Should we return a result for failed opens? Does Flash care?
        let window = window().expect("window()");
        match vars_method {
            Some((navmethod, formvars)) => {
                let document = window.document().expect("document()");
                let body = match document.body() {
                    Some(body) => body,
                    None => return,
                };

                let form = document
                    .create_element("form")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlFormElement>()
                    .unwrap();

                let _ = form.set_attribute(
                    "method",
                    match navmethod {
                        NavigationMethod::Get => "get",
                        NavigationMethod::Post => "post",
                    },
                );

                let _ = form.set_attribute("action", &url);

                if !target.is_empty() {
                    let _ = form.set_attribute("target", &target);
                }

                for (k, v) in formvars.iter() {
                    let hidden = document.create_element("input").unwrap();

                    let _ = hidden.set_attribute("type", "hidden");
                    let _ = hidden.set_attribute("name", k);
                    let _ = hidden.set_attribute("value", v);

                    let _ = form.append_child(&hidden);
                }

                let _ = body.append_child(&form);
                let _ = form.submit();
            }
            None => {
                if target.is_empty() {
                    let _ = window.location().assign(&url);
                } else {
                    let _ = window.open_with_url_and_target(&url, &target);
                }
            }
        };
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Response, Error> {
        let url = self.resolve_url(request.url()).into_owned();

        Box::pin(async move {
            let mut init = RequestInit::new();

            init.method(match request.method() {
                NavigationMethod::Get => "GET",
                NavigationMethod::Post => "POST",
            });

            if let Some((data, mime)) = request.body() {
                let arraydata = ArrayBuffer::new(data.len() as u32);
                let u8data = Uint8Array::new(&arraydata);

                for (i, byte) in data.iter().enumerate() {
                    u8data.fill(*byte, i as u32, i as u32 + 1);
                }

                let blobparts = Array::new();
                blobparts.push(&arraydata);

                let mut blobprops = BlobPropertyBag::new();
                blobprops.type_(mime);

                let datablob =
                    Blob::new_with_buffer_source_sequence_and_options(&blobparts, &blobprops)
                        .unwrap()
                        .dyn_into()
                        .unwrap();

                init.body(Some(&datablob));
            }

            let request = WebRequest::new_with_str_and_init(&url, &init)
                .map_err(|_| Error::FetchError(format!("Unable to create request for {}", url)))?;

            let window = web_sys::window().expect("window()");
            let fetchval = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|_| Error::FetchError("Got JS error".to_string()))?;

            let response: WebResponse = fetchval.dyn_into().unwrap();
            if !response.ok() {
                return Err(Error::FetchError(format!(
                    "HTTP status is not ok, got {}",
                    response.status_text()
                )));
            }

            let url = response.url();

            let body: ArrayBuffer = JsFuture::from(response.array_buffer().unwrap())
                .await
                .map_err(|_| {
                    Error::FetchError("Could not allocate array buffer for response".to_string())
                })?
                .dyn_into()
                .unwrap();
            let body = Uint8Array::new(&body).to_vec();

            Ok(Response { url, body })
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        spawn_local(async move {
            if let Err(e) = future.await {
                log::error!("Asynchronous error occurred: {}", e);
            }
        })
    }

    fn pre_process_url(&self, mut url: Url) -> Url {
        if self.upgrade_to_https && url.scheme() == "http" && url.set_scheme("https").is_err() {
            log::error!("Url::set_scheme failed on: {}", url);
        }
        url
    }
}
