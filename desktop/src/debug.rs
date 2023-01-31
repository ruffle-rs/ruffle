use ruffle_core::backend::debug::DebuggerBackend;
use ruffle_core::backend::navigator::OwnedFuture;
use ruffle_core::debug::avm1_message::Avm1Msg;
use ruffle_core::debug::debug_message_in::DebugMessageIn;
use ruffle_core::debug::debug_message_out::DebugMessageOut;
use ruffle_core::debug::player_message::PlayerMsg;
use ruffle_core::debug::targeted_message::TargetedMsg;
use ruffle_core::loader::Error as LoaderError;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// An implementation of a debugger backend using websockets and RIDP via tungstenite
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
    fn get_debug_event_player(&mut self) -> Option<PlayerMsg> {
        self.event_queue_player.write().unwrap().pop()
    }

    fn get_debug_event_targeted(&mut self) -> Option<(String, TargetedMsg)> {
        self.event_queue_targeted.write().unwrap().pop()
    }

    fn get_debug_event_avm1(&mut self) -> Option<Avm1Msg> {
        self.event_queue_avm1.write().unwrap().pop()
    }

    fn submit_debug_message(&self, evt: DebugMessageOut) {
        self.event_queue_out.write().unwrap().push(evt);
    }

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
            }
        });

        None
    }
}
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
