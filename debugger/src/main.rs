use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use crate::command::Command;
use ruffle_core::debugable::{Avm1Msg, DebugMessageIn, DebugMessageOut, PlayerMsg, TargetedMsg};
use tungstenite::{Message, WebSocket};

pub mod command;
pub mod command_parser;

#[derive(Debug, Default)]
struct DebuggerState {
    /// The last command that was executed
    last_cmd: Option<Command>,

    /// Are we in a breakpoint / is the avm paused
    ///
    /// If the avm is currently running, then some operations such as stack push or pop are
    /// practically impossible to send "safely" because the current state will be changing
    /// as they are being typed.
    /// Additionally, single stepping execution requires that the vm is paused.
    in_breakpoint: bool,
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

            // Write a prefix
            out.write_all(b"> ").unwrap();
            out.flush().unwrap();

            // Read blockingly
            let mut buf = [0u8; 4096];
            if let Ok(len) = std::io::stdin().read(&mut buf) {
                if len == 0 {
                    break;
                }

                // Convert input to string
                let buf = &buf[..len];
                let s = String::from_utf8(buf.to_vec()).unwrap();

                // Newline implies repeat last command, otherwise use parser
                let cmd = if s == "\n" {
                    state.write().unwrap().last_cmd.take()
                } else {
                    crate::command_parser::parse_command(&s.strip_suffix('\n').unwrap())
                };

                match cmd {
                    // No command, log warning
                    None => println!("Unknown command"),

                    // We have something to do, put it in the queue
                    Some(cmd) => {
                        queue.write().unwrap().push(cmd.clone());
                        state.write().unwrap().last_cmd = Some(cmd);
                        input_block.store(true, Ordering::SeqCst);
                    }
                }
            }
        }
    })
}

