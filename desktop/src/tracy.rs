use tracing::Metadata;
use tracing_subscriber::fmt::format::DefaultFields;
use tracing_tracy::client::register_demangler;
use tracing_tracy::Config;

#[derive(Default)]
pub struct RuffleTracyConfig(DefaultFields);

register_demangler!();

impl Config for RuffleTracyConfig {
    type Formatter = DefaultFields;

    fn formatter(&self) -> &Self::Formatter {
        &self.0
    }

    fn stack_depth(&self, _metadata: &Metadata<'_>) -> u16 {
        // How much, if any, of the stack trace to capture for each event
        // Obviously, this adds overhead
        0
    }
}
