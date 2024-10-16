use crate::{
    avm1::{
        globals::xml_socket::XmlSocket, Activation as Avm1Activation, ActivationIdentifier,
        ExecutionReason, Object as Avm1Object, TObject as Avm1TObject,
    },
    avm2::{
        object::SocketObject, Activation as Avm2Activation, Avm2, EventObject,
        TObject as Avm2TObject,
    },
    backend::navigator::NavigatorBackend,
    context::UpdateContext,
    string::AvmString,
};
use async_channel::{unbounded, Receiver, Sender as AsyncSender, Sender};
use gc_arena::Collect;
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

unsafe impl Collect for Sockets<'_> {
    fn trace(&self, cc: &gc_arena::Collection) {
        for (_, socket) in self.sockets.iter() {
            socket.trace(cc)
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
                                "onConnect".into(),
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

                            let io_error_evt = activation
                                .avm2()
                                .classes()
                                .ioerrorevent
                                .construct(
                                    &mut activation,
                                    &[
                                        "ioError".into(),
                                        false.into(),
                                        false.into(),
                                        "Error #2031: Socket Error.".into(),
                                        2031.into(),
                                    ],
                                )
                                .expect("IOErrorEvent should be constructed");

                            Avm2::dispatch_event(activation.context, io_error_evt, target.into());
                        }
                        // TODO: Not sure if avm1 xmlsocket has a way to notify a error. (Probably should just fire connect event with success as false).
                        SocketKind::Avm1(target) => {
                            let mut activation = Avm1Activation::from_stub(
                                context,
                                ActivationIdentifier::root("[XMLSocket]"),
                            );

                            let _ = target.call_method(
                                "onConnect".into(),
                                &[false.into()],
                                &mut activation,
                                ExecutionReason::Special,
                            );
                        }
                    }
                }
                SocketAction::Data(handle, mut data) => {
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

                            let progress_evt = activation
                                .avm2()
                                .classes()
                                .progressevent
                                .construct(
                                    &mut activation,
                                    &[
                                        "socketData".into(),
                                        false.into(),
                                        false.into(),
                                        bytes_loaded.into(),
                                        //NOTE: bytesTotal is not used by socketData event.
                                        0.into(),
                                    ],
                                )
                                .expect("ProgressEvent should be constructed");

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

                            // Check if the current received packet includes a null byte.
                            if let Some((index, _)) = data.iter().enumerate().find(|(_, &b)| b == 0)
                            {
                                // Received payload contains a null byte, so take data from sockets read buffer and append message data ontop.
                                let mut buffer = xml_socket
                                    .read_buffer()
                                    .drain(..)
                                    .chain(data.drain(..index))
                                    .collect::<Vec<_>>();

                                // Now we loop to check for more null bytes.
                                loop {
                                    // Remove null byte.
                                    data.drain(..1);

                                    // Create message from the buffer.
                                    let message =
                                        AvmString::new_utf8_bytes(activation.gc(), &buffer);

                                    // Call the event handler.
                                    let _ = target.call_method(
                                        "onData".into(),
                                        &[message.into()],
                                        &mut activation,
                                        ExecutionReason::Special,
                                    );

                                    // Check if we have another null byte in the same payload.
                                    if let Some((index, _)) =
                                        data.iter().enumerate().find(|(_, &b)| b == 0)
                                    {
                                        // Because data in XmlSocket::read_buffer() has already been consumed
                                        // we do not need to access it again.
                                        buffer = data.drain(..index).collect::<Vec<_>>();
                                    } else {
                                        // No more messages in the payload, so exit the loop.
                                        break;
                                    }
                                }

                                // Check if we have leftover bytes.
                                if !data.is_empty() {
                                    // We had leftover bytes, so append them to XmlSocket internal read buffer,
                                    // to be used when the next packet arrives.
                                    xml_socket.read_buffer().extend(data);
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
                            let activation = Avm2Activation::from_nothing(context);

                            // Clear the buffers if the connection was closed.
                            target.read_buffer().clear();
                            target.write_buffer().clear();

                            let close_evt =
                                EventObject::bare_default_event(activation.context, "close");
                            Avm2::dispatch_event(activation.context, close_evt, target.into());
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
                                "onClose".into(),
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
