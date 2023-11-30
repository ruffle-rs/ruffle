use ruffle_core::context::UpdateContext;
use ruffle_core::external::{
    ExternalInterfaceMethod, ExternalInterfaceProvider, Value as ExternalValue,
};
use url::Url;

pub struct DesktopExternalInterfaceProvider {
    pub spoof_url: Option<Url>,
}

struct FakeWindowLocationHrefToString(Url);

impl ExternalInterfaceMethod for FakeWindowLocationHrefToString {
    fn call(&self, _context: &mut UpdateContext<'_, '_>, _args: &[ExternalValue]) -> ExternalValue {
        ExternalValue::String(self.0.to_string())
    }
}

impl ExternalInterfaceProvider for DesktopExternalInterfaceProvider {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        if let Some(ref url) = self.spoof_url {
            if name == "window.location.href.toString" || name == "top.location.href.toString" {
                return Some(Box::new(FakeWindowLocationHrefToString(url.clone())));
            }
        }

        tracing::warn!("Trying to call unknown ExternalInterface method: {name}");
        None
    }

    fn on_callback_available(&self, _name: &str) {}
}
