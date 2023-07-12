use crate::{
    avm2::{Activation, Avm2, EventObject, Object},
    backend::navigator::NavigatorBackend,
    context::UpdateContext,
};
use gc_arena::Collect;
use generational_arena::{Arena, Index};
use std::collections::VecDeque;

/// Socket backend implementation
///
/// When `Socket.connect()` is called, the backend may provide a
/// [SocketConnection] instance, that will be polled every tick.
/// Methods must not block.
///
/// For performances reasons, only 1 message will be polled per tick per connection
/// and at most 1 message will be sent per tick per connection.
///
/// When the Socket gets closed, the underlying [SocketConnection]
/// is dropped, so simply implement [Drop] to handle cleanup stuff.
///
/// See [SocketConnection::is_connected] for details about how
/// Ruffle infer connection state, [SocketConnection::send] for details
/// about forwarding messages, [SocketConnection::poll] for details
/// about polling incoming messages.
pub trait SocketConnection {
    /// Return socket connection state
    ///
    /// Possibles values:
    ///  * `None`: connecting ([SocketConnection::send] and
    ///            [SocketConnection::poll] not yet called)
    ///  * `Some(true)`: connected ([SocketConnection::send] and
    ///                 [SocketConnection::poll] got called)
    ///  * `Some(false)`: disconnected, or connection refused socket wasn't connected
    ///                   (called only after [SocketConnection::poll] return `None`;
    ///                    [SocketConnection::send] and [SocketConnection::poll]
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

pub type SocketHandle = Index;

#[derive(Collect)]
#[collect(no_drop)]
struct Socket<'gc> {
    target: Object<'gc>,
    #[collect(require_static)]
    internal: Box<dyn SocketConnection>,
    send_buffer: VecDeque<Vec<u8>>,
    pending_connect: bool,
}

impl<'gc> Socket<'gc> {
    fn new(target: Object<'gc>, internal: Box<dyn SocketConnection>) -> Self {
        Self {
            target,
            internal,
            pending_connect: true,
            send_buffer: Default::default(),
        }
    }
}

/// Manages the collection of Sockets.
pub struct Sockets<'gc>(Arena<Socket<'gc>>);

unsafe impl<'gc> Collect for Sockets<'gc> {
    fn trace(&self, cc: &gc_arena::Collection) {
        for (_, socket) in self.0.iter() {
            socket.trace(cc)
        }
    }
}

impl<'gc> Sockets<'gc> {
    pub fn empty() -> Self {
        Self(Arena::new())
    }

    pub fn connect(
        &mut self,
        backend: &mut dyn NavigatorBackend,
        target: Object<'gc>,
        host: &str,
        port: u16,
    ) -> Option<SocketHandle> {
        if let Some(internal) = backend.connect_socket(host, port) {
            let socket = Socket::new(target, internal);

            let handle = self.0.insert(socket);
            Some(handle)
        } else {
            None
        }
    }

    pub fn is_connected(&self, handle: SocketHandle) -> Option<bool> {
        if let Some(Socket { internal, .. }) = self.0.get(handle) {
            internal.is_connected()
        } else {
            None
        }
    }

    pub fn send(&mut self, handle: SocketHandle, data: Vec<u8>) {
        if let Some(Socket { send_buffer, .. }) = self.0.get_mut(handle) {
            send_buffer.push_back(data);
        }
    }

    pub fn close(&mut self, handle: SocketHandle) {
        if let Some(Socket { internal, .. }) = self.0.remove(handle) {
            drop(internal); // explicitly close connections via `Drop::drop`
        }
    }

    pub fn update_sockets(context: &mut UpdateContext<'_, 'gc>) {
        #[derive(Debug)]
        enum SocketAction {
            Connect(SocketHandle, bool),
            Data(SocketHandle, Vec<u8>),
            Close(SocketHandle),
        }

        let mut actions = vec![];

        for (handle, socket) in context.sockets.0.iter_mut() {
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

        let mut activation = Activation::from_nothing(context.reborrow());

        for action in actions {
            match action {
                SocketAction::Connect(handle, success) => {
                    let arena = &mut activation.context.sockets.0;
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

                    if success {
                        let connect_evt =
                            EventObject::bare_default_event(&mut activation.context, "connect");
                        Avm2::dispatch_event(&mut activation.context, connect_evt, target);
                    }
                }
                SocketAction::Close(handle) => {
                    let target = activation
                        .context
                        .sockets
                        .0
                        .remove(handle)
                        .expect("only valid handles in SocketAction")
                        .target;

                    let close_evt =
                        EventObject::bare_default_event(&mut activation.context, "close");
                    Avm2::dispatch_event(&mut activation.context, close_evt, target);
                }
                SocketAction::Data(handle, _data) => {
                    let target = activation
                        .context
                        .sockets
                        .0
                        .get(handle)
                        .expect("only valid handles in SocketAction")
                        .target;

                    let socket_data_evt =
                        EventObject::bare_default_event(&mut activation.context, "socketData");
                    Avm2::dispatch_event(&mut activation.context, socket_data_evt, target);
                }
            };
        }
    }
}
