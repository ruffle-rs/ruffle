use ruffle_core::context::UpdateContext;
use ruffle_core::external::{
    ExternalInterfaceMethod, ExternalInterfaceProvider, Value as ExternalValue,
};
use url::Url;

pub struct DesktopExternalInterfaceProvider {
    pub spoof_url: Option<Url>,
}

struct FakeLocationHrefToString(Url);

impl ExternalInterfaceMethod for FakeLocationHrefToString {
    fn call(&self, _context: &mut UpdateContext<'_>, _args: &[ExternalValue]) -> ExternalValue {
        ExternalValue::String(self.0.to_string())
    }
}

fn is_location_href(code: &str) -> bool {
    matches!(
        code,
        "document.location.href" | "window.location.href" | "top.location.href"
    )
}

struct FakeEval(Option<Url>);

impl ExternalInterfaceMethod for FakeEval {
    fn call(&self, _context: &mut UpdateContext<'_>, args: &[ExternalValue]) -> ExternalValue {
        if let Some(ref url) = self.0 {
            if let [ExternalValue::String(ref code)] = args {
                if is_location_href(code) {
                    return ExternalValue::String(url.to_string());
                }
            }
        }

        tracing::warn!("Trying to call eval with ExternalInterface: {args:?}");
        ExternalValue::Undefined
    }
}

impl ExternalInterfaceProvider for DesktopExternalInterfaceProvider {
    fn get_method(&self, name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        if let Some(ref url) = self.spoof_url {
            // Check for e.g. "window.location.href.toString"
            if let Some(name) = name.strip_suffix(".toString") {
                if is_location_href(name) {
                    return Some(Box::new(FakeLocationHrefToString(url.clone())));
                }
            }
        }

        if name == "eval" {
            return Some(Box::new(FakeEval(self.spoof_url.clone())));
        }

        tracing::warn!("Trying to call unknown ExternalInterface method: {name}");
        None
    }

    fn on_callback_available(&self, _name: &str) {}

    fn get_id(&self) -> Option<String> {
        None
    }
}
