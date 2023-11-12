use std::{fs::File, io, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::from_reader;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SocketEvent {
    /// Wait for input data that matches this.
    Receive { expected: Vec<u8> },
    /// Send data to client.
    Send { payload: Vec<u8> },
    /// Expect client to disconnect.
    WaitForDisconnect,
    /// Disconnect the client.
    Disconnect,
}

impl SocketEvent {
    pub fn from_file<P>(path: P) -> Result<Vec<Self>, io::Error>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;

        Ok(from_reader(file)?)
    }

    pub fn from_reader<R>(reader: R) -> Result<Vec<Self>, io::Error>
    where
        R: io::Read,
    {
        Ok(from_reader(reader)?)
    }
}
