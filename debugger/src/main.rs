use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLock, Arc};
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write};
use std::time::Duration;

use ruffle_core::player::{DebugMessageOut, DebugMessageIn};

#[derive(Debug, Clone)]
pub enum Command {
    Pause,
    Play,
    Reconnect,
    GetVar { path: String },
}

fn stdin_thread(queue: Arc<RwLock<Vec<Command>>>, flag: Arc<AtomicBool>) -> thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut last_cmd: Option<Command> = None;

        while flag.load(Ordering::SeqCst) {
            let mut buf = [0u8; 4096];
            if let Ok(len) = std::io::stdin().read(&mut buf) {
                if len == 0 {
                    break;
                }

                let buf = &buf[..len];
                let s = String::from_utf8(buf.to_vec()).unwrap();

                let mut cmd = None;

                if s.starts_with("gv") {
                    let var_name = s.strip_suffix("\n").unwrap().split(" ").skip(1).next().unwrap();
                    cmd = Some(Command::GetVar { path: var_name.to_string() })
                } else if s.starts_with("pause") {
                    cmd = Some(Command::Pause);
                } else if s.starts_with("play") || s == "c\n" {
                    cmd = Some(Command::Play);
                } else if s.starts_with("reconnect") || s == "rc\n" {
                    cmd = Some(Command::Reconnect);
                } else if s == "\n" {
                    cmd = last_cmd.take();
                }

                if let Some(cmd) = cmd {
                    println!("Got cmd {:?}", cmd);
                    queue.write().unwrap().push(cmd.clone());
                    last_cmd = Some(cmd);
                }
            }
        }
    })
}

fn handle_client(mut stream: TcpStream) {
    stream.set_read_timeout(Some(Duration::from_millis(100))).unwrap();
    let queue = Arc::new(RwLock::new(Vec::new()));
    let flag = Arc::new(AtomicBool::new(true));
    let input_thread = stdin_thread(Arc::clone(&queue), Arc::clone(&flag));

    loop {
        if let Some(cmd) = queue.write().unwrap().pop() {
            match cmd {
                Command::Pause => {
                    stream.write(serde_json::to_string(&DebugMessageIn::Pause).unwrap().as_bytes()).unwrap();
                },
                Command::Play => {
                    stream.write(serde_json::to_string(&DebugMessageIn::Play).unwrap().as_bytes()).unwrap();
                },
                Command::Reconnect => {
                    break;
                },
                Command::GetVar { path } => {
                    stream.write(serde_json::to_string(&DebugMessageIn::GetVar { path: path }).unwrap().as_bytes()).unwrap();
                }
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
                    }
                }
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
