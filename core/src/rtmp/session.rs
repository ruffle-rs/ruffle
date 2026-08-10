use super::{
    BandwidthLimit, ChunkDecoder, ChunkEncoder, ChunkError, ChunkStreamId, ClientHandshake,
    Command, HandshakeError, MessageError, ProtocolMessage, UserControlEvent, decode_message,
};
use thiserror::Error;

#[derive(Debug)]
enum State {
    Handshaking(Box<ClientHandshake>),
    Active,
}

/// A side effect produced by the sans-I/O RTMP session.
#[derive(Debug, PartialEq)]
pub enum SessionAction {
    Outbound(Vec<u8>),
    HandshakeComplete,
    Command(Command),
    Message(ProtocolMessage),
}

/// Client RTMP session over an arbitrary ordered byte stream.
#[derive(Debug)]
pub struct RtmpSession {
    state: State,
    decoder: ChunkDecoder,
    encoder: ChunkEncoder,
    received_bytes: u64,
    acknowledgement_window: Option<u32>,
    last_acknowledgement: u64,
    sent_acknowledgement_window: Option<u32>,
    peer_bandwidth: Option<(u32, BandwidthLimit)>,
}

impl RtmpSession {
    pub fn new(time: u32, c1_random: [u8; 1_528], c2_random: [u8; 1_504]) -> (Self, Vec<u8>) {
        let (handshake, initial) = ClientHandshake::new(time, c1_random, c2_random);
        (
            Self {
                state: State::Handshaking(Box::new(handshake)),
                decoder: ChunkDecoder::default(),
                encoder: ChunkEncoder::default(),
                received_bytes: 0,
                acknowledgement_window: None,
                last_acknowledgement: 0,
                sent_acknowledgement_window: None,
                peer_bandwidth: None,
            },
            initial,
        )
    }

    pub fn receive(
        &mut self,
        bytes: &[u8],
        receive_time: u32,
    ) -> Result<Vec<SessionAction>, SessionError> {
        let mut actions = Vec::new();
        let mut handshake_remainder = None;
        let mut rtmp_bytes = bytes;
        if let State::Handshaking(handshake) = &mut self.state {
            let output = handshake.feed(bytes, receive_time)?;
            if !output.outbound.is_empty() {
                actions.push(SessionAction::Outbound(output.outbound));
            }
            if !output.complete {
                return Ok(actions);
            }
            self.state = State::Active;
            actions.push(SessionAction::HandshakeComplete);
            handshake_remainder = Some(output.remainder);
        }

        if let Some(remainder) = &handshake_remainder {
            rtmp_bytes = remainder;
        }

        self.received_bytes = self.received_bytes.saturating_add(rtmp_bytes.len() as u64);
        self.decoder.push(rtmp_bytes)?;
        self.emit_due_acknowledgements(&mut actions)?;
        while let Some(message) = self.decoder.next_message()? {
            let message = decode_message(message)?;
            self.handle_message(message, &mut actions)?;
        }
        Ok(actions)
    }

    pub fn send_command(
        &mut self,
        command: &Command,
        timestamp: u32,
    ) -> Result<Vec<u8>, SessionError> {
        let mut message = ProtocolMessage::Command(command.clone()).encode()?;
        message.timestamp = timestamp;
        Ok(self.encoder.encode(ChunkStreamId::COMMAND, &message)?)
    }

    pub fn clear_outbound_chunk_history(&mut self) {
        self.encoder.clear_history();
    }

    fn handle_message(
        &mut self,
        message: ProtocolMessage,
        actions: &mut Vec<SessionAction>,
    ) -> Result<(), SessionError> {
        match message {
            ProtocolMessage::SetChunkSize(size) => self.decoder.set_chunk_size(size),
            ProtocolMessage::Abort(chunk_stream_id) => self.decoder.abort(chunk_stream_id),
            ProtocolMessage::WindowAcknowledgementSize(window) => {
                self.acknowledgement_window = Some(window);
                self.emit_due_acknowledgements(actions)?;
            }
            ProtocolMessage::SetPeerBandwidth { window, limit } => {
                self.apply_peer_bandwidth(window, limit);
                if self.sent_acknowledgement_window.is_none() {
                    actions.push(SessionAction::Outbound(self.encode(
                        ProtocolMessage::WindowAcknowledgementSize(window),
                        ChunkStreamId::CONTROL,
                    )?));
                    self.sent_acknowledgement_window = Some(window);
                }
            }
            ProtocolMessage::UserControl(UserControlEvent::PingRequest(timestamp)) => {
                actions.push(SessionAction::Outbound(self.encode(
                    ProtocolMessage::UserControl(UserControlEvent::PingResponse(timestamp)),
                    ChunkStreamId::CONTROL,
                )?));
            }
            ProtocolMessage::Command(command) => actions.push(SessionAction::Command(command)),
            other => actions.push(SessionAction::Message(other)),
        }
        Ok(())
    }

    fn emit_due_acknowledgements(
        &mut self,
        actions: &mut Vec<SessionAction>,
    ) -> Result<(), SessionError> {
        let Some(window) = self.acknowledgement_window else {
            return Ok(());
        };
        let window = u64::from(window);
        while self
            .received_bytes
            .saturating_sub(self.last_acknowledgement)
            >= window
        {
            self.last_acknowledgement = self.last_acknowledgement.saturating_add(window);
            actions.push(SessionAction::Outbound(self.encode(
                ProtocolMessage::Acknowledgement(self.last_acknowledgement as u32),
                ChunkStreamId::CONTROL,
            )?));
        }
        Ok(())
    }

