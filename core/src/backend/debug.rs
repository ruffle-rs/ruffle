//! Backend for handling debugger communication

use crate::backend::navigator::OwnedFuture;
use crate::debugable::{Avm1Msg, DebugMessageIn, DebugMessageOut, PlayerMsg, TargetedMsg};
use crate::loader::Error as LoaderError;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// A trait that defines the async interactions between a connected debugger and the player
pub trait DebuggerBackend {
    /// Poll a single player debug event
    /// This wil be invoked before each frame and may or may not block until a message is available
    /// The recommendation is to pull events from an internal queue
    fn get_debug_event_player(&mut self) -> Option<PlayerMsg>;

    /// Poll a single targeted debug event
    /// See `get_debug_event_player` for details
    fn get_debug_event_targeted(&mut self) -> Option<(String, TargetedMsg)>;

    /// Poll a single avm1 debug event
    /// See `get_debug_event_player` for details
    fn get_debug_event_avm1(&mut self) -> Option<Avm1Msg>;

    /// Enqueue a debug message to be sent to the attached debugger if it exists
    /// This function should not block
    fn submit_debug_message(&self, _evt: DebugMessageOut);

    /// Attempt to connect to a debugger if one exists
    /// This function is free to block until the connection is established.
    //TODO: docs
    fn connect_debugger(&mut self) -> Option<OwnedFuture<(), LoaderError>>;
}

/// A null debugger backend
/// Never connects or emits events
#[derive(Debug, Default)]
pub struct NullDebuggerBackend {}

impl NullDebuggerBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DebuggerBackend for NullDebuggerBackend {
    fn get_debug_event_player(&mut self) -> Option<PlayerMsg> {
        None
    }

    fn get_debug_event_targeted(&mut self) -> Option<(String, TargetedMsg)> {
        None
    }

    fn get_debug_event_avm1(&mut self) -> Option<Avm1Msg> {
        None
    }

    fn submit_debug_message(&self, _evt: DebugMessageOut) {
        // NOOP
    }

    fn connect_debugger(&mut self) -> Option<OwnedFuture<(), LoaderError>> {
        None
    }
}

/// An implementation of a debugger backend using websockets and RIDP
/// We shouldn't need to create specific implements for each player environment as this should work everywhere,
/// however if we want to support the official Flash Player debugging protocol we can map it to this API
/// but it will require a desktop specific implementation
#[derive(Default)]
pub struct WebsocketDebugBackend {
    /// The queue of player events
    event_queue_player: Arc<RwLock<Vec<PlayerMsg>>>,

    /// The queue of targeted events
    event_queue_targeted: Arc<RwLock<Vec<(String, TargetedMsg)>>>,

    /// The queue of Avm1 events
    event_queue_avm1: Arc<RwLock<Vec<Avm1Msg>>>,

    /// The queue of outgoing events
    event_queue_out: Arc<RwLock<Vec<DebugMessageOut>>>,
}

impl WebsocketDebugBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DebuggerBackend for WebsocketDebugBackend {
    fn connect_debugger(&mut self) -> Option<OwnedFuture<(), LoaderError>> {
        let queue_local_player = Arc::clone(&self.event_queue_player);
        let queue_local_targeted = Arc::clone(&self.event_queue_targeted);
        let queue_local_avm1 = Arc::clone(&self.event_queue_avm1);
        let queue_out_local = Arc::clone(&self.event_queue_out);

        let (socket, _) = match tungstenite::connect("ws://localhost:7979/") {
            Ok(s) => s,
            Err(_) => return None,
        };
        let socket = Arc::new(RwLock::new(socket));

        // Read thread
        let socket_read = Arc::clone(&socket);
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(100));

