use crate::{
    avm1::{Activation, ActivationIdentifier, ExecutionReason, Object, TObject},
    backend::navigator::NavigatorBackend,
    context::UpdateContext,
    string::AvmString,
};
use encoding_rs::UTF_8;
use gc_arena::{Collect, CollectionContext};
use generational_arena::{Arena, Index};
use std::collections::VecDeque;

/// XMLSocket backend implementation
///
/// When `XMLSocket.connect()` is called, the backend may provide a
/// [XmlSocketConnection] instance, that will be polled every tick.
/// Methods must not block.
///
/// For performances reasons, only 1 message will be polled per tick per connection
/// and at most 1 message will be sent per tick per connection.
///
/// When the XMLSocket gets closed, the underlying [XmlSocketConnection]
/// ise dropped, so simply implement [Drop] to handle cleanup stuff.
///
/// See [XmlSocketConnection::is_connected] for details about how
/// Ruffle infer connection state, [XmlSocketConnection::send] for details
/// about forwarding messages, [XmlSocketConnection::poll] for details
/// about polling incoming messages.
pub trait XmlSocketConnection {
    /// Return socket connection state
    ///
    /// Possibles values:
    ///  * `None`: connecting ([XmlSocketConnection::send] and
    ///            [XmlSocketConnection::poll] not yet called)
    ///  * `Some(true)`: connected ([XmlSocketConnection::send] and
    ///                 [XmlSocketConnection::poll] got called)
    ///  * `Some(false)`: disconnected, or connection refused socket wasn't connected
    ///                   (called only after [XmlSocketConnection::poll] return `None`;
    ///                    [XmlSocketConnection::send] and [XmlSocketConnection::poll]
    ///                    won't be called after `Some(false)` is returned)
    ///
    /// Returning `None` after `Some(true)` was returned
    /// will be considered as `Some(false)` (i.e. connection closed).
    fn is_connected(&self) -> Option<bool>;

    /// Send a message to the remote side
    ///
    /// `buf` contains all bytes for the message to send, it's up to the backend to
    /// decide how to encode and transport them.
    ///
    /// Only called if there is at least 1 pending message to send.
    fn send(&mut self, buf: Vec<u8>);

    /// Poll the next available message, if any
    ///
    /// Called every ticks, so can be used to try to
    /// flush message not sent yet for instance.
    fn poll(&mut self) -> Option<Vec<u8>>;
}

pub type XmlSocketHandle = Index;

#[derive(Collect)]
#[collect(no_drop)]
struct Socket<'gc> {
    target: Object<'gc>,
    #[collect(require_static)]
    internal: Box<dyn XmlSocketConnection>,
    send_buffer: VecDeque<Vec<u8>>,
    pending_connect: bool,
}

impl<'gc> Socket<'gc> {
    fn new(target: Object<'gc>, internal: Box<dyn XmlSocketConnection>) -> Self {
        Self {
            target,
            internal,
            pending_connect: true,
            send_buffer: Default::default(),
        }
    }
}

/// Manages the collection of active XmlSockets connections
pub struct XmlSockets<'gc>(Arena<Socket<'gc>>);

unsafe impl<'gc> Collect for XmlSockets<'gc> {
    fn trace(&self, cc: CollectionContext) {
        for (_, socket) in self.0.iter() {
            socket.trace(cc)
        }
    }
}

impl<'gc> XmlSockets<'gc> {
    pub fn empty() -> Self {
        Self(Arena::new())
    }

    pub fn connect(
        &mut self,
        backend: &mut dyn NavigatorBackend,
        target: Object<'gc>,
        host: &str,
        port: u16,
    ) -> Option<XmlSocketHandle> {
        if let Some(internal) = backend.connect_xml_socket(host, port) {
            let socket = Socket::new(target, internal);
            let handle = self.0.insert(socket);
            Some(handle)
        } else {
            None
        }
    }

    pub fn send(&mut self, handle: XmlSocketHandle, data: Vec<u8>) {
        if let Some(Socket { send_buffer, .. }) = self.0.get_mut(handle) {
            send_buffer.push_back(data);
        }
    }

    pub fn close(&mut self, handle: XmlSocketHandle) {
        if let Some(Socket { internal, .. }) = self.0.remove(handle) {
            drop(internal); // explicitly close connections via `Drop::drop`
        }
    }

    pub fn update_sockets(uc: &mut UpdateContext<'_, 'gc, '_>) {
        #[derive(Debug)]
        enum SocketAction {
            Connect(XmlSocketHandle, bool),
            Data(XmlSocketHandle, Vec<u8>),
            Close(XmlSocketHandle),
        }

        let mut actions = vec![];

        for (handle, socket) in uc.xml_sockets.0.iter_mut() {
            let Socket {
                pending_connect: is_connecting,
                internal,
                send_buffer,
                ..
            } = socket;

            let is_connected = internal.is_connected();

            match is_connected {
                Some(success) if *is_connecting => {
                    *is_connecting = false;
                    actions.push(SocketAction::Connect(handle, success));
                    if !success {
                        continue;
                    }
                }
                None => continue,

                _ => {}
            }

            if let Some(received) = internal.poll() {
                actions.push(SocketAction::Data(handle, received));
            } else if matches!(is_connected, Some(false) | None) {
                actions.push(SocketAction::Close(handle));
                continue;
            }

            if let Some(to_send) = send_buffer.pop_front() {
                internal.send(to_send);
            }
        }

        if actions.is_empty() {
            return;
        }

        let mut activation =
            Activation::from_stub(uc.reborrow(), ActivationIdentifier::root("[XMLSocket]"));

        for action in actions {
            match action {
                SocketAction::Connect(handle, success) => {
                    let arena = &mut activation.context.xml_sockets.0;
                    let target = if success {
                        arena
                            .get(handle)
                            .expect("only valid handles in SocketAction")
                            .target
                    } else {
                        arena
                            .remove(handle)
                            .expect("only valid handles in SocketAction")
                            .target
                    };

                    let _ = TObject::call_method(
                        &target,
                        "onConnect".into(),
                        &[success.into()],
                        &mut activation,
                        ExecutionReason::FunctionCall,
                    );
                }
                SocketAction::Close(handle) => {
                    let target = activation
                        .context
                        .xml_sockets
                        .0
                        .remove(handle)
                        .expect("only valid handles in SocketAction")
                        .target;

                    let _ = TObject::call_method(
                        &target,
                        "onClose".into(),
                        &[],
                        &mut activation,
                        ExecutionReason::FunctionCall,
                    );
                }
                SocketAction::Data(handle, data) => {
                    let target = activation
                        .context
                        .xml_sockets
                        .0
                        .get(handle)
                        .expect("only valid handles in SocketAction")
                        .target;

                    let data =
                        AvmString::new_utf8(activation.context.gc_context, UTF_8.decode(&data).0);

                    let _ = TObject::call_method(
                        &target,
                        "onData".into(),
                        &[data.into()],
                        &mut activation,
                        ExecutionReason::FunctionCall,
                    );
                }
            };
        }
    }
}
