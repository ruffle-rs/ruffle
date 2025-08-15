use ruffle_core::context::UpdateContext;
use ruffle_core::external::{ExternalInterfaceProvider, Value as ExternalValue};
use url::Url;

pub struct DesktopExternalInterfaceProvider {
    pub spoof_url: Option<Url>,
}

fn is_location_href(code: &str) -> bool {
    matches!(
        code,
        "document.location.href" | "window.location.href" | "top.location.href"
    )
}

impl ExternalInterfaceProvider for DesktopExternalInterfaceProvider {
    fn call_method(
        &self,
        _context: &mut UpdateContext<'_>,
        name: &str,
        args: &[ExternalValue],
    ) -> ExternalValue {
        if let Some(ref url) = self.spoof_url {
            // Check for e.g. "window.location.href.toString"
            if let Some(name) = name.strip_suffix(".toString") {
                if is_location_href(name) {
                    return url.to_string().into();
                }
            }
        }

        if name == "eval" {
            if let Some(ref url) = self.spoof_url {
                if let [ExternalValue::String(ref code)] = args {
                    if is_location_href(code) {
                        return ExternalValue::String(url.to_string());
                    }
                }
            }

            tracing::warn!("Trying to call eval with ExternalInterface: {args:?}");
            return ExternalValue::Undefined;
        }

        tracing::warn!("Trying to call unknown ExternalInterface method: {name}");
        ExternalValue::Undefined
    }

    fn on_callback_available(&self, _name: &str) {}

    fn get_id(&self) -> Option<String> {
        None
    }
}
