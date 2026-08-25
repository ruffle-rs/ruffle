//! Sans-I/O implementation of the parts of RTMP used by `NetConnection` RPC.
//!
//! This module owns RTMP wire framing. It deliberately has no knowledge of
//! sockets, WebSockets, AVM objects, or application-specific method names.

mod chunk;
mod command;
mod handshake;
mod message;
mod session;

pub use chunk::{ChunkDecoder, ChunkEncoder, ChunkError};
pub use command::{Command, CommandError};
pub use handshake::{ClientHandshake, HandshakeError};
pub use message::{
    BandwidthLimit, MessageError, ProtocolMessage, UserControlEvent, decode_message,
};
pub use session::{RtmpSession, SessionAction};

use std::num::NonZeroU32;
use thiserror::Error;

/// The initial maximum chunk payload in each RTMP direction.
pub const DEFAULT_CHUNK_SIZE: u32 = 128;

/// Largest value representable by the RTMP 24-bit message length field.
pub const MAX_MESSAGE_LENGTH: u32 = 0x00ff_ffff;

/// A defensive per-chunk allocation limit for untrusted network input.
pub const MAX_INBOUND_CHUNK_SIZE: u32 = 1024 * 1024;

/// Identifies an RTMP chunk stream, independently of an RTMP message stream.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChunkStreamId(u32);

impl ChunkStreamId {
    pub const CONTROL: Self = Self(2);
    pub const COMMAND: Self = Self(3);

    pub fn get(self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for ChunkStreamId {
    type Error = WireTypeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if (2..=65_599).contains(&value) {
            Ok(Self(value))
        } else {
            Err(WireTypeError::ChunkStreamId(value))
        }
    }
}

/// Maximum payload carried by one RTMP chunk.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChunkSize(NonZeroU32);

impl ChunkSize {
    pub fn get(self) -> u32 {
        self.0.get()
    }

    pub fn inbound(value: u32) -> Result<Self, WireTypeError> {
        let size = Self::try_from(value)?;
        if value > MAX_INBOUND_CHUNK_SIZE {
            return Err(WireTypeError::InboundChunkSize(value));
        }
        Ok(size)
    }
}

impl Default for ChunkSize {
    fn default() -> Self {
        Self(NonZeroU32::new(DEFAULT_CHUNK_SIZE).expect("the default chunk size is non-zero"))
    }
}

impl TryFrom<u32> for ChunkSize {
    type Error = WireTypeError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value & 0x8000_0000 != 0 {
            return Err(WireTypeError::ChunkSize(value));
        }
        NonZeroU32::new(value)
            .map(Self)
            .ok_or(WireTypeError::ChunkSize(value))
    }
}

/// Integral RTMP RPC transaction identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TransactionId(u32);

impl TransactionId {
    pub const NOTIFICATION: Self = Self(0);
    pub const CONNECT: Self = Self(1);

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn from_wire(value: f64) -> Result<Self, WireTypeError> {
        if value.is_finite() && value.fract() == 0.0 && (0.0..=u32::MAX as f64).contains(&value) {
            Ok(Self(value as u32))
        } else {
            Err(WireTypeError::TransactionId(value))
        }
    }
}

impl From<u32> for TransactionId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// A complete RTMP message after chunk reassembly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RtmpMessage {
    pub timestamp: u32,
    pub message_type: u8,
    pub message_stream_id: u32,
    pub payload: Vec<u8>,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum WireTypeError {
    #[error("invalid RTMP chunk stream ID {0}")]
    ChunkStreamId(u32),
    #[error("invalid RTMP chunk size {0}")]
    ChunkSize(u32),
    #[error("inbound RTMP chunk size {0} exceeds the player safety limit")]
    InboundChunkSize(u32),
    #[error("RTMP transaction ID is not an unsigned integer: {0}")]
    TransactionId(f64),
}
