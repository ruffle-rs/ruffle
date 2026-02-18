use anyhow::{Error, anyhow};
use clap::Parser;
use ruffle_socket_format::SocketEvent;
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

static POLICY_REQUEST: &[u8] = b"<policy-file-request/>\0";

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
    let (stream, addr) = listener.accept()?;
    tracing::info!("Incoming connection from {}", addr);

    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    // Handle socket policy stuff. (Required as Flash Player wont want to connect otherwise.)
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_until(0, &mut buffer)?;
    if &buffer[..] != POLICY_REQUEST {
        return Err(anyhow!("No policy request, received: {buffer:?}"));
    }
    writer.write_all(POLICY)?;
    tracing::info!("Policy sent successfully!");

    // Now we listen again as flash reopens socket connection.
    let (stream, addr) = listener.accept()?;
    tracing::info!("Incoming connection from {}", addr);

    let mut writer = stream.try_clone()?;
    let mut reader = BufReader::new(stream);

    for (index, event) in events.into_iter().enumerate() {
        tracing::info!("Running step {}/{}", index + 1, event_count);

        match event {
            SocketEvent::Receive { expected } => {
                let mut output: Vec<u8> = Vec::new();
                reader.read_until(0, &mut output)?;

                if output != expected {
                    tracing::error!(
                        "Received data did not match expected data\nExpected: {:?}\nActual: {:?}",
                        expected,
                        output
                    );
                }
            }
            SocketEvent::Send { payload } => {
                writer.write_all(&payload)?;
            }
            SocketEvent::WaitForDisconnect => {
                let mut buffer = [0; 1];

                match reader.read_exact(&mut buffer) {
                    Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                        tracing::info!("Client has closed the connection!");
                        return Ok(());
                    }
                    e @ Err(_) => e?,
                    Ok(_) => {
                        tracing::error!(
                            "Expected client to close connection, but data was sent instead."
                        );
                    }
                }
            }
            SocketEvent::Disconnect => {
                tracing::info!("Disconnecting client.");
                break;
            }
        }
    }

    Ok(())
}
