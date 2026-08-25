use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

const VERSION: u8 = 3;
const HANDSHAKE_BLOCK_LEN: usize = 1_536;
const RANDOM_LEN: usize = HANDSHAKE_BLOCK_LEN - 8;
const DIGEST_LEN: usize = 32;
const SIGNATURE_INPUT_LEN: usize = HANDSHAKE_BLOCK_LEN - DIGEST_LEN;
const COMPLEX_VERSION: [u8; 4] = [0x80, 0x00, 0x07, 0x02];
const GENUINE_FLASH_PLAYER: &[u8] = b"Genuine Adobe Flash Player 001";
const GENUINE_FLASH_MEDIA_SERVER: &[u8] = b"Genuine Adobe Flash Media Server 001";
const GENUINE_KEY_SUFFIX: [u8; 32] = [
    0xf0, 0xee, 0xc2, 0x4a, 0x80, 0x68, 0xbe, 0xe8, 0x2e, 0x00, 0xd0, 0xd1, 0x02, 0x9e, 0x7e, 0x57,
    0x6e, 0xec, 0x5d, 0x2d, 0x29, 0x80, 0x6f, 0xab, 0x93, 0xb8, 0xe6, 0x36, 0xcf, 0xeb, 0x31, 0xae,
];

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum State {
    ReadS0S1,
    ReadS2,
    Complete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HandshakeKind {
    Simple,
    Complex { client_digest_offset: usize },
}

/// Incremental RTMP client handshake with complex-handshake validation and a
/// simple-handshake fallback for older servers.
#[derive(Debug)]
pub struct ClientHandshake {
    state: State,
    c1: [u8; HANDSHAKE_BLOCK_LEN],
    kind: HandshakeKind,
    c2_random: [u8; SIGNATURE_INPUT_LEN],
    received: Vec<u8>,
}

/// Output produced by one incremental handshake step.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct HandshakeOutput {
    /// Bytes that must be written to the transport before processing continues.
    pub outbound: Vec<u8>,
    /// Bytes following S2, which belong to the RTMP chunk stream.
    pub remainder: Vec<u8>,
    pub complete: bool,
}

impl ClientHandshake {
    /// Creates a handshake and the C0+C1 bytes that begin it.
    pub fn new(
        time: u32,
        c1_random: [u8; RANDOM_LEN],
        c2_random: [u8; SIGNATURE_INPUT_LEN],
    ) -> (Self, Vec<u8>) {
        let mut c1 = [0; HANDSHAKE_BLOCK_LEN];
        c1[..4].copy_from_slice(&time.to_be_bytes());
        c1[4..8].copy_from_slice(&COMPLEX_VERSION);
        c1[8..].copy_from_slice(&c1_random);
        let client_digest_offset = scheme_two_digest_offset(&c1);
        let digest = digest_without_slot(&c1, client_digest_offset, GENUINE_FLASH_PLAYER);
        c1[client_digest_offset..client_digest_offset + DIGEST_LEN].copy_from_slice(&digest);

        let mut initial = Vec::with_capacity(1 + HANDSHAKE_BLOCK_LEN);
        initial.push(VERSION);
        initial.extend_from_slice(&c1);
        (
            Self {
                state: State::ReadS0S1,
                c1,
                kind: HandshakeKind::Complex {
                    client_digest_offset,
                },
                c2_random,
                received: Vec::with_capacity(1 + 2 * HANDSHAKE_BLOCK_LEN),
            },
            initial,
        )
    }

    /// Feeds an arbitrary transport fragment into the handshake state machine.
    pub fn feed(
        &mut self,
        bytes: &[u8],
        _receive_time: u32,
    ) -> Result<HandshakeOutput, HandshakeError> {
        if self.state == State::Complete {
            return Err(HandshakeError::AlreadyComplete);
        }
        self.received.extend_from_slice(bytes);
        let mut output = HandshakeOutput::default();

        if self.state == State::ReadS0S1 && self.received.len() > HANDSHAKE_BLOCK_LEN {
            if self.received[0] != VERSION {
                return Err(HandshakeError::UnsupportedVersion(self.received[0]));
            }
            let s1 = &self.received[1..=HANDSHAKE_BLOCK_LEN];
            if let Some(server_digest_offset) = server_digest_offset(s1) {
                output.outbound = self.complex_c2(s1, server_digest_offset);
            } else {
                self.kind = HandshakeKind::Simple;
                output.outbound.reserve(HANDSHAKE_BLOCK_LEN);
                output.outbound.extend_from_slice(&s1[..4]);
                // Flash Player leaves the simple C2 time2 field at zero.
                // Some servers accept another value initially but later reject
                // the connection when this fingerprint does not match Flash.
                output.outbound.extend_from_slice(&0_u32.to_be_bytes());
                output.outbound.extend_from_slice(&s1[8..]);
            }
            self.received.drain(..=HANDSHAKE_BLOCK_LEN);
            self.state = State::ReadS2;
        }

        if self.state == State::ReadS2 && self.received.len() >= HANDSHAKE_BLOCK_LEN {
            let s2 = &self.received[..HANDSHAKE_BLOCK_LEN];
            match self.kind {
                HandshakeKind::Simple => {
                    if s2[..4] != self.c1[..4] || s2[8..] != self.c1[8..] {
                        return Err(HandshakeError::InvalidS2Echo);
                    }
                }
                HandshakeKind::Complex {
                    client_digest_offset,
                } => {
                    let client_digest =
                        &self.c1[client_digest_offset..client_digest_offset + DIGEST_LEN];
                    let key = hmac_sha256(&full_key(GENUINE_FLASH_MEDIA_SERVER), client_digest);
                    let expected = hmac_sha256(&key, &s2[..SIGNATURE_INPUT_LEN]);
                    if s2[SIGNATURE_INPUT_LEN..] != expected {
                        return Err(HandshakeError::InvalidS2Signature);
                    }
                }
            }
            output
                .remainder
                .extend_from_slice(&self.received[HANDSHAKE_BLOCK_LEN..]);
            self.received.clear();
            self.state = State::Complete;
            output.complete = true;
        }

        Ok(output)
    }

