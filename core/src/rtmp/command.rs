use super::{TransactionId, WireTypeError};
use flash_lso::amf0::{read::AMF0Decoder, write::write_values_to_bytes};
use flash_lso::types::Value as AmfValue;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use thiserror::Error;

/// An RTMP AMF0 command message, independent of any application method names.
#[derive(Clone)]
pub struct Command {
    pub name: String,
    pub transaction_id: TransactionId,
    pub command_object: Rc<AmfValue>,
    pub arguments: Vec<Rc<AmfValue>>,

    /// Decoder state used to resolve AMF0 references shared between values in
    /// an inbound command. Outbound commands have an empty decoder.
    decoder: Rc<AMF0Decoder>,
}

impl Command {
    pub fn new(
        name: String,
        transaction_id: TransactionId,
        command_object: Rc<AmfValue>,
        arguments: Vec<Rc<AmfValue>>,
    ) -> Self {
        Self {
            name,
            transaction_id,
            command_object,
            arguments,
            decoder: Rc::new(AMF0Decoder::default()),
        }
    }

    pub fn decode(payload: &[u8]) -> Result<Self, CommandError> {
        let mut decoder = AMF0Decoder::default();
        let mut remaining = payload;
        let mut values = Vec::new();
        while !remaining.is_empty() {
            let (next, value) = decoder
                .parse_single_element(remaining)
                .map_err(|_| CommandError::Decode)?;
            if next.len() >= remaining.len() {
                return Err(CommandError::DecoderMadeNoProgress);
            }
            values.push(value);
            remaining = next;
        }
        if values.len() < 3 {
            return Err(CommandError::MissingFields(values.len()));
        }
        let name = match values.remove(0).as_ref() {
            AmfValue::String(name) => name.clone(),
            _ => return Err(CommandError::Name),
        };
        let transaction_id = match values.remove(0).as_ref() {
            AmfValue::Number(value) => TransactionId::from_wire(*value)?,
            _ => return Err(CommandError::Transaction),
        };
        let command_object = values.remove(0);
        Ok(Self {
            name,
            transaction_id,
            command_object,
            arguments: values,
            decoder: Rc::new(decoder),
        })
    }

    pub fn decoder(&self) -> &AMF0Decoder {
        &self.decoder
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommandError> {
        let mut values = Vec::with_capacity(3 + self.arguments.len());
        values.push(Rc::new(AmfValue::String(self.name.clone())));
        values.push(Rc::new(AmfValue::Number(f64::from(
            self.transaction_id.get(),
        ))));
        values.push(self.command_object.clone());
        values.extend(self.arguments.iter().cloned());
        write_values_to_bytes(&values).map_err(CommandError::Encode)
    }
}

impl Debug for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Command")
            .field("name", &self.name)
            .field("transaction_id", &self.transaction_id)
            .field("command_object", &self.command_object)
            .field("arguments", &self.arguments)
            .finish_non_exhaustive()
    }
}

impl PartialEq for Command {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.transaction_id == other.transaction_id
            && self.command_object == other.command_object
            && self.arguments == other.arguments
    }
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("AMF0 command decode failed")]
    Decode,
    #[error("AMF0 decoder made no input progress")]
    DecoderMadeNoProgress,
    #[error("AMF0 command needs at least three values, got {0}")]
    MissingFields(usize),
    #[error("AMF0 command name is not a string")]
    Name,
    #[error("AMF0 command transaction ID is not a number")]
    Transaction,
    #[error(transparent)]
    WireType(#[from] WireTypeError),
    #[error("AMF0 command encode failed: {0}")]
    Encode(#[source] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use flash_lso::types::ObjectId;

    #[test]
    fn command_arguments_are_consecutive_amf_values() {
        let command = Command::new(
            "sum".into(),
            TransactionId::from_wire(2.0).expect("2 is an integer"),
            Rc::new(AmfValue::Null),
            vec![
                Rc::new(AmfValue::Number(20.0)),
                Rc::new(AmfValue::Number(50.0)),
            ],
        );
        let encoded = command.encode().expect("command encodes");
        assert_eq!(encoded.len(), 34);
        assert_eq!(encoded[15], 0x05);
        assert_ne!(encoded[16], 0x0a);
        assert_eq!(Command::decode(&encoded).expect("command decodes"), command);
    }

    #[test]
    fn command_without_arguments_has_only_the_three_required_fields() {
        let command = Command::new(
            "ping".into(),
            TransactionId::from_wire(2.0).expect("2 is an integer"),
            Rc::new(AmfValue::Null),
            Vec::new(),
        );
        let encoded = command.encode().expect("command encodes");
        assert_eq!(Command::decode(&encoded).expect("command decodes"), command);
        assert_eq!(encoded.last(), Some(&0x05));
    }

    #[test]
    fn transaction_id_must_be_an_unsigned_integer() {
        for value in [-1.0, 1.5, f64::INFINITY, f64::NAN] {
            let payload = write_values_to_bytes(&[
                Rc::new(AmfValue::String("call".into())),
                Rc::new(AmfValue::Number(value)),
                Rc::new(AmfValue::Null),
            ])
            .expect("test payload encodes");
            assert!(matches!(
                Command::decode(&payload),
                Err(CommandError::WireType(WireTypeError::TransactionId(_)))
            ));
        }
    }

    #[test]
    fn decoder_state_preserves_references_between_command_arguments() {
        let object = Rc::new(AmfValue::Object(ObjectId::INVALID, Vec::new(), None));
        let mut payload = write_values_to_bytes(&[
            Rc::new(AmfValue::String("references".into())),
            Rc::new(AmfValue::Number(0.0)),
            Rc::new(AmfValue::Null),
            object,
        ])
        .expect("test command encodes");
        payload.extend_from_slice(&[0x07, 0x00, 0x00]);

        let command = Command::decode(&payload).expect("test command decodes");
        assert!(
            command
                .decoder()
                .as_reference(&command.arguments[0])
                .is_some()
        );
        assert!(matches!(
            command.arguments[1].as_ref(),
            AmfValue::Reference(_)
        ));
    }
}
