//! Browser-related platform functions

use std::collections::HashMap;
use swf::avm1::types::SendVarsMethod;

/// Enumerates all possible navigation methods.
pub enum NavigationMethod {
    /// Indicates that navigation should generate a GET request.
    GET,

    /// Indicates that navigation should generate a POST request.
    POST
}

impl NavigationMethod {
    /// Convert an SWF method enum into a NavigationMethod.
    pub fn from_send_vars_method(s: SendVarsMethod) -> Option<Self> {
        match s {
            SendVarsMethod::None => None,
            SendVarsMethod::Get => Some(Self::GET),
            SendVarsMethod::Post => Some(Self::POST),
        }
    }
}

/// A backend interacting with a browser environment.
pub trait NavigatorBackend {
    /// Cause a browser navigation to a given URL.
    /// 
    /// The URL given may be any URL scheme a browser can support. This may not
    /// be meaningful for all environments: for example, `javascript:` URLs may
    /// not be executable in a desktop context.
    /// 
    /// The `window` parameter, if provided, should be treated identically to
    /// the `window` parameter on an HTML `<a>nchor` tag.
    /// 
    /// This function may be used to send variables to an eligible target. If
    /// desired, the `vars_method` will be specified with a suitable
    /// `NavigationMethod` and a key-value representation of the variables to
    /// be sent. What the backend needs to do depends on the `NavigationMethod`:
    /// 
    /// * `GET` - Variables are appended onto the query parameters of the given
    ///   URL.
    /// * `POST` - Variables are sent as form data in a POST request, as if the
    ///   user had filled out and submitted an HTML form.
    /// 
    /// Flash Player implemented sandboxing to prevent certain kinds of XSS
    /// attacks. The `NavigatorBackend` is not responsible for enforcing this
    /// sandbox.
    fn navigate_to_url(&self, url: String, window: Option<String>, vars_method: Option<(NavigationMethod, HashMap<String, String>)>);
}

/// A null implementation for platforms that do not live in a web browser.
pub struct NullNavigatorBackend {
}

impl NullNavigatorBackend {
    pub fn new() -> Self {
        NullNavigatorBackend {

        }
    }
}

impl Default for NullNavigatorBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigatorBackend for NullNavigatorBackend {
    fn navigate_to_url(&self, _url: String, _window: Option<String>, _vars_method: Option<(NavigationMethod, HashMap<String, String>)>) {

    }
}