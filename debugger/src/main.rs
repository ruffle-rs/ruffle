use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use ruffle_core::debugable::{DebugMessageIn, DebugMessageOut, TargetedMsg, PlayerMsg, Avm1Msg};

/// Commands that the debugger client can send the the current debuggee
#[derive(Debug, Clone)]
pub enum Command {
    /// Pause at the start of the next frame
    Pause,

    /// Resume execution of the next frame
    Play,

    /// Reconnect client
    Reconnect,

    /// Get information about the display object at the given path
    Info { path: String },

    /// Get the children of the display object at the given depth
    GetChildren { path: String },

    /// Get the properties on this object
    GetProps { path: String },

    /// Get the value of a property
    GetPropValue { path: String, name: String },

    /// Set the value of a property
    SetPropValue {
        path: String,
        name: String,
        value: String,
    },

    /// Stop the current display object
    StopDO { path: String },

    Avm1Break,
    Avm1Stack,
    Avm1StepInto,
}

#[derive(Debug, Default)]
struct DebuggerState {
    /// The last command that was executed
    last_cmd: Option<Command>,

    /// The current targeted object
    target: Option<String>,
}

fn stdin_thread(
    queue: Arc<RwLock<Vec<Command>>>,
    flag: Arc<AtomicBool>,
    state: Arc<RwLock<DebuggerState>>,
    input_block: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    std::thread::spawn(move || {
        while flag.load(Ordering::SeqCst) {
            // Don't allow input while it's blocked
            if input_block.load(Ordering::SeqCst) {
                continue;
            }

            let mut out = std::io::stdout();
            if let Some(select) = &state.read().unwrap().target {
                out.write(b"[");
                out.write(select.as_bytes());
                out.write(b"]");
            }
            out.write(b"> ");
            out.flush();

            let mut buf = [0u8; 4096];
            if let Ok(len) = std::io::stdin().read(&mut buf) {
                if len == 0 {
                    break;
                }

                let buf = &buf[..len];
                let s = String::from_utf8(buf.to_vec()).unwrap();

                let mut cmd = None;

                if s.starts_with("pause") {
                    cmd = Some(Command::Pause);
                } else if s.starts_with("play") || s == "c\n" {
                    cmd = Some(Command::Play);
                } else if s.starts_with("reconnect") || s == "rc\n" {
                    cmd = Some(Command::Reconnect);
                } else if s.starts_with("get_info") {
                    let var_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(1)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| {
                            state
                                .read()
                                .unwrap()
                                .target
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "".to_string())
                        });
                    cmd = Some(Command::Info { path: var_name });
                } else if s.starts_with("get_children") {
                    let var_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(1)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| {
                            state
                                .read()
                                .unwrap()
                                .target
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "".to_string())
                        });
                    cmd = Some(Command::GetChildren { path: var_name });
                } else if s.starts_with("get_props") {
                    let var_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(1)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| {
                            state
                                .read()
                                .unwrap()
                                .target
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "".to_string())
                        });
                    cmd = Some(Command::GetProps { path: var_name });
                } else if s.starts_with("stop_do") {
                    let var_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(1)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| {
                            state
                                .read()
                                .unwrap()
                                .target
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "".to_string())
                        });
                    cmd = Some(Command::StopDO { path: var_name });
                } else if s.starts_with("get_prop") {
                    let var_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(1)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| {
                            state
                                .read()
                                .unwrap()
                                .target
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "".to_string())
                        });

                    let arg_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(2)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap();

                    cmd = Some(Command::GetPropValue {
                        path: var_name,
                        name: arg_name,
                    });
                } else if s.starts_with("set_prop") {
                    let var_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(1)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap_or_else(|| {
                            state
                                .read()
                                .unwrap()
                                .target
                                .as_ref()
                                .map(|x| x.to_string())
                                .unwrap_or_else(|| "".to_string())
                        });

                    let arg_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(2)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap();

                    let new_value = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(3)
                        .next()
                        .map(|x| x.to_string())
                        .unwrap();

                    cmd = Some(Command::SetPropValue {
                        path: var_name,
                        name: arg_name,
                        value: new_value,
                    });
                } else if s.starts_with("select") {
                    let var_name = s
                        .strip_suffix("\n")
                        .unwrap()
                        .split(" ")
                        .skip(1)
                        .next()
                        .unwrap();
                    state.write().unwrap().target = Some(var_name.to_string());
                } else if s.starts_with("avm1_break") {
                    cmd = Some(Command::Avm1Break);
                } else if s.starts_with("avm1_stack") {
                    cmd = Some(Command::Avm1Stack);
                } else if s.starts_with("avm1_step") {
                    cmd = Some(Command::Avm1StepInto);
                } else if s == "\n" {
                    cmd = state.write().unwrap().last_cmd.take();
                } else {
                    println!("Unknown command");
                }

                if let Some(cmd) = cmd {
                    println!("Got cmd {:?}", cmd);
                    queue.write().unwrap().push(cmd.clone());
                    state.write().unwrap().last_cmd = Some(cmd);
                    input_block.store(true, Ordering::SeqCst);
                }
            }
        }
    })
}

