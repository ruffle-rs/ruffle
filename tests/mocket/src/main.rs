use anyhow::{Error, anyhow};
use clap::Parser;
use ruffle_socket_format::SocketEvent;
use rustls::ServerConfig;
use rustls_pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;
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

    /// Enable TLS mode for SecureSocket testing.
    #[clap(long)]
    tls: bool,

    /// Path to a PEM-encoded certificate file (required with --tls).
    #[clap(long, requires = "tls")]
    cert: Option<PathBuf>,

    /// Path to a PEM-encoded private key file (required with --tls).
    #[clap(long, requires = "tls")]
    key: Option<PathBuf>,
}

fn load_tls_config(cert_path: &Path, key_path: &Path) -> Result<Arc<ServerConfig>, Error> {
    let certs: Vec<CertificateDer<'static>> =
        CertificateDer::pem_file_iter(cert_path)?.collect::<Result<Vec<_>, _>>()?;

    if certs.is_empty() {
        return Err(anyhow!("No certificates found in {:?}", cert_path));
    }

    let key = PrivateKeyDer::from_pem_file(key_path)?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    Ok(Arc::new(config))
}

/// Run the event loop over a stream that supports both `Read` and `Write`.
fn run_events(stream: &mut (impl Read + Write), events: Vec<SocketEvent>) -> Result<(), Error> {
    let event_count = events.len();

    for (index, event) in events.into_iter().enumerate() {
        tracing::info!("Running step {}/{}", index + 1, event_count);

        match event {
            SocketEvent::Receive { expected } => {
                let mut output = vec![0u8; expected.len()];
                stream.read_exact(&mut output)?;

                if output != expected {
                    tracing::error!(
                        "Received data did not match expected data\nExpected: {:?}\nActual: {:?}",
                        expected,
                        output
                    );
                }
            }
            SocketEvent::Send { payload } => {
                stream.write_all(&payload)?;
            }
            SocketEvent::WaitForDisconnect => {
                let mut buffer = [0; 1];

                match stream.read_exact(&mut buffer) {
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

fn main() -> Result<(), Error> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

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

    let listener = TcpListener::bind("0.0.0.0:8001")?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    if opt.tls {
        let cert_path = opt
            .cert
            .ok_or_else(|| anyhow!("--cert is required when --tls is enabled"))?;
        let key_path = opt
            .key
            .ok_or_else(|| anyhow!("--key is required when --tls is enabled"))?;

        let tls_config = load_tls_config(&cert_path, &key_path)?;
        tracing::info!("TLS enabled (SecureSocket mode, no policy exchange)");

        let (stream, addr) = listener.accept()?;
        tracing::info!("Incoming connection from {}", addr);

        let conn = rustls::ServerConnection::new(tls_config)?;
        let mut tls_stream = rustls::StreamOwned::new(conn, stream);

        // Reuse common event loop for TLS streams.
        run_events(&mut tls_stream, events)?;
    } else {
        let (stream, addr) = listener.accept()?;
        tracing::info!("Incoming connection from {}", addr);

        let mut writer = stream.try_clone()?;
        let mut reader = BufReader::new(stream);

        // Handle socket policy (required for plain Socket connections).
        let mut buffer: Vec<u8> = Vec::new();
        reader.read_until(0, &mut buffer)?;
        if &buffer[..] != POLICY_REQUEST {
            return Err(anyhow!("No policy request, received: {buffer:?}"));
        }
        writer.write_all(POLICY)?;
        tracing::info!("Policy sent successfully!");

        // Flash reopens socket connection after policy exchange.
        let (mut stream, addr) = listener.accept()?;
        tracing::info!("Incoming connection from {}", addr);

        run_events(&mut stream, events)?;
    }

    Ok(())
}
