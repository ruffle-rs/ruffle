pub trait ExternalInterfaceProvider {}

#[derive(Default)]
pub struct ExternalInterface {
    providers: Vec<Box<dyn ExternalInterfaceProvider>>,
}

impl ExternalInterface {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_provider(&mut self, provider: Box<dyn ExternalInterfaceProvider>) {
        self.providers.push(provider);
    }

    pub fn available(&self) -> bool {
        !self.providers.is_empty()
    }
}
