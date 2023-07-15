use crate::{
    avm2::{object::SocketObject, Activation, Avm2, EventObject, TObject},
    backend::navigator::NavigatorBackend,
    context::UpdateContext,
};
use gc_arena::Collect;
use generational_arena::{Arena, Index};
use std::{
    cell::RefCell,
    collections::VecDeque,
    sync::mpsc::{channel, Receiver, Sender},
};

pub type SocketHandle = Index;

#[derive(Collect)]
#[collect(no_drop)]
struct Socket<'gc> {
    target: SocketObject<'gc>,
    sender: RefCell<Sender<Vec<u8>>>,
    send_buffer: VecDeque<Vec<u8>>,
}

impl<'gc> Socket<'gc> {
    fn new(target: SocketObject<'gc>, sender: Sender<Vec<u8>>) -> Self {
        Self {
            target,
            sender: RefCell::new(sender),
            send_buffer: Default::default(),
        }
    }
}

#[derive(Debug)]
pub enum SocketAction {
    Connect(SocketHandle, bool),
    Data(SocketHandle, Vec<u8>),
    Close(SocketHandle),
}

/// Manages the collection of Sockets.
pub struct Sockets<'gc> {
    sockets: Arena<Socket<'gc>>,

    receiver: Receiver<SocketAction>,
    sender: Sender<SocketAction>,
}

unsafe impl<'gc> Collect for Sockets<'gc> {
    fn trace(&self, cc: &gc_arena::Collection) {
        for (_, socket) in self.sockets.iter() {
            socket.trace(cc)
        }
    }
}

impl<'gc> Sockets<'gc> {
    pub fn empty() -> Self {
        let (sender, receiver) = channel();

        Self {
            sockets: Arena::new(),
            receiver,
            sender,
        }
    }

    pub fn connect(
        &mut self,
        backend: &mut dyn NavigatorBackend,
        target: SocketObject<'gc>,
        host: String,
        port: u16,
    ) {
        let (sender, receiver) = channel();

        let socket = Socket::new(target, sender);
        let handle = self.sockets.insert(socket);

        // NOTE: This call will send SocketAction::Connect to sender when successfully connected
        //       or SocketAction::Failed when connection failed.
        backend.connect_socket(host, port, handle, receiver, self.sender.clone());

        if let Some(existing_handle) = target.set_handle(handle) {
            // As written in the AS3 docs, we are supposed to close the existing connection,
            // when a new one is created.
            self.close(existing_handle)
        }
    }

    pub fn is_connected(&self, handle: SocketHandle) -> bool {
        matches!(self.sockets.get(handle), Some(Socket { .. }))
    }

    pub fn send(&mut self, handle: SocketHandle, data: Vec<u8>) {
        if let Some(Socket { send_buffer, .. }) = self.sockets.get_mut(handle) {
            send_buffer.push_back(data);
        }
    }

    pub fn close(&mut self, handle: SocketHandle) {
        if let Some(Socket { sender, .. }) = self.sockets.remove(handle) {
            drop(sender); // NOTE: By dropping the sender, the reading task will close automatically.
        }
    }

    pub fn update_sockets(context: &mut UpdateContext<'_, 'gc>) {
        let mut activation = Activation::from_nothing(context.reborrow());

        let mut actions = vec![];

        while let Ok(action) = activation.context.sockets.receiver.try_recv() {
            actions.push(action)
        }

        for action in actions {
            match action {
                SocketAction::Connect(handle, success) => {
                    if success {
                        let target = activation
                            .context
                            .sockets
                            .sockets
                            .get_mut(handle)
                            .expect("only valid handles in SocketAction")
                            .target;

                        let connect_evt =
                            EventObject::bare_default_event(&mut activation.context, "connect");
                        Avm2::dispatch_event(&mut activation.context, connect_evt, target.into());
                    } else {
                        // FIXME: Dispatch ioError event as connection failed.
                    }
                }
                SocketAction::Data(handle, data) => {
                    let target = activation
                        .context
                        .sockets
                        .sockets
                        .get(handle)
                        .expect("only valid handles in SocketAction")
                        .target;

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

                    Avm2::dispatch_event(&mut activation.context, progress_evt, target.into());
                }
                SocketAction::Close(handle) => {
                    let socket = activation
                        .context
                        .sockets
                        .sockets
                        .remove(handle)
                        .expect("only valid handles in SocketAction");

                    let close_evt =
                        EventObject::bare_default_event(&mut activation.context, "close");
                    Avm2::dispatch_event(&mut activation.context, close_evt, socket.target.into());
                }
            }
        }

        for (_handle, socket) in context.sockets.sockets.iter_mut() {
            let Socket {
                sender,
                send_buffer,
                ..
            } = socket;

            if let Some(to_send) = send_buffer.pop_front() {
                let _ = sender.borrow().send(to_send);
            }
        }
    }
}
