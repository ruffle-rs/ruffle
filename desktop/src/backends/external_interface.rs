use ruffle_core::external::{ExternalInterfaceMethod, ExternalInterfaceProvider};

#[derive(Default)]
pub struct DesktopExternalInterfaceProvider;

impl ExternalInterfaceProvider for DesktopExternalInterfaceProvider {
    fn get_method(&self, _name: &str) -> Option<Box<dyn ExternalInterfaceMethod>> {
        None
    }

    fn on_callback_available(&self, _name: &str) {}
}
