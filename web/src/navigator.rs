//! Navigator backend for web

use js_sys::{Array, ArrayBuffer, Uint8Array};
use ruffle_core::backend::navigator::{
    Error, NavigationMethod, NavigatorBackend, OwnedFuture, RequestOptions,
};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{window, Blob, BlobPropertyBag, Request, RequestInit, Response};

pub struct WebNavigatorBackend {}

impl WebNavigatorBackend {
    pub fn new() -> Self {
        WebNavigatorBackend {}
    }
}

impl NavigatorBackend for WebNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: String,
        window_spec: Option<String>,
        vars_method: Option<(NavigationMethod, HashMap<String, String>)>,
    ) {
        if let Some(window) = window() {
            //TODO: Should we return a result for failed opens? Does Flash care?
            #[allow(unused_must_use)]
            match (vars_method, window_spec) {
                (Some((navmethod, formvars)), window_spec) => {
                    let document = match window.document() {
                        Some(document) => document,
                        None => return,
                    };

                    let form = document
                        .create_element("form")
                        .unwrap()
                        .dyn_into::<web_sys::HtmlFormElement>()
                        .unwrap();

                    form.set_attribute(
                        "method",
                        match navmethod {
                            NavigationMethod::GET => "get",
                            NavigationMethod::POST => "post",
                        },
                    );

                    form.set_attribute("action", &url);

                    if let Some(target) = window_spec {
                        form.set_attribute("target", &target);
                    }

                    for (k, v) in formvars.iter() {
                        let hidden = document.create_element("hidden").unwrap();

                        hidden.set_attribute("type", "hidden");
                        hidden.set_attribute("name", k);
                        hidden.set_attribute("value", v);

                        form.append_child(&hidden);
                    }

                    document.body().unwrap().append_child(&form);
                    form.submit();
                }
                (_, Some(ref window_name)) if window_name != "" => {
                    window.open_with_url_and_target(&url, window_name);
                }
                _ => {
                    window.location().assign(&url);
                }
            };
        }
    }

    fn fetch(&self, url: String, options: RequestOptions) -> OwnedFuture<Vec<u8>, Error> {
        Box::pin(async move {
            let mut init = RequestInit::new();

            init.method(match options.method() {
                NavigationMethod::GET => "GET",
                NavigationMethod::POST => "POST",
            });

            if let Some((data, mime)) = options.body() {
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

            let request = Request::new_with_str_and_init(&url, &init).unwrap();

            let window = web_sys::window().unwrap();
            let fetchval = JsFuture::from(window.fetch_with_request(&request)).await;
            if fetchval.is_err() {
                return Err("Could not fetch, got JS Error".into());
            }

            let resp: Response = fetchval.unwrap().dyn_into().unwrap();
            let data: ArrayBuffer = JsFuture::from(resp.array_buffer().unwrap())
                .await
                .unwrap()
                .dyn_into()
                .unwrap();
            let jsarray = Uint8Array::new(&data);
            let mut rust_array = vec![0; jsarray.length() as usize];
            jsarray.copy_to(&mut rust_array);

            Ok(rust_array)
        })
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        spawn_local(async move {
            if let Err(e) = future.await {
                log::error!("Asynchronous error occured: {}", e);
            }
        })
    }
}
