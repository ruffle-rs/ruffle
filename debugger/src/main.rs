use std::io::{Read, Write};
use std::net::{Shutdown, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use ruffle_core::debugable::{DebugMessageIn, DebugMessageOut, TargetedMsg, PlayerMsg, Avm1Msg, DValue};


fn smatch<'a, 'b: 'a>(s: &'a str, t: &'b str) -> Option<&'a str> {
    let s = s.trim_start();
    if s.starts_with(t) {
        Some(&s[t.len()..].trim_start())
    } else {
        None
    }
}

fn parse_value(s: &str) -> Option<DValue> {
    if let Ok(v) = s.parse::<i32>() {
        Some(DValue::Int(v))
    } else if let Ok(v) = s.parse::<f64>() {
        Some(DValue::Number(v))
    } else if s == "null" {
        Some(DValue::Null)
    } else if s == "undefined" {
        Some(DValue::Undefined)
    } else {
        //TODO: should this require quotes
        Some(DValue::String(s.to_string()))
    }
}

fn parse_avm1_command(cmd: &str) -> Option<Command> {
    if let Some(_) = smatch(cmd, "help") {
        println!("Ruffle Debugger Help (AVM1)");
        println!("");
        println!("() = Short form");
        println!("");
        println!("Values:");
        println!("null - Null");
        println!("undefined - Undefined");
        println!("123.4 - Number");
        println!("123 - Int");
        println!("\"Foo\" - String");
        println!("");
        println!("Commands:");
        println!("avm1 break - Break execution at the next instruction");
        println!("avm1 breakpoint list - List active breakpoints");
        println!("avm1 breakpoint add \"function_name\" - Break execution when \"function_name\" is called");
        println!("avm1 breakpoint remove \"function_name\" - Remove a breakpoint");
        println!("");
        println!("Only available when in a breakpoint:");
        println!("avm1 (si)/step - Execute next instruction");
        //println!("avm1 (so)/step_over - Execute next instruction, without following calls");
        //println!("avm1 (sr)/step_out - Continue execution, until returning");
        println!("");
        println!("avm1 stack show");
        println!("avm1 stack push <Value>");
        println!("avm1 stack pop");
        println!("");
        println!("avm1 get <path> - Get the value of the variable at <path>");
        println!("avm1 set <path> <value> - Set the value of the variable at <path> to <value>");
        println!("avm1 props <path> - Get the sub-properties of the variable at <path>");
        println!();
        println!("avm1 continue");
        println!("");
        println!("avm1 help - View this message");
    } else if let Some(path) = smatch(cmd, "set") {
        let mut parts = path.split(" ");
        let path = parts.next().unwrap();
        let value = parts.next().unwrap();

        if let Some(value) = parse_value(value) {
            return Some(Command::Avm1VariableSet { path: path.to_string(), value, });
        } else {
            return None;
        }
    } else if let Some(path) = smatch(cmd, "props") {
        return Some(Command::Avm1SubpropGet { path: path.to_string() });
    } else if let Some(path) = smatch(cmd, "get") {
        return Some(Command::Avm1VariableGet { path: path.to_string() });
    } else if let Some(bp) = smatch(cmd, "breakpoint") {
        if let Some(name) = smatch(bp, "add") {
            return Some(Command::Avm1FunctionBreak { name: name.to_string() });
        } else if let Some(name) = smatch(bp, "remove") {
            return Some(Command::Avm1FunctionBreakDelete { name: name.to_string() });
        } else if let Some(_) = smatch(bp, "list") {
            return Some(Command::Avm1BreakpointsGet);
        }    
    } else if let Some(bp) = smatch(cmd, "break") {
            return Some(Command::Avm1Break);
    } else if let Some(_) = smatch(cmd, "step") {
        return Some(Command::Avm1StepInto);
    } else if let Some(stack) = smatch(cmd, "stack") {
        if let Some(_) = smatch(stack, "show") {
            return Some(Command::Avm1Stack);
        } else if let Some(arg) = smatch(stack, "push") {
            if let Some(val) = parse_value(arg) {
                return Some(Command::Avm1Push { val });
            }
        } else if let Some(_) = smatch(stack, "pop") {
            return Some(Command::Avm1Pop);
        }
    } else if let Some(_) = smatch(cmd, "continue") {
        return Some(Command::Avm1Continue);
    }

    None
}

fn parse_player_command(cmd: &str) -> Option<Command> {
    if let Some(_) = smatch(cmd, "help") {
        println!("Ruffle Debugger Help (Player)");
        println!("");
        println!("Commands:");
        println!("player pause");
        println!("player resume");
        println!("");
        println!("player help - View this message");
    } else if let Some(_) = smatch(cmd, "pause") {
        return Some(Command::Pause);
    } else if let Some(_) = smatch(cmd, "resume") {
        return Some(Command::Play);
    }

    None
}

fn parse_do_command(cmd: &str) -> Option<Command> {
    if let Some(_) = smatch(cmd, "help") {
        println!("Ruffle Debugger Help (Display Object)");
        println!("");
        println!("() = Short form");
        println!("");
        println!("Commands:");
        println!("do info <path>");
        println!("do children <path>");
        println!("do props <path>");
        println!("do stop <path>");
        println!("do prop get <path> <prop_name>");
        println!("");
        println!("do help - View this message");
    } else if let Some(path) = smatch(cmd, "info") {
        return Some(Command::Info { path: path.to_string()});
    } else if let Some(path) = smatch(cmd, "children") {
        return Some(Command::GetChildren { path: path.to_string()});
    } else if let Some(path) = smatch(cmd, "props") {
        return Some(Command::GetProps { path: path.to_string()});
    } else if let Some(path) = smatch(cmd, "stop") {
        return Some(Command::StopDO { path: path.to_string()});
    } else if let Some(prop) = smatch(cmd, "prop") {
        if let Some(args) = smatch(prop, "get") {
            //TODO: target support
            let next_space_or_end = args.chars().position(|c| c == ' ').unwrap_or_else(|| args.len());
            let path = &args[..next_space_or_end];
            let name = &args[next_space_or_end..];
            return Some(Command::GetPropValue { path: path.to_string(), name: name.to_string()});
        }
    }

    None
}

fn parse_command(cmd: &str) -> Option<Command> {
    if let Some(avm1_cmd) = smatch(cmd, "avm1") {
        parse_avm1_command(avm1_cmd)
    } else if let Some(do_cmd) = smatch(cmd, "do") {
        parse_do_command(do_cmd)
    } else if let Some(player_cmd) = smatch(cmd, "player") {
        parse_player_command(player_cmd)
    } else if let Some(_) = smatch(cmd, "help") {
        println!("Ruffle Debugger Help (Top level)");
        println!("");
        println!("Commands:");
        println!("avm1 - Query the state of the AVM1 interpreter");
        println!("do - Query a display object");
        println!("player - Control the state of the player");
        println!("help - View this message");
        println!("");
        println!("Try \"<command> help\" for more details");
        None
    } else {
        None
    }
}

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

    /// Break the execution of AVM1
    Avm1Break,

    /// Get the state of the AVM1 stack
    Avm1Stack,

    /// Execute the next instruction, stepping into function calls
    Avm1StepInto,
    
    /// Add a breakpoint that will break when `name` is called, either as a function or a method
    Avm1FunctionBreak { name: String },

    /// Remove any breakpoint on `name`
    Avm1FunctionBreakDelete { name: String},

    /// Continue execution
    Avm1Continue,

    /// Push a value onto the stack
    Avm1Push { val: DValue },

    /// Pop a value from the stack
    Avm1Pop,

    /// Get all the current breakpoints
    Avm1BreakpointsGet,

    /// Get the value of a avm1 variable
    Avm1VariableGet { path: String },

    /// Set the value of a avm1 variable
    Avm1VariableSet { path: String, value: DValue },

    /// Get the sub-properties of an avm1 variable
    Avm1SubpropGet { path: String },
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
                out.write(b"[").unwrap();
                out.write(select.as_bytes()).unwrap();
                out.write(b"]").unwrap();
            }
            out.write(b"> ").unwrap();
            out.flush().unwrap();

            let mut buf = [0u8; 4096];
            if let Ok(len) = std::io::stdin().read(&mut buf) {
                if len == 0 {
                    break;
                }

                let buf = &buf[..len];
                let s = String::from_utf8(buf.to_vec()).unwrap();

                let mut cmd = None;
                cmd = parse_command(&s.strip_suffix("\n").unwrap());

                if cmd.is_none() {
                if s.starts_with("reconnect") || s == "rc\n" {
                    cmd = Some(Command::Reconnect);
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
                } else if s == "\n" {
                    cmd = state.write().unwrap().last_cmd.take();
                } else {
                    println!("Unknown command");
                }
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

    fn send_msg(stream: &mut TcpStream, msg: DebugMessageIn) {
                    stream
                        .write(
                            serde_json::to_string(&msg)
                                .unwrap()
                                .as_bytes(),
                        )
                        .unwrap();
    }

    loop {
        if let Some(cmd) = queue.write().unwrap().pop() {
            match cmd {
                Command::Avm1SubpropGet { path } => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::GetSubprops { path } });
                }
                Command::Avm1VariableSet { path, value } => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::SetVariable { path, value } });
                }
                Command::Avm1VariableGet { path } => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::GetVariable { path } });
                }
                Command::Avm1BreakpointsGet => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::GetBreakpoints });
                }
                Command::Avm1Pop => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::Pop});
                }
                Command::Avm1Push { val } => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::Push {val }});
                }
                Command::Avm1Continue => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::Continue});
                }
                Command::Avm1FunctionBreakDelete { name }  => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::BreakFunctionDelete {name}});
                }
                Command::Avm1FunctionBreak { name }  => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::BreakFunction {name}});
                }
                Command::Avm1StepInto => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::StepInto});
                }
                Command::Avm1Stack => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::GetStack});
                }
                Command::Avm1Break => {
                    send_msg(&mut stream, DebugMessageIn::Avm1 { msg: Avm1Msg::Break});
                }
                Command::Pause => {
                    send_msg(&mut stream, DebugMessageIn::Player { msg: PlayerMsg::Pause});
                }
                Command::Play => {
                    send_msg(&mut stream, DebugMessageIn::Player { msg: PlayerMsg::Play});
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
                        DebugMessageOut::BreakpointList {bps} => {
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

//TODO: avm1 ops should be disabled unless in a bp context
