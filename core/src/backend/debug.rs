//! Backend for handling debugger communication

use crate::debugable::{DebugMessageIn, DebugMessageOut, PlayerMsg, TargetedMsg, Avm1Msg};
use std::{
    io::Read,
    net::TcpStream,
    sync::{Arc, RwLock},
};

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
    fn submit_debug_message(&mut self, _evt: DebugMessageOut);

    /// Attempt to connect to a debugger if one exists
    /// This function is free to block until the connection is established.
    fn connect_debugger(&mut self);
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

    fn submit_debug_message(&mut self, _evt: DebugMessageOut) {
        // NOOP
    }

    fn connect_debugger(&mut self) {
        // NOOP
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
    fn connect_debugger(&mut self) {
        let queue_local_player = Arc::clone(&self.event_queue_player);
        let queue_local_targeted = Arc::clone(&self.event_queue_targeted);
        let queue_local_avm1 = Arc::clone(&self.event_queue_avm1);
        let queue_out_local = Arc::clone(&self.event_queue_out);

        // spawn debugger thread
        std::thread::spawn(move || {
            if let Ok(mut stream) = TcpStream::connect("localhost:7979") {
                stream
                    .set_read_timeout(Some(std::time::Duration::from_millis(100)))
                    .unwrap();

                loop {
                    {
                        let mut data = [0u8; 1024];
                        if let Ok(len) = stream.read(&mut data) {
                            if len == 0 {
                                break;
                            }

                            let data = &data[..len];
                            let s = String::from_utf8(data.to_vec()).unwrap();

                            if let Ok(msg) = serde_json::from_str::<DebugMessageIn>(&s) {
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

                    if let Some(out_msg) = queue_out_local.write().unwrap().pop() {
                        std::io::Write::write(
                            &mut stream,
                            serde_json::to_string(&out_msg).unwrap().as_bytes(),
                        )
                        .unwrap();
                    }
                }
            }
        });
    }

    fn submit_debug_message(&mut self, evt: DebugMessageOut) {
        self.event_queue_out.write().unwrap().push(evt.clone());
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

//TODO: procotol name: ridp?
//TODO: feature flag all debug changes
//TODO: websocket
