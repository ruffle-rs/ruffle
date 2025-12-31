use crate::avm1::{
    globals::xml_socket::XmlSocket, Activation as Avm1Activation, ActivationIdentifier,
    ExecutionReason, Object as Avm1Object,
};
use crate::avm2::object::{EventObject, SocketObject};
use crate::avm2::{Activation as Avm2Activation, Avm2};
use crate::backend::navigator::NavigatorBackend;
use crate::context::UpdateContext;
use crate::string::AvmString;

use async_channel::{unbounded, Receiver, Sender as AsyncSender, Sender};
use gc_arena::collect::Trace;
use gc_arena::Collect;
use ruffle_macros::istr;
use slotmap::{new_key_type, SlotMap};
use std::{
    cell::{Cell, RefCell},
    time::Duration,
};

new_key_type! {
    pub struct SocketHandle;
}

#[derive(Copy, Clone, Collect)]
#[collect(no_drop)]
enum SocketKind<'gc> {
    Avm2(SocketObject<'gc>),
    Avm1(Avm1Object<'gc>),
}

#[derive(Collect)]
#[collect(no_drop)]
struct Socket<'gc> {
    target: SocketKind<'gc>,
    sender: RefCell<AsyncSender<Vec<u8>>>,
    connected: Cell<bool>,
}

impl<'gc> Socket<'gc> {
    fn new(target: SocketKind<'gc>, sender: AsyncSender<Vec<u8>>) -> Self {
        Self {
            target,
            sender: RefCell::new(sender),
            connected: Cell::new(false),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Failed,
    TimedOut,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SocketAction {
    Connect(SocketHandle, ConnectionState),
    Data(SocketHandle, Vec<u8>),
    Close(SocketHandle),
}

/// Manages the collection of Sockets.
pub struct Sockets<'gc> {
    sockets: SlotMap<SocketHandle, Socket<'gc>>,

    receiver: Receiver<SocketAction>,
    sender: Sender<SocketAction>,
}

unsafe impl<'gc> Collect<'gc> for Sockets<'gc> {
    fn trace<C: Trace<'gc>>(&self, cc: &mut C) {
        for (_, socket) in self.sockets.iter() {
            cc.trace(socket);
        }
    }
}

impl<'gc> Sockets<'gc> {
    pub fn empty() -> Self {
        let (sender, receiver) = unbounded();

        Self {
            sockets: SlotMap::with_key(),
            receiver,
            sender,
        }
    }

    pub fn connect_avm2(
        &mut self,
        backend: &mut dyn NavigatorBackend,
        target: SocketObject<'gc>,
        host: String,
        port: u16,
    ) {
        let (sender, receiver) = unbounded();

        let socket = Socket::new(SocketKind::Avm2(target), sender);
        let handle = self.sockets.insert(socket);

        // NOTE: This call will send SocketAction::Connect to sender with connection status.
        backend.connect_socket(
            sanitize_host(&host).to_string(),
            port,
            Duration::from_millis(target.timeout().into()),
            handle,
            receiver,
            self.sender.clone(),
        );

        if let Some(existing_handle) = target.set_handle(handle) {
            // As written in the AS3 docs, we are supposed to close the existing connection,
            // when a new one is created.
            self.close(existing_handle)
        }
    }

    pub fn connect_avm1(
        &mut self,
        backend: &mut dyn NavigatorBackend,
        target: Avm1Object<'gc>,
        host: String,
        port: u16,
    ) {
        let (sender, receiver) = unbounded();

        let xml_socket = match XmlSocket::cast(target.into()) {
            Some(xml_socket) => xml_socket,
            None => return,
        };

        let socket = Socket::new(SocketKind::Avm1(target), sender);
        let handle = self.sockets.insert(socket);

        // NOTE: This call will send SocketAction::Connect to sender with connection status.
        backend.connect_socket(
            sanitize_host(&host).to_string(),
            port,
            Duration::from_millis(xml_socket.timeout().into()),
            handle,
            receiver,
            self.sender.clone(),
        );

        if let Some(existing_handle) = xml_socket.set_handle(handle) {
            // NOTE: AS2 docs don't specify what happens when connect is called with open connection,
            //       but we will close the existing connection anyway.
            self.close(existing_handle)
        }
    }

