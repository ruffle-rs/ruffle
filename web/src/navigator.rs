//! Navigator backend for web

use ruffle_core::backend::navigator::{Error, NavigationMethod, NavigatorBackend};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

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

    fn fetch(&self, _url: String) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, Error>>>> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn spawn_future(&mut self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        spawn_local(future)
    }
}