    fn complex_c2(&self, s1: &[u8], server_digest_offset: usize) -> Vec<u8> {
        let server_digest = &s1[server_digest_offset..server_digest_offset + DIGEST_LEN];
        let key = hmac_sha256(&full_key(GENUINE_FLASH_PLAYER), server_digest);
        let mut c2 = vec![0; HANDSHAKE_BLOCK_LEN];
        c2[..SIGNATURE_INPUT_LEN].copy_from_slice(&self.c2_random);
        let signature = hmac_sha256(&key, &c2[..SIGNATURE_INPUT_LEN]);
        c2[SIGNATURE_INPUT_LEN..].copy_from_slice(&signature);
        c2
    }
}

fn hmac_sha256(key: &[u8], input: &[u8]) -> [u8; DIGEST_LEN] {
    let mut hmac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of any length");
    hmac.update(input);
    hmac.finalize().into_bytes().into()
}

fn full_key(prefix: &[u8]) -> Vec<u8> {
    let mut key = Vec::with_capacity(prefix.len() + GENUINE_KEY_SUFFIX.len());
    key.extend_from_slice(prefix);
    key.extend_from_slice(&GENUINE_KEY_SUFFIX);
    key
}

fn digest_without_slot(block: &[u8], offset: usize, key: &[u8]) -> [u8; DIGEST_LEN] {
    let mut input = Vec::with_capacity(SIGNATURE_INPUT_LEN);
    input.extend_from_slice(&block[..offset]);
    input.extend_from_slice(&block[offset + DIGEST_LEN..]);
    hmac_sha256(key, &input)
}

fn scheme_one_digest_offset(block: &[u8]) -> usize {
    (usize::from(block[8])
        + usize::from(block[9])
        + usize::from(block[10])
        + usize::from(block[11]))
        % 728
        + 12
}

fn scheme_two_digest_offset(block: &[u8]) -> usize {
    (usize::from(block[772])
        + usize::from(block[773])
        + usize::from(block[774])
        + usize::from(block[775]))
        % 728
        + 776
}