    pub fn is_connected(&self, handle: SocketHandle) -> bool {
        if let Some(socket) = self.sockets.get(handle) {
            socket.connected.get()
        } else {
            false
        }
    }

    pub fn send(&mut self, handle: SocketHandle, data: Vec<u8>) {
        if let Some(Socket { sender, .. }) = self.sockets.get_mut(handle) {
            // We use an unbounded socket, so this should only ever error if the channel is closed
            // (the receiver was dropped)
            if let Err(e) = sender.borrow().try_send(data) {
                tracing::error!("Failed to send data to socket: {:?}", e);
            }
        }
    }

    pub fn close_all(&mut self) {
        for (_, socket) in self.sockets.drain() {
            Self::close_internal(socket);
        }
    }

    pub fn close(&mut self, handle: SocketHandle) {
        if let Some(socket) = self.sockets.remove(handle) {
            Self::close_internal(socket);
        }
    }

    fn close_internal(socket: Socket) {
        let Socket {
            sender,
            target,
            connected: _,
        } = socket;

        drop(sender); // NOTE: By dropping the sender, the reading task will close automatically.

        // Clear the buffers if the connection was closed.
        match target {
            SocketKind::Avm1(target) => {
                let target = XmlSocket::cast(target.into()).expect("target should be XmlSocket");

                target.read_buffer().clear();
            }
            SocketKind::Avm2(target) => {
                target.read_buffer().clear();
                target.write_buffer().clear();
            }
        }
    }