fn handle_client(mut stream: TcpStream) {
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .unwrap();
    let queue = Arc::new(RwLock::new(Vec::new()));
    let flag = Arc::new(AtomicBool::new(true));
    let state = Arc::new(RwLock::new(DebuggerState::default()));
    let input_block = Arc::new(AtomicBool::new(false));
    let input_thread = stdin_thread(
        Arc::clone(&queue),
        Arc::clone(&flag),
        Arc::clone(&state),
        Arc::clone(&input_block),
    );

    loop {
        if let Some(cmd) = queue.write().unwrap().pop() {
            match cmd {
                Command::Avm1StepInto => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Avm1 { msg: Avm1Msg::StepInto})
                                .unwrap()
                                .as_bytes(),
                        )
                        .unwrap();
                    
                }
                Command::Avm1Stack => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Avm1 { msg: Avm1Msg::GetStack})
                                .unwrap()
                                .as_bytes(),
                        )
                        .unwrap();
                    
                }
                Command::Avm1Break => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Avm1 { msg: Avm1Msg::Break})
                                .unwrap()
                                .as_bytes(),
                        )
                        .unwrap();
                    
                }
                Command::Pause => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Player { msg: PlayerMsg::Pause})
                                .unwrap()
                                .as_bytes(),
                        )
                        .unwrap();
                }
                Command::Play => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Player { msg: PlayerMsg::Play})
                                .unwrap()
                                .as_bytes(),
                        )
                        .unwrap();
                }
                Command::Reconnect => {
                    break;
                }
                Command::Info { path } => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Targeted {
                                path,
                                msg: TargetedMsg::GetInfo,
                            })
                            .unwrap()
                            .as_bytes(),
                        )
                        .unwrap();
                }
                Command::GetChildren { path } => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Targeted {
                                path,
                                msg: TargetedMsg::GetChildren,
                            })
                            .unwrap()
                            .as_bytes(),
                        )
                        .unwrap();
                }
                Command::GetProps { path } => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Targeted {
                                path,
                                msg: TargetedMsg::GetProps,
                            })
                            .unwrap()
                            .as_bytes(),
                        )
                        .unwrap();
                }
                Command::GetPropValue { path, name } => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Targeted {
                                path,
                                msg: TargetedMsg::GetPropValue { name },
                            })
                            .unwrap()
                            .as_bytes(),
                        )
                        .unwrap();
                }
                Command::SetPropValue { path, name, value } => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Targeted {
                                path,
                                msg: TargetedMsg::SetPropValue { name, value },
                            })
                            .unwrap()
                            .as_bytes(),
                        )
                        .unwrap();
                }
                Command::StopDO { path } => {
                    stream
                        .write(
                            serde_json::to_string(&DebugMessageIn::Targeted {
                                path,
                                msg: TargetedMsg::Stop,
                            })
                            .unwrap()
                            .as_bytes(),
                        )
                        .unwrap();
                }
                _ => {}
            }
        }

        {
            let mut buf = [0u8; 4096];
            if let Ok(len) = stream.read(&mut buf) {
                if len == 0 {
                    break;
                }

                let buf = &buf[..len];
                let s = String::from_utf8(buf.to_vec()).unwrap();
                println!("Got incomming {:?}", s);

                if let Ok(msg) = serde_json::from_str::<DebugMessageOut>(&s) {
                    match msg {
                        DebugMessageOut::State { playing } => {
                            println!("Playing: {}", playing);
                        }
                        DebugMessageOut::BreakpointHit { name } => {
                            println!("Hit BP {}", name);
                            //stream.write(serde_json::to_string(&DebugMessageIn::Pause).unwrap().as_bytes()).unwrap();
                        }
                        DebugMessageOut::GetVarResult { value } => {
                            println!("Result: {:?}", value);
                        }
                        DebugMessageOut::DisplayObjectInfo(i) => {
                            println!("Info = {:#?}", i);
                        }
                        DebugMessageOut::GetPropsResult { keys } => {
                            for key in &keys {
                                println!("\"{}\"", key);
                            }
                        }
                        DebugMessageOut::GenericResult { success } => {
                            if success {
                                println!("success");
                            } else {
                                println!("fail");
                            }
                        }
                    }
                }
                input_block.store(false, Ordering::SeqCst);
            }
        }
    }

    flag.store(false, Ordering::SeqCst);
    input_thread.join().unwrap();
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7979").unwrap();

    loop {
        if let Some(Ok(stream)) = listener.incoming().next() {
            println!("New connection: {}", stream.peer_addr().unwrap());
            handle_client(stream);
            println!("client d/c");
        }
    }
    // close the socket server
    drop(listener);
}
