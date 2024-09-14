use anyhow::Error;
use clap::Parser;
use ruffle_socket_format::SocketEvent;
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

static POLICY: &[u8] = b"<?xml version=\"1.0\"?>
<!DOCTYPE cross-domain-policy SYSTEM \"http://www.adobe.com/xml/dtds/cross-domain-policy.dtd\">
<cross-domain-policy>
<allow-access-from domain=\"*\" to-ports=\"*\"/>
</cross-domain-policy>\0";

#[derive(Parser, Debug)]
struct Opt {
    /// Path to a `socket.json` file.
    #[clap(name = "FILE")]
    file_path: PathBuf,
}

fn main() -> Result<(), Error> {
    let opt = Opt::parse();

    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .finish();
    // Ignore error if it's already been set
    let _ = tracing::subscriber::set_global_default(subscriber);

    let events = SocketEvent::from_file(opt.file_path)?;
    let event_count = events.len();

    let listener = TcpListener::bind("0.0.0.0:8001")?;
    tracing::info!("Listening on {}", listener.local_addr()?);
    let (mut stream, addr) = listener.accept()?;
    tracing::info!("Incoming connection from {}", addr);

    // Handle socket policy stuff. (Required as Flash Player wont want to connect otherwise.)
    let mut buffer = [0; 4096];
    let _ = stream.read(&mut buffer);
    stream.write_all(POLICY)?;
    tracing::info!("Policy sent successfully!");

    // Now we listen again as flash reopens socket connection.
    let (mut stream, addr) = listener.accept()?;
    tracing::info!("Incoming connection from {}", addr);

    for (index, event) in events.into_iter().enumerate() {
        tracing::info!("Running step {}/{}", index + 1, event_count);

        match event {
            SocketEvent::Receive { expected } => {
                let mut output = vec![];

                loop {
                    let mut buffer = [0; 4096];

                    match stream.read(&mut buffer) {
                        Err(_) | Ok(0) => {
                            tracing::error!("Expected data, but socket was closed.");
                            return Ok(());
                        }
                        Ok(read) => {
                            if read == 4096 {
                                output.extend(buffer);
                            } else {
                                let data = buffer.into_iter().take(read).collect::<Vec<_>>();
                                output.extend(data);
                                break;
                            }
                        }
                    }
                }

                if output != expected {
                    tracing::error!(
                        "Received data did not match expected data\nExpected: {:?}\nActual: {:?}",
                        expected,
                        output
                    );
                }
            }
            SocketEvent::Send { mut payload } => {
                while !payload.is_empty() {
                    match stream.write(&payload) {
                        Err(_) | Ok(0) => {
                            tracing::error!("Socket was closed in middle of writing.");
                            return Ok(());
                        }
                        Ok(written) => {
                            let _ = payload.drain(..written);
                        }
                    }
                }
            }
            #[allow(clippy::unused_io_amount)]
            SocketEvent::WaitForDisconnect => {
                let mut buffer = [0; 4096];

                match stream.read(&mut buffer) {
                    Err(_) | Ok(0) => {
                        tracing::info!("Client has closed the connection!");
                        return Ok(());
                    }
                    Ok(_) => {
                        tracing::error!(
                            "Expected client to close connection, but data was sent instead."
                        );
                    }
                }
            }
            SocketEvent::Disconnect => {
                tracing::info!("Disconnecting client.");
                drop(stream);
                break;
            }
        }
    }

    Ok(())
}