    pub fn update_sockets(context: &mut UpdateContext<'gc>) {
        let mut actions = vec![];

        while let Ok(action) = context.sockets.receiver.try_recv() {
            actions.push(action)
        }

        for action in actions {
            match action {
                SocketAction::Connect(handle, ConnectionState::Connected) => {
                    let target = match context.sockets.sockets.get(handle) {
                        Some(socket) => {
                            socket.connected.set(true);
                            socket.target
                        }
                        // Socket must have been closed before we could send event.
                        None => continue,
                    };

                    match target {
                        SocketKind::Avm2(target) => {
                            let activation = Avm2Activation::from_nothing(context);

                            let connect_evt =
                                EventObject::bare_default_event(activation.context, "connect");
                            Avm2::dispatch_event(activation.context, connect_evt, target.into());
                        }
                        SocketKind::Avm1(target) => {
                            let mut activation = Avm1Activation::from_stub(
                                context,
                                ActivationIdentifier::root("[XMLSocket]"),
                            );

                            let _ = target.call_method(
                                istr!("onConnect"),
                                &[true.into()],
                                &mut activation,
                                ExecutionReason::Special,
                            );
                        }
                    }
                }
                SocketAction::Connect(
                    handle,
                    ConnectionState::Failed | ConnectionState::TimedOut,
                ) => {
                    let target = match context.sockets.sockets.get(handle) {
                        Some(socket) => socket.target,
                        // Socket must have been closed before we could send event.
                        None => continue,
                    };

                    match target {
                        SocketKind::Avm2(target) => {
                            let mut activation = Avm2Activation::from_nothing(context);

                            let io_error_evt = EventObject::io_error_event(
                                &mut activation,
                                "Error #2031: Socket Error.",
                                2031,
                            );

                            Avm2::dispatch_event(activation.context, io_error_evt, target.into());
                        }
                        // TODO: Not sure if avm1 xmlsocket has a way to notify a error. (Probably should just fire connect event with success as false).
                        SocketKind::Avm1(target) => {
                            let mut activation = Avm1Activation::from_stub(
                                context,
                                ActivationIdentifier::root("[XMLSocket]"),
                            );

                            let _ = target.call_method(
                                istr!("onConnect"),
                                &[false.into()],
                                &mut activation,
                                ExecutionReason::Special,
                            );
                        }
                    }
                }
                SocketAction::Data(handle, data) => {
                    let target = match context.sockets.sockets.get(handle) {
                        Some(socket) => socket.target,
                        // Socket must have been closed before we could send event.
                        None => continue,
                    };

                    match target {
                        SocketKind::Avm2(target) => {
                            let mut activation = Avm2Activation::from_nothing(context);

                            let bytes_loaded = data.len();
                            target.read_buffer().extend(data);

                            let progress_evt = EventObject::progress_event(
                                &mut activation,
                                "socketData",
                                bytes_loaded,
                                0, // NOTE: bytesTotal is not used by socketData event.
                            );

                            Avm2::dispatch_event(activation.context, progress_evt, target.into());
                        }
                        SocketKind::Avm1(target) => {
                            let mut activation = Avm1Activation::from_stub(
                                context,
                                ActivationIdentifier::root("[XMLSocket]"),
                            );

                            // NOTE: This is enforced in connect_avm1() function.
                            let xml_socket =
                                XmlSocket::cast(target.into()).expect("target should be XmlSocket");

                            // The read buffer should never contain null bytes at this point,
                            // since they are always processed immediately. Therefore, we
                            // only need to check the new data for null bytes.
                            let has_null = data.contains(&0);
                            xml_socket.read_buffer().extend(data);

                            if has_null {
                                // Process complete messages (null-terminated) one at a time.
                                // We release the buffer borrow before each AS call to avoid
                                // conflicts if AS code accesses the socket.
                                loop {
                                    let message = {
                                        let buffer = &mut *xml_socket.read_buffer();
                                        match buffer.iter().position(|&b| b == 0) {
                                            Some(pos) => {
                                                let msg: Vec<u8> = buffer.drain(..=pos).collect();
                                                Some(AvmString::new_utf8_bytes(
                                                    activation.gc(),
                                                    &msg[..msg.len() - 1],
                                                ))
                                            }
                                            None => None,
                                        }
                                    };

                                    match message {
                                        Some(msg) => {
                                            let _ = target.call_method(
                                                istr!("onData"),
                                                &[msg.into()],
                                                &mut activation,
                                                ExecutionReason::Special,
                                            );
                                        }
                                        None => break,
                                    }
                                }
                            }
                        }
                    }
                }
                SocketAction::Close(handle) => {
                    let target = match context.sockets.sockets.remove(handle) {
                        Some(socket) => {
                            socket.connected.set(false);
                            socket.target
                        }
                        // Socket must have been closed before we could send event.
                        None => continue,
                    };

                    match target {
                        SocketKind::Avm2(target) => {
                            // Clear the buffers if the connection was closed.
                            target.read_buffer().clear();
                            target.write_buffer().clear();

                            let close_evt = EventObject::bare_default_event(context, "close");
                            Avm2::dispatch_event(context, close_evt, target.into());
                        }
                        SocketKind::Avm1(target) => {
                            let mut activation = Avm1Activation::from_stub(
                                context,
                                ActivationIdentifier::root("[XMLSocket]"),
                            );

                            // Clear the read buffer if the connection was closed.
                            let socket =
                                XmlSocket::cast(target.into()).expect("target should be XmlSocket");

                            socket.read_buffer().clear();

                            let _ = target.call_method(
                                istr!("onClose"),
                                &[],
                                &mut activation,
                                ExecutionReason::Special,
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Flash treats a socket host as a cstring, and stops reading at a null byte.
/// We need to account for this here.
fn sanitize_host(host: &str) -> &str {
    host.split('\0').next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::sanitize_host;

    #[test]
    fn truncate_host_at_null() {
        assert_eq!(
            sanitize_host("1.2.3.4\0nonsense that gets dropped\0"),
            "1.2.3.4"
        );
        assert_eq!(sanitize_host("\0nonsense"), "");
        assert_eq!(sanitize_host("host\0"), "host");
    }

    #[test]
    fn normal_host() {
        assert_eq!(sanitize_host("1.2.3.4"), "1.2.3.4");
    }
}
