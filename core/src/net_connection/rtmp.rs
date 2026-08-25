use super::{ResponderCallback, ResponderHandle};
use crate::rtmp::{Command, RtmpSession, SessionAction, TransactionId};
use crate::socket::SocketHandle;
use flash_lso::types::{Element, ObjectId, Value as AmfValue};
use std::collections::HashMap;
use std::rc::Rc;
use url::Url;

#[derive(Debug)]
pub(super) struct RtmpConnection {
    uri: String,
    host: String,
    port: u16,
    session: RtmpSession,
    initial_handshake: Option<Vec<u8>>,
    connect_command: Option<Command>,
    socket: Option<SocketHandle>,
    pending_responders: HashMap<TransactionId, ResponderHandle>,
    next_transaction_id: u32,
    epoch_time: u32,
    connected: bool,
}

#[derive(Debug)]
pub(crate) enum RtmpTransportEvent {
    Connected(SocketHandle),
    Failed(SocketHandle),
    Data(Vec<u8>),
    Closed,
}

#[derive(Debug)]
pub(super) enum RtmpConnectionAction {
    Send(SocketHandle, Vec<u8>),
    CloseSocket(SocketHandle),
    Connected(Command),
    ConnectFailed(Option<Command>),
    Closed,
    Responder {
        responder: ResponderHandle,
        callback: ResponderCallback,
        command: Command,
    },
    Invoke(Command),
}

pub(super) struct ConnectProperties {
    pub flash_version: String,
    pub swf_url: String,
    pub page_url: String,
}