fn server_digest_offset(s1: &[u8]) -> Option<usize> {
    [scheme_two_digest_offset(s1), scheme_one_digest_offset(s1)]
        .into_iter()
        .find(|&offset| {
            let expected = digest_without_slot(s1, offset, GENUINE_FLASH_MEDIA_SERVER);
            s1[offset..offset + DIGEST_LEN] == expected
        })
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum HandshakeError {
    #[error("unsupported RTMP handshake version {0}")]
    UnsupportedVersion(u8),
    #[error("RTMP S2 did not echo C1")]
    InvalidS2Echo,
    #[error("RTMP S2 has an invalid complex-handshake signature")]
    InvalidS2Signature,
    #[error("RTMP handshake is already complete")]
    AlreadyComplete,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server_response(c1: &[u8], tail: &[u8]) -> Vec<u8> {
        let mut response = Vec::new();
        response.push(VERSION);
        response.extend_from_slice(&0x0102_0304u32.to_be_bytes());
        response.extend_from_slice(&0u32.to_be_bytes());
        response.extend((0..RANDOM_LEN).map(|value| value as u8));
        response.extend_from_slice(&c1[..4]);
        response.extend_from_slice(&0x0506_0708u32.to_be_bytes());
        response.extend_from_slice(&c1[8..]);
        response.extend_from_slice(tail);
        response
    }

    fn complex_server_response(c1: &[u8], tail: &[u8]) -> Vec<u8> {
        let mut s1 = [0x33; HANDSHAKE_BLOCK_LEN];
        s1[..4].copy_from_slice(&0x0102_0304u32.to_be_bytes());
        s1[4..8].copy_from_slice(&COMPLEX_VERSION);
        let server_digest_offset = scheme_two_digest_offset(&s1);
        let server_digest =
            digest_without_slot(&s1, server_digest_offset, GENUINE_FLASH_MEDIA_SERVER);
        s1[server_digest_offset..server_digest_offset + DIGEST_LEN].copy_from_slice(&server_digest);

        let client_digest_offset = scheme_two_digest_offset(c1);
        let client_digest = &c1[client_digest_offset..client_digest_offset + DIGEST_LEN];
        let s2_key = hmac_sha256(&full_key(GENUINE_FLASH_MEDIA_SERVER), client_digest);
        let mut s2 = [0x44; HANDSHAKE_BLOCK_LEN];
        let s2_signature = hmac_sha256(&s2_key, &s2[..SIGNATURE_INPUT_LEN]);
        s2[SIGNATURE_INPUT_LEN..].copy_from_slice(&s2_signature);

        let mut response = Vec::with_capacity(1 + 2 * HANDSHAKE_BLOCK_LEN + tail.len());
        response.push(VERSION);
        response.extend_from_slice(&s1);
        response.extend_from_slice(&s2);
        response.extend_from_slice(tail);
        response
    }

    #[test]
    fn complex_handshake_uses_flash_scheme_two_and_validates_both_signatures() {
        let c1_random = [0x5a; RANDOM_LEN];
        let c2_random = [0xa5; SIGNATURE_INPUT_LEN];
        let (mut handshake, initial) = ClientHandshake::new(42, c1_random, c2_random);
        let c1 = &initial[1..];
        assert_eq!(c1[4..8], COMPLEX_VERSION);
        let client_digest_offset = scheme_two_digest_offset(c1);
        assert_eq!(
            c1[client_digest_offset..client_digest_offset + DIGEST_LEN],
            digest_without_slot(c1, client_digest_offset, GENUINE_FLASH_PLAYER)
        );

        let response = complex_server_response(c1, &[1, 2, 3]);
        let output = handshake
            .feed(&response, 100)
            .expect("complex server response is valid");
        assert!(output.complete);
        assert_eq!(output.remainder, [1, 2, 3]);
        assert_eq!(output.outbound.len(), HANDSHAKE_BLOCK_LEN);
        assert_eq!(output.outbound[..SIGNATURE_INPUT_LEN], c2_random);

        let s1 = &response[1..=HANDSHAKE_BLOCK_LEN];
        let server_digest_offset = scheme_two_digest_offset(s1);
        let server_digest = &s1[server_digest_offset..server_digest_offset + DIGEST_LEN];
        let c2_key = hmac_sha256(&full_key(GENUINE_FLASH_PLAYER), server_digest);
        assert_eq!(
            output.outbound[SIGNATURE_INPUT_LEN..],
            hmac_sha256(&c2_key, &output.outbound[..SIGNATURE_INPUT_LEN])
        );
    }

    #[test]
    fn simple_handshake_survives_every_fragment_boundary() {
        let random = [0x5a; RANDOM_LEN];
        let c2_random = [0xa5; SIGNATURE_INPUT_LEN];
        let (_, initial) = ClientHandshake::new(42, random, c2_random);
        let response = server_response(&initial[1..], &[1, 2, 3]);

        for split in 0..=response.len() {
            let (mut handshake, _) = ClientHandshake::new(42, random, c2_random);
            let first = handshake
                .feed(&response[..split], 100)
                .expect("the first fragment is valid");
            let second = if first.complete {
                HandshakeOutput::default()
            } else {
                handshake
                    .feed(&response[split..], 100)
                    .expect("the second fragment is valid")
            };
            assert_eq!(
                first.outbound.len() + second.outbound.len(),
                HANDSHAKE_BLOCK_LEN
            );
            let outbound = if first.outbound.is_empty() {
                &second.outbound
            } else {
                &first.outbound
            };
            assert_eq!(outbound[4..8], [0; 4]);
            assert!(first.complete || second.complete);
            let remainder = if first.complete {
                let mut remainder = first.remainder;
                remainder.extend_from_slice(&response[split..]);
                remainder
            } else {
                second.remainder
            };
            assert_eq!(remainder, [1, 2, 3]);
        }
    }

    #[test]
    fn rejects_invalid_version_and_echo() {
        let random = [0; RANDOM_LEN];
        let c2_random = [1; SIGNATURE_INPUT_LEN];
        let (mut handshake, initial) = ClientHandshake::new(0, random, c2_random);
        let mut response = server_response(&initial[1..], &[]);
        response[0] = 2;
        assert_eq!(
            handshake.feed(&response, 0),
            Err(HandshakeError::UnsupportedVersion(2))
        );

        let (mut handshake, initial) = ClientHandshake::new(0, random, c2_random);
        let mut response = server_response(&initial[1..], &[]);
        response[1 + HANDSHAKE_BLOCK_LEN + 8] ^= 1;
        assert_eq!(
            handshake.feed(&response, 0),
            Err(HandshakeError::InvalidS2Echo)
        );
    }
}
