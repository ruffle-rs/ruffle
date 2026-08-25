use super::{
    ChunkSize, ChunkStreamId, MAX_INBOUND_CHUNK_SIZE, MAX_MESSAGE_LENGTH, RtmpMessage,
    WireTypeError,
};
use std::collections::HashMap;
use thiserror::Error;

const MAX_TIMESTAMP_24: u32 = 0x00ff_ffff;
const MAX_CHUNK_STREAMS: usize = 1_024;
const MAX_BUFFERED_INPUT: usize = MAX_MESSAGE_LENGTH as usize + MAX_INBOUND_CHUNK_SIZE as usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TimestampBasis {
    Absolute,
    Delta,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MessageHeader {
    timestamp: u32,
    timestamp_delta: u32,
    message_length: u32,
    message_type: u8,
    message_stream_id: u32,
    timestamp_basis: TimestampBasis,
    extended_timestamp: bool,
}

impl MessageHeader {
    fn extended_value(&self) -> u32 {
        match self.timestamp_basis {
            TimestampBasis::Absolute => self.timestamp,
            TimestampBasis::Delta => self.timestamp_delta,
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ReadState {
    previous: Option<MessageHeader>,
    partial: Option<PartialMessage>,
}

#[derive(Clone, Debug)]
struct PartialMessage {
    header: MessageHeader,
    payload: Vec<u8>,
}

#[derive(Debug)]
struct ParsedHeader {
    chunk_stream_id: ChunkStreamId,
    header: MessageHeader,
    bytes: usize,
    starts_message: bool,
}

/// Incrementally reassembles RTMP chunks into complete messages.
#[derive(Debug, Default)]
pub struct ChunkDecoder {
    chunk_size: ChunkSize,
    states: HashMap<ChunkStreamId, ReadState>,
    input: Vec<u8>,
    cursor: usize,
    partial_bytes: usize,
}

impl ChunkDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), ChunkError> {
        let buffered = self.input.len().saturating_sub(self.cursor);
        if buffered.saturating_add(bytes.len()) > MAX_BUFFERED_INPUT {
            return Err(ChunkError::InputLimit);
        }
        if self.cursor == self.input.len() {
            self.input.clear();
            self.cursor = 0;
        } else if self.cursor > 4_096 && self.cursor > self.input.len() / 2 {
            self.input.drain(..self.cursor);
            self.cursor = 0;
        }
        self.input.extend_from_slice(bytes);
        Ok(())
    }

    pub fn set_chunk_size(&mut self, size: ChunkSize) {
        self.chunk_size = size;
    }

    pub fn abort(&mut self, chunk_stream_id: ChunkStreamId) {
        if let Some(state) = self.states.get_mut(&chunk_stream_id)
            && let Some(partial) = state.partial.take()
        {
            self.partial_bytes = self.partial_bytes.saturating_sub(partial.payload.len());
        }
    }

    /// Decodes at most one complete message.
    ///
    /// The caller gets an opportunity to apply protocol-control messages before
    /// invoking this again on bytes already present in the input buffer.
    pub fn next_message(&mut self) -> Result<Option<RtmpMessage>, ChunkError> {
        loop {
            let available = &self.input[self.cursor..];
            let Some(parsed) = self.parse_header(available)? else {
                return Ok(None);
            };
            let already_received = self
                .states
                .get(&parsed.chunk_stream_id)
                .and_then(|state| state.partial.as_ref())
                .map_or(0, |partial| partial.payload.len());
            let remaining = parsed.header.message_length as usize - already_received;
            let fragment_length = remaining.min(self.chunk_size.get() as usize);
            let chunk_length = parsed.bytes.saturating_add(fragment_length);
            if available.len() < chunk_length {
                return Ok(None);
            }

            if !self.states.contains_key(&parsed.chunk_stream_id)
                && self.states.len() >= MAX_CHUNK_STREAMS
            {
                return Err(ChunkError::ChunkStreamLimit);
            }
            let fragment = available[parsed.bytes..chunk_length].to_vec();
            self.cursor += chunk_length;

            let state = self.states.entry(parsed.chunk_stream_id).or_default();
            if parsed.starts_message {
                if state.partial.is_some() {
                    return Err(ChunkError::MessageInterrupted(parsed.chunk_stream_id.get()));
                }
                state.previous = Some(parsed.header.clone());
                state.partial = Some(PartialMessage {
                    header: parsed.header,
                    payload: Vec::with_capacity(remaining),
                });
            }
            let partial = state
                .partial
                .as_mut()
                .ok_or(ChunkError::MissingPartial(parsed.chunk_stream_id.get()))?;
            partial.payload.extend_from_slice(&fragment);
            self.partial_bytes = self.partial_bytes.saturating_add(fragment.len());
            if self.partial_bytes > MAX_MESSAGE_LENGTH as usize {
                return Err(ChunkError::PartialPayloadLimit);
            }

            if partial.payload.len() == partial.header.message_length as usize {
                let partial = state
                    .partial
                    .take()
                    .expect("a completed partial message exists");
                self.partial_bytes = self.partial_bytes.saturating_sub(partial.payload.len());
                return Ok(Some(RtmpMessage {
                    timestamp: partial.header.timestamp,
                    message_type: partial.header.message_type,
                    message_stream_id: partial.header.message_stream_id,
                    payload: partial.payload,
                }));
            }
        }
    }

    fn parse_header(&self, bytes: &[u8]) -> Result<Option<ParsedHeader>, ChunkError> {
        let Some(&first) = bytes.first() else {
            return Ok(None);
        };
        let format = first >> 6;
        let marker = first & 0x3f;
        let (chunk_stream_id, basic_length) = match marker {
            0 => {
                if bytes.len() < 2 {
                    return Ok(None);
                }
                (64 + u32::from(bytes[1]), 2)
            }
            1 => {
                if bytes.len() < 3 {
                    return Ok(None);
                }
                (64 + u32::from(bytes[1]) + 256 * u32::from(bytes[2]), 3)
            }
            value => (u32::from(value), 1),
        };
        let chunk_stream_id = ChunkStreamId::try_from(chunk_stream_id)?;
        let state = self.states.get(&chunk_stream_id);
        let previous = state.and_then(|state| state.previous.as_ref());
        let partial = state.and_then(|state| state.partial.as_ref());
        let message_header_length = match format {
            0 => 11,
            1 => 7,
            2 => 3,
            3 => 0,
            _ => unreachable!("the RTMP format field is two bits"),
        };
        if bytes.len() < basic_length + message_header_length {
            return Ok(None);
        }
        let wire = &bytes[basic_length..];

        let (mut header, starts_message) = match format {
            0 => {
                let timestamp_field = read_u24(&wire[..3]);
                let message_length = read_u24(&wire[3..6]);
                let message_type = wire[6];
                let message_stream_id = u32::from_le_bytes(
                    wire[7..11]
                        .try_into()
                        .expect("the format-0 stream ID has four bytes"),
                );
                (
                    MessageHeader {
                        timestamp: timestamp_field,
                        timestamp_delta: 0,
                        message_length,
                        message_type,
                        message_stream_id,
                        timestamp_basis: TimestampBasis::Absolute,
                        extended_timestamp: timestamp_field == MAX_TIMESTAMP_24,
                    },
                    true,
                )
            }
            1 => {
                let previous =
                    previous.ok_or(ChunkError::MissingPrevious(chunk_stream_id.get()))?;
                let delta_field = read_u24(&wire[..3]);
                (
                    MessageHeader {
                        timestamp: previous.timestamp.wrapping_add(delta_field),
                        timestamp_delta: delta_field,
                        message_length: read_u24(&wire[3..6]),
                        message_type: wire[6],
                        message_stream_id: previous.message_stream_id,
                        timestamp_basis: TimestampBasis::Delta,
                        extended_timestamp: delta_field == MAX_TIMESTAMP_24,
                    },
                    true,
                )
            }
            2 => {
                let previous =
                    previous.ok_or(ChunkError::MissingPrevious(chunk_stream_id.get()))?;
                let delta_field = read_u24(&wire[..3]);
                (
                    MessageHeader {
                        timestamp: previous.timestamp.wrapping_add(delta_field),
                        timestamp_delta: delta_field,
                        message_length: previous.message_length,
                        message_type: previous.message_type,
                        message_stream_id: previous.message_stream_id,
                        timestamp_basis: TimestampBasis::Delta,
                        extended_timestamp: delta_field == MAX_TIMESTAMP_24,
                    },
                    true,
                )
            }
            3 => {
                if let Some(partial) = partial {
                    (partial.header.clone(), false)
                } else {
                    let previous =
                        previous.ok_or(ChunkError::MissingPrevious(chunk_stream_id.get()))?;
                    let mut header = previous.clone();
                    if header.timestamp_basis == TimestampBasis::Delta {
                        header.timestamp = header.timestamp.wrapping_add(header.timestamp_delta);
                    }
                    (header, true)
                }
            }
            _ => unreachable!("the RTMP format field is two bits"),
        };

        if starts_message && partial.is_some() {
            return Err(ChunkError::MessageInterrupted(chunk_stream_id.get()));
        }
        let mut header_length = basic_length + message_header_length;
        if header.extended_timestamp {
            if bytes.len() < header_length + 4 {
                return Ok(None);
            }
            let value = u32::from_be_bytes(
                bytes[header_length..header_length + 4]
                    .try_into()
                    .expect("the extended timestamp has four bytes"),
            );
            header_length += 4;
            if format == 0 {
                header.timestamp = value;
            } else if format == 1 || format == 2 {
                header.timestamp_delta = value;
                header.timestamp = previous
                    .expect("compressed headers require a previous header")
                    .timestamp
                    .wrapping_add(value);
            } else if value != header.extended_value() {
                return Err(ChunkError::ExtendedTimestampMismatch {
                    expected: header.extended_value(),
                    actual: value,
                });
            }
        }
        if header.message_length > MAX_MESSAGE_LENGTH {
            return Err(ChunkError::MessageLength(header.message_length));
        }

        Ok(Some(ParsedHeader {
            chunk_stream_id,
            header,
            bytes: header_length,
            starts_message,
        }))
    }
}

/// Encodes complete messages with per-chunk-stream header compression.
#[derive(Clone, Debug, Default)]
pub struct ChunkEncoder {
    chunk_size: ChunkSize,
    previous: HashMap<ChunkStreamId, MessageHeader>,
}

impl ChunkEncoder {
    pub fn set_chunk_size(&mut self, size: ChunkSize) {
        self.chunk_size = size;
    }

    pub fn clear_history(&mut self) {
        self.previous.clear();
    }

    pub fn encode(
        &mut self,
        chunk_stream_id: ChunkStreamId,
        message: &RtmpMessage,
    ) -> Result<Vec<u8>, ChunkError> {
        let message_length = u32::try_from(message.payload.len())
            .map_err(|_| ChunkError::MessageLength(u32::MAX))?;
        if message_length > MAX_MESSAGE_LENGTH {
            return Err(ChunkError::MessageLength(message_length));
        }
        let previous = self.previous.get(&chunk_stream_id);
        let delta = previous.map_or(0, |header| message.timestamp.wrapping_sub(header.timestamp));
        let format = match previous {
            None => 0,
            Some(header)
                if message.message_stream_id != header.message_stream_id
                    || message.timestamp < header.timestamp =>
            {
                0
            }
            Some(header)
                if message_length != header.message_length
                    || message.message_type != header.message_type =>
            {
                1
            }
            Some(header)
                if header.timestamp_basis == TimestampBasis::Delta
                    && delta == header.timestamp_delta =>
            {
                3
            }
            Some(_) => 2,
        };
        let timestamp_basis = if format == 0 {
            TimestampBasis::Absolute
        } else {
            TimestampBasis::Delta
        };
        let timestamp_value = if format == 0 {
            message.timestamp
        } else {
            delta
        };
        let extended = timestamp_value >= MAX_TIMESTAMP_24;
        let header = MessageHeader {
            timestamp: message.timestamp,
            timestamp_delta: if format == 0 { 0 } else { delta },
            message_length,
            message_type: message.message_type,
            message_stream_id: message.message_stream_id,
            timestamp_basis,
            extended_timestamp: extended,
        };
        let mut output = Vec::with_capacity(message.payload.len().saturating_add(32));
        write_basic_header(&mut output, format, chunk_stream_id);
        if format <= 2 {
            write_u24(
                &mut output,
                if extended {
                    MAX_TIMESTAMP_24
                } else {
                    timestamp_value
                },
            );
        }
        if format <= 1 {
            write_u24(&mut output, message_length);
            output.push(message.message_type);
        }
        if format == 0 {
            output.extend_from_slice(&message.message_stream_id.to_le_bytes());
        }
        if extended {
            output.extend_from_slice(&timestamp_value.to_be_bytes());
        }
        self.previous.insert(chunk_stream_id, header);

        let chunk_size = self.chunk_size.get() as usize;
        if message.payload.is_empty() {
            return Ok(output);
        }
        for (index, fragment) in message.payload.chunks(chunk_size).enumerate() {
            if index != 0 {
                write_basic_header(&mut output, 3, chunk_stream_id);
                if extended {
                    output.extend_from_slice(&message.timestamp.to_be_bytes());
                }
            }
            output.extend_from_slice(fragment);
        }
        Ok(output)
    }
}

fn read_u24(bytes: &[u8]) -> u32 {
    u32::from(bytes[0]) << 16 | u32::from(bytes[1]) << 8 | u32::from(bytes[2])
}

fn write_u24(output: &mut Vec<u8>, value: u32) {
    output.extend_from_slice(&value.to_be_bytes()[1..]);
}

fn write_basic_header(output: &mut Vec<u8>, format: u8, chunk_stream_id: ChunkStreamId) {
    let id = chunk_stream_id.get();
    let prefix = format << 6;
    match id {
        2..=63 => output.push(prefix | id as u8),
        64..=319 => {
            output.push(prefix);
            output.push((id - 64) as u8);
        }
        320..=65_599 => {
            let extended = id - 64;
            output.push(prefix | 1);
            output.push(extended as u8);
            output.push((extended >> 8) as u8);
        }
        _ => unreachable!("ChunkStreamId enforces its wire range"),
    }
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ChunkError {
    #[error(transparent)]
    WireType(#[from] WireTypeError),
    #[error("RTMP chunk input buffer exceeded its safety limit")]
    InputLimit,
    #[error("RTMP partial payloads exceeded their safety limit")]
    PartialPayloadLimit,
    #[error("too many RTMP chunk streams")]
    ChunkStreamLimit,
    #[error("chunk stream {0} has no previous message header")]
    MissingPrevious(u32),
    #[error("chunk stream {0} has no partial message")]
    MissingPartial(u32),
    #[error("a new message interrupted chunk stream {0} before Abort")]
    MessageInterrupted(u32),
    #[error("RTMP message length {0} exceeds the 24-bit wire limit")]
    MessageLength(u32),
    #[error("extended timestamp mismatch: expected {expected}, got {actual}")]
    ExtendedTimestampMismatch { expected: u32, actual: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(message: RtmpMessage, chunk_stream_id: u32, chunk_size: u32) {
        let chunk_size = ChunkSize::try_from(chunk_size).expect("test chunk size is valid");
        let mut encoder = ChunkEncoder::default();
        encoder.set_chunk_size(chunk_size);
        let wire = encoder
            .encode(
                ChunkStreamId::try_from(chunk_stream_id).expect("test CSID is valid"),
                &message,
            )
            .expect("test message encodes");
        let mut decoder = ChunkDecoder::default();
        decoder.set_chunk_size(chunk_size);
        for byte in wire {
            decoder.push(&[byte]).expect("one-byte fragment fits");
        }
        assert_eq!(
            decoder.next_message().expect("test message decodes"),
            Some(message)
        );
        assert_eq!(decoder.next_message().expect("decoder remains valid"), None);
    }

    #[test]
    fn round_trips_chunk_stream_boundaries_and_fragmentation() {
        for chunk_stream_id in [2, 63, 64, 319, 320, 65_599] {
            round_trip(
                RtmpMessage {
                    timestamp: 7,
                    message_type: 20,
                    message_stream_id: 0x4433_2211,
                    payload: (0..307).map(|value| value as u8).collect(),
                },
                chunk_stream_id,
                128,
            );
        }
    }

    #[test]
    fn round_trips_extended_timestamp_on_every_continuation() {
        round_trip(
            RtmpMessage {
                timestamp: MAX_TIMESTAMP_24,
                message_type: 20,
                message_stream_id: 0,
                payload: vec![0xaa; 40],
            },
            3,
            7,
        );
    }

    #[test]
    fn encoder_uses_128_128_51_payload_fragments() {
        let message = RtmpMessage {
            timestamp: 0,
            message_type: 20,
            message_stream_id: 0,
            payload: vec![0; 307],
        };
        let wire = ChunkEncoder::default()
            .encode(ChunkStreamId::COMMAND, &message)
            .expect("test message encodes");
        // One 12-byte format-0 header and two one-byte format-3 headers.
        assert_eq!(wire.len(), 12 + 128 + 1 + 128 + 1 + 51);
    }

    #[test]
    fn encoder_compresses_repeated_message_headers_per_chunk_stream() {
        let mut encoder = ChunkEncoder::default();
        let stream = ChunkStreamId::COMMAND;
        let message = |timestamp, payload: &[u8]| RtmpMessage {
            timestamp,
            message_type: 20,
            message_stream_id: 0,
            payload: payload.to_vec(),
        };

        let first = encoder
            .encode(stream, &message(100, &[1, 2]))
            .expect("format-0 message encodes");
        let second = encoder
            .encode(stream, &message(110, &[3, 4]))
            .expect("format-2 message encodes");
        let third = encoder
            .encode(stream, &message(120, &[5, 6]))
            .expect("format-3 message encodes");
        let fourth = encoder
            .encode(stream, &message(130, &[7, 8, 9]))
            .expect("format-1 message encodes");

        assert_eq!(first[0] >> 6, 0);
        assert_eq!(second[0] >> 6, 2);
        assert_eq!(third[0] >> 6, 3);
        assert_eq!(fourth[0] >> 6, 1);

        let mut decoder = ChunkDecoder::default();
        for (wire, expected) in [
            (first, message(100, &[1, 2])),
            (second, message(110, &[3, 4])),
            (third, message(120, &[5, 6])),
            (fourth, message(130, &[7, 8, 9])),
        ] {
            decoder.push(&wire).expect("compressed message fits");
            assert_eq!(
                decoder.next_message().expect("message decodes"),
                Some(expected)
            );
        }
    }
}