impl RtmpConnection {
    pub fn new(
        uri: String,
        extra_arguments: Vec<Rc<AmfValue>>,
        properties: ConnectProperties,
        time: u32,
        c1_random: [u8; 1_528],
        c2_random: [u8; 1_504],
    ) -> Result<Self, url::ParseError> {
        let parsed = Url::parse(&uri)?;
        let host = parsed
            .host_str()
            .ok_or(url::ParseError::EmptyHost)?
            .to_string();
        let port = parsed.port().unwrap_or(1_935);
        let app = parsed.path().trim_start_matches('/').to_string();
        let command_object = Rc::new(AmfValue::Object(
            ObjectId::INVALID,
            vec![
                element("app", AmfValue::String(app)),
                element("flashVer", AmfValue::String(properties.flash_version)),
                element("swfUrl", AmfValue::String(properties.swf_url)),
                element("tcUrl", AmfValue::String(uri.clone())),
                element("fpad", AmfValue::Bool(false)),
                element("capabilities", AmfValue::Number(239.0)),
                element("audioCodecs", AmfValue::Number(3_575.0)),
                element("videoCodecs", AmfValue::Number(252.0)),
                element("videoFunction", AmfValue::Number(1.0)),
                element("pageUrl", AmfValue::String(properties.page_url)),
            ],
            None,
        ));
        let connect_command = Command::new(
            "connect".to_string(),
            TransactionId::CONNECT,
            command_object,
            extra_arguments,
        );
        let (session, initial_handshake) = RtmpSession::new(time, c1_random, c2_random);

        Ok(Self {
            uri,
            host,
            port,
            session,
            initial_handshake: Some(initial_handshake),
            connect_command: Some(connect_command),
            socket: None,
            pending_responders: HashMap::new(),
            next_transaction_id: 2,
            epoch_time: time,
            connected: false,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }

    pub fn socket(&self) -> Option<SocketHandle> {
        self.socket
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn send(
        &mut self,
        name: String,
        responder: Option<ResponderHandle>,
        arguments: Vec<Rc<AmfValue>>,
        current_time: u32,
    ) -> Vec<RtmpConnectionAction> {
        let transaction_id = if responder.is_some() {
            let id = TransactionId::from(self.next_transaction_id);
            self.next_transaction_id = self.next_transaction_id.checked_add(1).unwrap_or(2);
            id
        } else {
            TransactionId::NOTIFICATION
        };
        if let Some(responder) = responder {
            self.pending_responders.insert(transaction_id, responder);
        }
        let command = Command::new(name, transaction_id, Rc::new(AmfValue::Null), arguments);
        self.encode_command(&command, current_time.wrapping_sub(self.epoch_time))
            .into_iter()
            .collect()
    }

    pub fn send_result(
        &mut self,
        transaction_id: TransactionId,
        value: AmfValue,
        current_time: u32,
    ) -> Vec<RtmpConnectionAction> {
        if transaction_id == TransactionId::NOTIFICATION {
            return Vec::new();
        }

        let command = Command::new(
            "_result".to_string(),
            transaction_id,
            Rc::new(AmfValue::Null),
            vec![Rc::new(value)],
        );
        self.encode_command(&command, current_time.wrapping_sub(self.epoch_time))
            .into_iter()
            .collect()
    }

    pub fn handle_transport_event(
        &mut self,
        event: RtmpTransportEvent,
        receive_time: u32,
    ) -> Vec<RtmpConnectionAction> {
        match event {
            RtmpTransportEvent::Connected(socket) => {
                self.socket = Some(socket);
                self.initial_handshake
                    .take()
                    .map(|bytes| RtmpConnectionAction::Send(socket, bytes))
                    .into_iter()
                    .collect()
            }
            RtmpTransportEvent::Failed(socket) => {
                self.connected = false;
                self.socket = None;
                vec![
                    RtmpConnectionAction::ConnectFailed(None),
                    RtmpConnectionAction::CloseSocket(socket),
                ]
            }
            RtmpTransportEvent::Closed => {
                self.socket = None;
                self.connected = false;
                vec![RtmpConnectionAction::Closed]
            }
            RtmpTransportEvent::Data(bytes) => match self.session.receive(&bytes, receive_time) {
                Ok(actions) => self.handle_session_actions(actions),
                Err(error) => {
                    tracing::warn!(error = %error, "RTMP session rejected peer data");
                    self.connected = false;
                    let mut actions = vec![RtmpConnectionAction::ConnectFailed(None)];
                    if let Some(socket) = self.socket.take() {
                        actions.push(RtmpConnectionAction::CloseSocket(socket));
                    }
                    actions
                }
            },
        }
    }

    fn handle_session_actions(&mut self, actions: Vec<SessionAction>) -> Vec<RtmpConnectionAction> {
        let mut output = Vec::new();
        for action in actions {
            match action {
                SessionAction::Outbound(bytes) => {
                    if let Some(socket) = self.socket {
                        output.push(RtmpConnectionAction::Send(socket, bytes));
                    }
                }
                SessionAction::HandshakeComplete => {
                    tracing::debug!("RTMP handshake completed");
                    if let Some(command) = self.connect_command.take() {
                        let action = self.encode_command(&command, 0);
                        // Flash starts the first post-connect command with a
                        // fresh format-0 header on the command chunk stream.
                        self.session.clear_outbound_chunk_history();
                        if let Some(action) = action {
                            output.push(action);
                        }
                    }
                }
                SessionAction::Command(command) => {
                    tracing::debug!(
                        method = %command.name,
                        transaction_id = command.transaction_id.get(),
                        "Received RTMP command"
                    );
                    self.handle_command(command, &mut output);
                }
                SessionAction::Message(_) => {}
            }
        }
        output
    }

    fn handle_command(&mut self, command: Command, output: &mut Vec<RtmpConnectionAction>) {
        let transaction_id = command.transaction_id;
        if transaction_id == TransactionId::CONNECT {
            if command.name == "_result" {
                self.connected = true;
                output.push(RtmpConnectionAction::Connected(command));
            } else if command.name == "_error" {
                self.connected = false;
                output.push(RtmpConnectionAction::ConnectFailed(Some(command)));
            }
            return;
        }

        if matches!(command.name.as_str(), "_result" | "_error") {
            if let Some(responder) = self.pending_responders.remove(&transaction_id) {
                let callback = if command.name == "_error" {
                    ResponderCallback::Status
                } else {
                    ResponderCallback::Result
                };
                output.push(RtmpConnectionAction::Responder {
                    responder,
                    callback,
                    command,
                });
            }
            return;
        }

        output.push(RtmpConnectionAction::Invoke(command));
    }

    fn encode_command(
        &mut self,
        command: &Command,
        timestamp: u32,
    ) -> Option<RtmpConnectionAction> {
        let socket = self.socket?;
        tracing::debug!(
            method = %command.name,
            transaction_id = command.transaction_id.get(),
            argument_count = command.arguments.len(),
            "Sending RTMP command"
        );
        match self.session.send_command(command, timestamp) {
            Ok(bytes) => Some(RtmpConnectionAction::Send(socket, bytes)),
            Err(error) => {
                tracing::warn!(error = %error, "RTMP command could not be encoded");
                None
            }
        }
    }
}

fn element(name: &str, value: AmfValue) -> Element {
    Element::new(name.to_string(), Rc::new(value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rtmp::{ChunkDecoder, ChunkEncoder, ChunkStreamId, ProtocolMessage};
    use slotmap::SlotMap;

    fn socket_handle() -> SocketHandle {
        let mut sockets = SlotMap::with_key();
        sockets.insert(())
    }

    fn connection() -> RtmpConnection {
        RtmpConnection::new(
            "rtmp://example.test/application".into(),
            vec![Rc::new(AmfValue::String("credential".into()))],
            ConnectProperties {
                flash_version: "WIN 23,0,0,0".into(),
                swf_url: "https://example.test/game.swf".into(),
                page_url: "https://example.test/game".into(),
            },
            7,
            [0x5a; 1_528],
            [0xa5; 1_504],
        )
        .expect("test RTMP URL parses")
    }

    fn complete_handshake(connection: &mut RtmpConnection, socket: SocketHandle) -> Command {
        let actions = connection.handle_transport_event(RtmpTransportEvent::Connected(socket), 7);
        let c0c1 = match actions.as_slice() {
            [RtmpConnectionAction::Send(sent_socket, bytes)] if *sent_socket == socket => bytes,
            _ => panic!("transport connect must emit C0+C1"),
        };
        let c1 = &c0c1[1..];
        let mut server = Vec::with_capacity(3_073);
        server.push(3);
        server.extend_from_slice(&8_u32.to_be_bytes());
        server.extend_from_slice(&0_u32.to_be_bytes());
        server.extend_from_slice(&[0x33; 1_528]);
        server.extend_from_slice(&c1[..4]);
        server.extend_from_slice(&9_u32.to_be_bytes());
        server.extend_from_slice(&c1[8..]);

        let actions = connection.handle_transport_event(RtmpTransportEvent::Data(server), 9);
        let connect_bytes = actions
            .iter()
            .find_map(|action| match action {
                RtmpConnectionAction::Send(_, bytes) if bytes.len() != 1_536 => Some(bytes),
                _ => None,
            })
            .expect("handshake completion emits connect command");
        let mut decoder = ChunkDecoder::default();
        decoder.push(connect_bytes).expect("connect chunks decode");
        let message = decoder
            .next_message()
            .expect("connect chunk is valid")
            .expect("connect message exists");
        match crate::rtmp::decode_message(message).expect("connect message decodes") {
            ProtocolMessage::Command(command) => command,
            _ => panic!("handshake must be followed by an RTMP command"),
        }
    }

    fn server_command(command: Command) -> Vec<u8> {
        ChunkEncoder::default()
            .encode(
                ChunkStreamId::COMMAND,
                &ProtocolMessage::Command(command)
                    .encode()
                    .expect("server command encodes"),
            )
            .expect("server command chunks encode")
    }

    #[test]
    fn handshake_sends_generic_connect_profile_and_preserves_extra_arguments() {
        let mut connection = connection();
        let command = complete_handshake(&mut connection, socket_handle());

        assert_eq!(command.name, "connect");
        assert_eq!(command.transaction_id, TransactionId::CONNECT);
        assert_eq!(
            command.arguments,
            [Rc::new(AmfValue::String("credential".into()))]
        );
        let AmfValue::Object(_, properties, _) = command.command_object.as_ref() else {
            panic!("connect command object must be an AMF object");
        };
        assert!(properties.iter().any(|property| {
            property.name() == "tcUrl"
                && property.value() == &AmfValue::String("rtmp://example.test/application".into())
        }));
        assert!(
            !properties
                .iter()
                .any(|property| property.name() == "objectEncoding")
        );
    }

    #[test]
    fn result_changes_connection_state_and_remote_calls_remain_generic() {
        let mut connection = connection();
        let socket = socket_handle();
        let _ = complete_handshake(&mut connection, socket);
        let result = Command::new(
            "_result".into(),
            TransactionId::CONNECT,
            Rc::new(AmfValue::Null),
            vec![Rc::new(AmfValue::Object(
                ObjectId::INVALID,
                vec![element(
                    "code",
                    AmfValue::String("NetConnection.Connect.Success".into()),
                )],
                None,
            ))],
        );
        let actions =
            connection.handle_transport_event(RtmpTransportEvent::Data(server_command(result)), 10);
        assert!(connection.is_connected());
        assert!(matches!(
            actions.as_slice(),
            [RtmpConnectionAction::Connected(_)]
        ));

        let notification = Command::new(
            "applicationMethod".into(),
            TransactionId::NOTIFICATION,
            Rc::new(AmfValue::Null),
            vec![Rc::new(AmfValue::Number(42.0))],
        );
        let actions = connection
            .handle_transport_event(RtmpTransportEvent::Data(server_command(notification)), 11);
        assert!(matches!(
            actions.as_slice(),
            [RtmpConnectionAction::Invoke(command)]
                if command.name == "applicationMethod"
        ));

        let remote_call = Command::new(
            "applicationMethodWithResult".into(),
            TransactionId::from(19),
            Rc::new(AmfValue::Null),
            vec![Rc::new(AmfValue::Number(43.0))],
        );
        let actions = connection
            .handle_transport_event(RtmpTransportEvent::Data(server_command(remote_call)), 12);
        assert!(matches!(
            actions.as_slice(),
            [RtmpConnectionAction::Invoke(command)]
                if command.name == "applicationMethodWithResult"
                    && command.transaction_id == TransactionId::from(19)
        ));
    }

    #[test]
    fn remote_call_result_uses_the_server_transaction_id() {
        let mut connection = connection();
        let socket = socket_handle();
        let _ = complete_handshake(&mut connection, socket);
        let actions = connection.send_result(TransactionId::from(27), AmfValue::Number(7.0), 47);
        let bytes = match actions.as_slice() {
            [RtmpConnectionAction::Send(sent_socket, bytes)] if *sent_socket == socket => bytes,
            _ => panic!("remote call result must be sent on the active socket"),
        };
        let mut decoder = ChunkDecoder::default();
        decoder.push(bytes).expect("result chunks decode");
        let message = decoder
            .next_message()
            .expect("result chunk is valid")
            .expect("result message exists");
        assert_eq!(message.timestamp, 40);
        let ProtocolMessage::Command(command) =
            crate::rtmp::decode_message(message).expect("result message decodes")
        else {
            panic!("remote call result must be a command");
        };
        assert_eq!(command.name, "_result");
        assert_eq!(command.transaction_id, TransactionId::from(27));
        assert_eq!(command.arguments, [Rc::new(AmfValue::Number(7.0))]);
    }
}
