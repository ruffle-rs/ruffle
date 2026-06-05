/// Backend trait for cross-context LocalConnection transport.
///
/// This trait abstracts the transport mechanism used to send and receive
/// LocalConnection messages between different Ruffle player instances
/// (e.g., different browser tabs or iframes).
///
/// In Flash Player, LocalConnection used shared memory to allow SWF files
/// to communicate across different player instances on the same machine.
/// This trait enables similar functionality in Ruffle by allowing different
/// platform backends to provide cross-context message passing.
pub trait LocalConnectionBackend {
    /// Called when a LocalConnection.connect() is made in this player.
    /// The backend should register interest in messages for this connection name
    /// so that other contexts know a listener exists.
    fn register_listener(&mut self, connection_name: &str);

    /// Called when a LocalConnection.close() is made.
    /// The backend should unregister interest in messages for this connection name.
    fn unregister_listener(&mut self, connection_name: &str);

    /// Check whether a listener for the given connection name exists in another context.
    /// Used by connect() to enforce cross-tab uniqueness and by send() to determine
    /// the correct status event when no local listener exists.
    fn has_remote_listener(&self, connection_name: &str) -> bool;

    /// Broadcast a message to other contexts.
    /// Called from send() so that other Ruffle instances can receive it.
    ///
    /// `connection_name` is the fully-qualified connection name (including superdomain prefix).
    /// `method_name` is the method to invoke on the receiver.
    /// `amf_data` is the AMF-serialized arguments.
    fn send_message(&mut self, connection_name: &str, method_name: &str, amf_data: &[u8]);

    /// Poll for incoming messages from other contexts.
    /// Called once per frame during update_connections().
    /// Returns a Vec of received messages that should be delivered to local listeners.
    fn poll_incoming(&mut self) -> Vec<ExternalLocalConnectionMessage>;

    /// Provide a reference to the core Player.
    /// Used by asynchronous backends to instantly wake the Player upon receiving messages.
    fn set_player(&mut self, _player: std::sync::Weak<std::sync::Mutex<crate::Player>>) {}
}

/// A message received from another Ruffle context via the LocalConnection backend.
#[derive(Debug, Clone)]
pub struct ExternalLocalConnectionMessage {
    /// The fully-qualified connection name (e.g., "someDomain.com:myConnection").
    pub connection_name: String,
    /// The method name to invoke on the receiver.
    pub method_name: String,
    /// The AMF-serialized arguments.
    pub amf_data: Vec<u8>,
}

/// No-op backend for desktop, tests, and contexts where cross-context
/// LocalConnection is not supported.
#[derive(Default)]
pub struct NullLocalConnectionBackend;

impl LocalConnectionBackend for NullLocalConnectionBackend {
    fn register_listener(&mut self, _connection_name: &str) {}
    fn unregister_listener(&mut self, _connection_name: &str) {}
    fn has_remote_listener(&self, _connection_name: &str) -> bool {
        false
    }
    fn send_message(&mut self, _connection_name: &str, _method_name: &str, _amf_data: &[u8]) {}
    fn poll_incoming(&mut self) -> Vec<ExternalLocalConnectionMessage> {
        vec![]
    }
    fn set_player(&mut self, _player: std::sync::Weak<std::sync::Mutex<crate::Player>>) {}
}