            if let Ok(msg) = socket_read.write().unwrap().read_message() {
                if msg.is_ping() {
                    continue;
                }
                if msg.is_close() {
                    break;
                }

                if let Ok(txt) = msg.to_text() {
                    if let Ok(msg) = serde_json::from_str::<DebugMessageIn>(txt) {
                        println!("Got data: {:?}", msg);

                        match msg {
                            DebugMessageIn::Player { msg } => {
                                queue_local_player.write().unwrap().push(msg);
                            }
                            DebugMessageIn::Targeted { path, msg } => {
                                queue_local_targeted.write().unwrap().push((path, msg));
                            }
                            DebugMessageIn::Avm1 { msg } => {
                                queue_local_avm1.write().unwrap().push(msg);
                            }
                        }
                    }
                }
            }
        });

        // Write thread
        let socket_write = Arc::clone(&socket);
        std::thread::spawn(move || loop {
            if let Some(out_msg) = queue_out_local.write().unwrap().pop() {
                let mut socket_write = socket_write.write().unwrap();
                socket_write
                    .write_message(tungstenite::Message::text(
                        serde_json::to_string(&out_msg).unwrap(),
                    ))
                    .unwrap();
                socket_write.write_pending().unwrap();
                println!("Sent {:?}", out_msg);
            }
        });

        /*

        return Some(Box::pin(async move {
            use futures::{StreamExt, SinkExt};

            //stream.set_read_timeout(Some(std::time::Duration::from_millis(100))).unwrap();
            //stream.set_nonblocking(true).unwrap();

            println!("Socket connected");

            let reader = async {
                loop {
                    println!("Checking for msg");
                    if let Ok(msg) = socket.read_message() {
                        if let Ok(txt) = msg.to_text() {
                            if let Ok(msg) = serde_json::from_str::<DebugMessageIn>(txt) {
                                println!("Got data: {:?}", msg);

                                match msg {
                                    DebugMessageIn::Player { msg } => {
                                        queue_local_player.write().unwrap().push(msg);
                                    }
                                    DebugMessageIn::Targeted { path, msg } => {
                                        queue_local_targeted.write().unwrap().push((path, msg));
                                    }
                                    DebugMessageIn::Avm1 { msg } => {
                                        queue_local_avm1.write().unwrap().push(msg);
                                    }
                                }
                            }
                        }
                    }
                }
            };

            //reader.await;

            /*
            let reader = async {
                loop {
                    while let Some(msg) = stream.next().await {
                        if let Ok(txt) = msg {
                            if let Ok(msg) = serde_json::from_str::<DebugMessageIn>(&txt) {
                                println!("Got data: {:?}", msg);

                                match msg {
                                    DebugMessageIn::Player { msg } => {
                                        queue_local_player.write().unwrap().push(msg);
                                    }
                                    DebugMessageIn::Targeted { path, msg } => {
                                        queue_local_targeted.write().unwrap().push((path, msg));
                                    }
                                    DebugMessageIn::Avm1 { msg } => {
                                        queue_local_avm1.write().unwrap().push(msg);
                                    }
                                }
                            }
                        }
                    }
                }
            };

            let writer = async {
                loop {
                    if let Some(out_msg) = queue_out_local.write().unwrap().pop() {
                        sink.send(serde_json::to_string(&out_msg).unwrap()).await.unwrap();
                    }
                }
            };

            let _ = futures::join!(reader, writer);
            */
            Ok(())
        }));

        */

        None
    }

    fn submit_debug_message(&self, evt: DebugMessageOut) {
        self.event_queue_out.write().unwrap().push(evt);
    }

    fn get_debug_event_player(&mut self) -> Option<PlayerMsg> {
        self.event_queue_player.write().unwrap().pop()
    }

    fn get_debug_event_targeted(&mut self) -> Option<(String, TargetedMsg)> {
        self.event_queue_targeted.write().unwrap().pop()
    }

    fn get_debug_event_avm1(&mut self) -> Option<Avm1Msg> {
        self.event_queue_avm1.write().unwrap().pop()
    }
}

//TODO: feature flag all debug changes
//TODO: websocket
//TODO: support do commands in avm1
