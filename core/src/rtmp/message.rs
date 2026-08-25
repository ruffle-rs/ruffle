use super::{ChunkSize, ChunkStreamId, Command, CommandError, RtmpMessage, WireTypeError};
use thiserror::Error;

pub const TYPE_SET_CHUNK_SIZE: u8 = 1;
pub const TYPE_ABORT: u8 = 2;
pub const TYPE_ACKNOWLEDGEMENT: u8 = 3;
pub const TYPE_USER_CONTROL: u8 = 4;
pub const TYPE_WINDOW_ACKNOWLEDGEMENT_SIZE: u8 = 5;
pub const TYPE_SET_PEER_BANDWIDTH: u8 = 6;
pub const TYPE_COMMAND_AMF0: u8 = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BandwidthLimit {
    Hard,
    Soft,
    Dynamic,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UserControlEvent {
    StreamBegin(u32),
    StreamEof(u32),
    StreamDry(u32),
    SetBufferLength { stream_id: u32, milliseconds: u32 },
    StreamIsRecorded(u32),
    PingRequest(u32),
    PingResponse(u32),
    BufferEmpty(u32),
    BufferReady(u32),
    Other { event_type: u16, payload: Vec<u8> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ProtocolMessage {
    SetChunkSize(ChunkSize),
    Abort(ChunkStreamId),
    Acknowledgement(u32),
    UserControl(UserControlEvent),
    WindowAcknowledgementSize(u32),
    SetPeerBandwidth { window: u32, limit: BandwidthLimit },
    Command(Command),
    Unknown(RtmpMessage),
}

pub fn decode_message(message: RtmpMessage) -> Result<ProtocolMessage, MessageError> {
    let payload = &message.payload;
    match message.message_type {
        TYPE_SET_CHUNK_SIZE => {
            expect_len(payload, 4, message.message_type)?;
            Ok(ProtocolMessage::SetChunkSize(ChunkSize::inbound(
                read_u32(payload),
            )?))
        }
        TYPE_ABORT => {
            expect_len(payload, 4, message.message_type)?;
            Ok(ProtocolMessage::Abort(ChunkStreamId::try_from(read_u32(
                payload,
            ))?))
        }
        TYPE_ACKNOWLEDGEMENT => {
            expect_len(payload, 4, message.message_type)?;
            Ok(ProtocolMessage::Acknowledgement(read_u32(payload)))
        }
        TYPE_USER_CONTROL => Ok(ProtocolMessage::UserControl(decode_user_control(payload)?)),
        TYPE_WINDOW_ACKNOWLEDGEMENT_SIZE => {
            expect_len(payload, 4, message.message_type)?;
            let window = read_u32(payload);
            if window == 0 {
                return Err(MessageError::ZeroAcknowledgementWindow);
            }
            Ok(ProtocolMessage::WindowAcknowledgementSize(window))
        }
        TYPE_SET_PEER_BANDWIDTH => {
            expect_len(payload, 5, message.message_type)?;
            let limit = match payload[4] {
                0 => BandwidthLimit::Hard,
                1 => BandwidthLimit::Soft,
                2 => BandwidthLimit::Dynamic,
                value => return Err(MessageError::BandwidthLimit(value)),
            };
            Ok(ProtocolMessage::SetPeerBandwidth {
                window: read_u32(payload),
                limit,
            })
        }
        TYPE_COMMAND_AMF0 => Ok(ProtocolMessage::Command(Command::decode(payload)?)),
        _ => Ok(ProtocolMessage::Unknown(message)),
    }
}

impl ProtocolMessage {
    pub fn encode(&self) -> Result<RtmpMessage, MessageError> {
        let (message_type, payload) = match self {
            Self::SetChunkSize(size) => (TYPE_SET_CHUNK_SIZE, size.get().to_be_bytes().to_vec()),
            Self::Abort(chunk_stream_id) => {
                (TYPE_ABORT, chunk_stream_id.get().to_be_bytes().to_vec())
            }
            Self::Acknowledgement(sequence) => {
                (TYPE_ACKNOWLEDGEMENT, sequence.to_be_bytes().to_vec())
            }
            Self::UserControl(event) => (TYPE_USER_CONTROL, encode_user_control(event)),
            Self::WindowAcknowledgementSize(window) => (
                TYPE_WINDOW_ACKNOWLEDGEMENT_SIZE,
                window.to_be_bytes().to_vec(),
            ),
            Self::SetPeerBandwidth { window, limit } => {
                let mut payload = window.to_be_bytes().to_vec();
                payload.push(match limit {
                    BandwidthLimit::Hard => 0,
                    BandwidthLimit::Soft => 1,
                    BandwidthLimit::Dynamic => 2,
                });
                (TYPE_SET_PEER_BANDWIDTH, payload)
            }
            Self::Command(command) => (TYPE_COMMAND_AMF0, command.encode()?),
            Self::Unknown(message) => return Ok(message.clone()),
        };
        Ok(RtmpMessage {
            timestamp: 0,
            message_type,
            message_stream_id: 0,
            payload,
        })
    }
}

fn decode_user_control(payload: &[u8]) -> Result<UserControlEvent, MessageError> {
    if payload.len() < 2 {
        return Err(MessageError::Length {
            message_type: TYPE_USER_CONTROL,
            expected: 2,
            actual: payload.len(),
        });
    }
    let event_type = u16::from_be_bytes([payload[0], payload[1]]);
    let data = &payload[2..];
    let stream = || -> Result<u32, MessageError> {
        expect_len(payload, 6, TYPE_USER_CONTROL)?;
        Ok(read_u32(data))
    };
    Ok(match event_type {
        0 => UserControlEvent::StreamBegin(stream()?),
        1 => UserControlEvent::StreamEof(stream()?),
        2 => UserControlEvent::StreamDry(stream()?),
        3 => {
            expect_len(payload, 10, TYPE_USER_CONTROL)?;
            UserControlEvent::SetBufferLength {
                stream_id: read_u32(data),
                milliseconds: read_u32(&data[4..]),
            }
        }
        4 => UserControlEvent::StreamIsRecorded(stream()?),
        6 => UserControlEvent::PingRequest(stream()?),
        7 => UserControlEvent::PingResponse(stream()?),
        31 => UserControlEvent::BufferEmpty(stream()?),
        32 => UserControlEvent::BufferReady(stream()?),
        _ => UserControlEvent::Other {
            event_type,
            payload: data.to_vec(),
        },
    })
}

fn encode_user_control(event: &UserControlEvent) -> Vec<u8> {
    let (event_type, first, second): (u16, u32, Option<u32>) = match event {
        UserControlEvent::StreamBegin(value) => (0, *value, None),
        UserControlEvent::StreamEof(value) => (1, *value, None),
        UserControlEvent::StreamDry(value) => (2, *value, None),
        UserControlEvent::SetBufferLength {
            stream_id,
            milliseconds,
        } => (3, *stream_id, Some(*milliseconds)),
        UserControlEvent::StreamIsRecorded(value) => (4, *value, None),
        UserControlEvent::PingRequest(value) => (6, *value, None),
        UserControlEvent::PingResponse(value) => (7, *value, None),
        UserControlEvent::BufferEmpty(value) => (31, *value, None),
        UserControlEvent::BufferReady(value) => (32, *value, None),
        UserControlEvent::Other {
            event_type,
            payload,
        } => {
            let mut output = event_type.to_be_bytes().to_vec();
            output.extend_from_slice(payload);
            return output;
        }
    };
    let mut output = event_type.to_be_bytes().to_vec();
    output.extend_from_slice(&first.to_be_bytes());
    if let Some(second) = second {
        output.extend_from_slice(&second.to_be_bytes());
    }
    output
}

fn expect_len(payload: &[u8], expected: usize, message_type: u8) -> Result<(), MessageError> {
    if payload.len() == expected {
        Ok(())
    } else {
        Err(MessageError::Length {
            message_type,
            expected,
            actual: payload.len(),
        })
    }
}

fn read_u32(payload: &[u8]) -> u32 {
    u32::from_be_bytes(
        payload[..4]
            .try_into()
            .expect("the caller validates four-byte protocol fields"),
    )
}

#[derive(Debug, Error)]
pub enum MessageError {
    #[error("RTMP type {message_type} needs {expected} payload bytes, got {actual}")]
    Length {
        message_type: u8,
        expected: usize,
        actual: usize,
    },
    #[error("RTMP acknowledgement window must be non-zero")]
    ZeroAcknowledgementWindow,
    #[error("invalid RTMP bandwidth limit type {0}")]
    BandwidthLimit(u8),
    #[error(transparent)]
    WireType(#[from] WireTypeError),
    #[error(transparent)]
    Command(#[from] CommandError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_response_echoes_the_exact_timestamp() {
        let request = RtmpMessage {
            timestamp: 0,
            message_type: TYPE_USER_CONTROL,
            message_stream_id: 0,
            payload: vec![0, 6, 1, 2, 3, 4],
        };
        assert_eq!(
            decode_message(request).expect("ping request decodes"),
            ProtocolMessage::UserControl(UserControlEvent::PingRequest(0x0102_0304))
        );
        assert_eq!(
            ProtocolMessage::UserControl(UserControlEvent::PingResponse(0x0102_0304))
                .encode()
                .expect("ping response encodes")
                .payload,
            [0, 7, 1, 2, 3, 4]
        );
    }
}