    fn apply_peer_bandwidth(&mut self, window: u32, limit: BandwidthLimit) {
        let next = match (self.peer_bandwidth, limit) {
            (_, BandwidthLimit::Hard) => Some((window, BandwidthLimit::Hard)),
            (Some((current, kind)), BandwidthLimit::Soft) => Some((current.min(window), kind)),
            (None, BandwidthLimit::Soft) => Some((window, BandwidthLimit::Soft)),
            (Some((_, BandwidthLimit::Hard)), BandwidthLimit::Dynamic) => {
                Some((window, BandwidthLimit::Hard))
            }
            (current, BandwidthLimit::Dynamic) => current,
        };
        self.peer_bandwidth = next;
    }

    fn encode(
        &mut self,
        message: ProtocolMessage,
        chunk_stream_id: ChunkStreamId,
    ) -> Result<Vec<u8>, SessionError> {
        let set_chunk_size = match &message {
            ProtocolMessage::SetChunkSize(size) => Some(*size),
            _ => None,
        };
        let encoded = self.encoder.encode(chunk_stream_id, &message.encode()?)?;
        if let Some(size) = set_chunk_size {
            self.encoder.set_chunk_size(size);
        }
        Ok(encoded)
    }
}

#[derive(Debug, Error)]
pub enum SessionError {
    #[error(transparent)]
    Handshake(#[from] HandshakeError),
    #[error(transparent)]
    Chunk(#[from] ChunkError),
    #[error(transparent)]
    Message(#[from] MessageError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtmp::{ChunkSize, RtmpMessage};

    fn active_session() -> RtmpSession {
        let (mut session, c0c1) = RtmpSession::new(1, [7; 1_528], [8; 1_504]);
        let c1 = &c0c1[1..];
        let mut server = Vec::new();
        server.push(3);
        server.extend_from_slice(&2u32.to_be_bytes());
        server.extend_from_slice(&0u32.to_be_bytes());
        server.extend_from_slice(&[8; 1_528]);
        server.extend_from_slice(&c1[..4]);
        server.extend_from_slice(&3u32.to_be_bytes());
        server.extend_from_slice(&c1[8..]);
        let actions = session
            .receive(&server, 4)
            .expect("test handshake completes");
        assert!(actions.contains(&SessionAction::HandshakeComplete));
        session
    }

    fn inbound(message: ProtocolMessage) -> Vec<u8> {
        ChunkEncoder::default()
            .encode(
                ChunkStreamId::CONTROL,
                &message.encode().expect("test message encodes"),
            )
            .expect("test chunks encode")
    }

    #[test]
    fn applies_new_chunk_size_before_the_next_message_in_the_same_read() {
        let mut session = active_session();
        let size = ChunkSize::try_from(4_096).expect("4096 is valid");
        let mut bytes = inbound(ProtocolMessage::SetChunkSize(size));
        let command = Command::new(
            "large".into(),
            super::super::TransactionId::NOTIFICATION,
            std::rc::Rc::new(flash_lso::types::Value::Null),
            vec![std::rc::Rc::new(flash_lso::types::Value::String(
                "x".repeat(1_000),
            ))],
        );
        let mut encoder = ChunkEncoder::default();
        encoder.set_chunk_size(size);
        bytes.extend(
            encoder
                .encode(
                    ChunkStreamId::COMMAND,
                    &ProtocolMessage::Command(command.clone())
                        .encode()
                        .expect("command message encodes"),
                )
                .expect("large chunks encode"),
        );

        let actions = session
            .receive(&bytes, 5)
            .expect("same-read messages decode");
        assert!(actions.contains(&SessionAction::Command(command)));
    }

    #[test]
    fn ping_during_a_fragmented_message_produces_an_exact_response() {
        let mut session = active_session();
        let ping = inbound(ProtocolMessage::UserControl(UserControlEvent::PingRequest(
            0x0102_0304,
        )));
        let actions = session.receive(&ping, 5).expect("ping decodes");
        let response = actions
            .into_iter()
            .find_map(|action| match action {
                SessionAction::Outbound(bytes) => Some(bytes),
                _ => None,
            })
            .expect("ping response is emitted");
        let mut decoder = ChunkDecoder::default();
        decoder.push(&response).expect("response fits");
        let message = decoder
            .next_message()
            .expect("response chunks decode")
            .expect("response message exists");
        assert_eq!(message.payload, [0, 7, 1, 2, 3, 4]);
    }

    #[test]
    fn emits_one_acknowledgement_per_completed_window() {
        let mut session = active_session();
        let window = inbound(ProtocolMessage::WindowAcknowledgementSize(8));
        let _ = session.receive(&window, 5).expect("window message decodes");
        // Include enough unknown RTMP bytes to cross several windows. A complete
        // message is used so the decoder remains synchronized.
        let bytes = ChunkEncoder::default()
            .encode(
                ChunkStreamId::COMMAND,
                &RtmpMessage {
                    timestamp: 0,
                    message_type: 18,
                    message_stream_id: 0,
                    payload: vec![0; 40],
                },
            )
            .expect("unknown message encodes");
        let actions = session.receive(&bytes, 6).expect("unknown message decodes");
        assert!(
            actions
                .iter()
                .filter(|action| matches!(action, SessionAction::Outbound(_)))
                .count()
                >= 3
        );
    }
}