fn handle_client(mut stream: WebSocket<TcpStream>) {
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

    fn send_msg(stream: &mut WebSocket<TcpStream>, msg: DebugMessageIn) {
        let _ = stream.write_message(Message::text(serde_json::to_string(&msg).unwrap()));
        let _ = stream.write_pending();
    }

    let mut last_ping = Instant::now();

    loop {
        // If we have a command, convert to `DebugMessageIn` and send
        if let Some(cmd) = queue.write().unwrap().pop() {
            // Check that we can exec this command
            if cmd.requires_paused_vm() && !state.read().unwrap().in_breakpoint {
                println!("This command requires that the vm is paused, try breaking execution");
                continue;
            }

            match cmd {
                Command::Avm1Locals => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetLocals,
                        },
                    );
                }
                Command::Avm1Globals => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetGlobals,
                        },
                    );
                }
                Command::Avm1Backtrace => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetBacktrace,
                        },
                    );
                }
                Command::Avm1Registers => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetRegisters,
                        },
                    );
                }
                Command::Avm1SubpropGet { path } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetSubprops { path },
                        },
                    );
                }
                Command::Avm1VariableSet { path, value } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::SetVariable { path, value },
                        },
                    );
                }
                Command::Avm1VariableGet { path } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetVariable { path },
                        },
                    );
                }
                Command::Avm1BreakpointsGet => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetBreakpoints,
                        },
                    );
                }
                Command::Avm1Pop => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::Pop });
                }
                Command::Avm1Push { val } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::Push { val },
                        },
                    );
                }
                Command::Avm1Continue => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::Continue,
                        },
                    );
                    state.write().unwrap().in_breakpoint = true;
                }
                Command::Avm1FunctionBreakDelete { name } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::BreakFunctionDelete { name },
                        },
                    );
                }
                Command::Avm1FunctionBreak { name } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::BreakFunction { name },
                        },
                    );
                }
                Command::Avm1StepInto => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::StepInto,
                        },
                    );
                }
                Command::Avm1StepOut => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::StepOut,
                        },
                    );
                }
                Command::Avm1Stack => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::GetStack,
                        },
                    );
                }
                Command::Avm1Break => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Avm1 {
                            msg: Avm1Msg::Break,
                        },
                    );
                }
                Command::Pause => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Player {
                            msg: PlayerMsg::Pause,
                        },
                    );
                }
                Command::Play => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Player {
                            msg: PlayerMsg::Play,
                        },
                    );
                }
                Command::Info { path } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Targeted {
                            path,
                            msg: TargetedMsg::GetInfo,
                        },
                    );
                }
                Command::GetChildren { path } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Targeted {
                            path,
                            msg: TargetedMsg::GetChildren,
                        },
                    );
                }
                Command::GetProps { path } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Targeted {
                            path,
                            msg: TargetedMsg::GetChildren,
                        },
                    );
                }
                Command::GetPropValue { path, name } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Targeted {
                            path,
                            msg: TargetedMsg::GetPropValue { name },
                        },
                    );
                }
                Command::SetPropValue { path, name, value } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Targeted {
                            path,
                            msg: TargetedMsg::SetPropValue { name, value },
                        },
                    );
                }
                Command::StopDO { path } => {
                    send_msg(
                        &mut stream,
                        DebugMessageIn::Targeted {
                            path,
                            msg: TargetedMsg::Stop,
                        },
                    );
                }
            }
        }

        // Handle any incoming messages
        if let Ok(msg) = stream.read_message() {
            // Exit if the stream is closed
            if msg.is_close() {
                break;
            }

            // Ignore ping and pong
            if msg.is_ping() || msg.is_pong() {
            } else if let Ok(txt) = msg.to_text() {
                // We got a message, display it
                if let Ok(msg) = serde_json::from_str::<DebugMessageOut>(txt) {
                    match msg {
                        DebugMessageOut::State { playing } => {
                            println!("Playing: {}", playing);
                        }
                        DebugMessageOut::BreakpointHit { name } => {
                            println!("Hit BP {}", name);
                            state.write().unwrap().in_breakpoint = true;
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
                        DebugMessageOut::BreakpointList { bps } => {
                            println!("Breakpoints:");
                            for bp in &bps {
                                println!("{}", bp);
                            }
                        }
                        DebugMessageOut::GetValueResult { path, value } => {
                            println!("{} = {:?}", path, value);
                        }
                        DebugMessageOut::GetSubpropsResult { path, props } => {
                            println!("{} = {{", path);
                            for p in &props {
                                println!("    {},", p);
                            }
                            println!("}}");
                        }
                        DebugMessageOut::GetRegisterResult { regs } => {
                            for (i, r) in regs.iter().enumerate() {
                                println!("register{} => {:?}", i + 1, r);
                            }
                        }
                        DebugMessageOut::GetBacktraceResult { backtrace } => {
                            for b in backtrace {
                                println!("{}", b);
                            }
                        }
                        DebugMessageOut::GetLocalsResult { locals } => {
                            for b in locals {
                                println!("{}", b);
                            }
                        }
                        DebugMessageOut::GetGlobalsResult { globals } => {
                            for b in globals {
                                println!("{}", b);
                            }
                        }
                        DebugMessageOut::LogTrace(t) => {
                            println!("[trace] {}", t);
                        }
                    }
                }
                input_block.store(false, Ordering::SeqCst);
            }
        }

        // We send a ping at least once every 500ms so that the client won't get stuck waiting for incomming data when using a blocking tcp channel
        // This is needed as async tcp without a standard async runtime (such as Ruffle desktop) is quite difficult
        // This won't really be needed on web, but shouldn't hurt either
        if Instant::now().duration_since(last_ping) > Duration::from_millis(200) {
            if stream.can_write() {
                stream.write_message(Message::Ping(Vec::new())).unwrap();
                stream.write_pending().unwrap();
            } else {
                break;
            }
            last_ping = Instant::now();
        }
    }

    // The client has d/c'ed, stop the input thread
    flag.store(false, Ordering::SeqCst);

    // Wait for the thread to die
    input_thread.join().unwrap();
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7979").unwrap();

    while let Some(Ok(stream)) = listener.incoming().next() {
        println!("New connection: {}", stream.peer_addr().unwrap());
        stream.set_nonblocking(true).unwrap();
        let wsc = tungstenite::accept(stream).unwrap();
        handle_client(wsc);
        println!("client d/c");
    }
}
