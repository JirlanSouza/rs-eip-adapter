use bytes::Bytes;
use error::EncapsulationError;
use header::EncapsulationHeader;

pub mod command;
mod cpf;
pub mod error;
pub mod handler;
pub mod header;
mod list_identity;

pub const ENCAPSULATION_PROTOCOL_VERSION: u16 = 1;

struct Encapsulation {
    header: EncapsulationHeader,
    payload: Bytes,
}

impl Encapsulation {
    fn decode(mut in_buff: Bytes) -> Option<Self> {
        let header = EncapsulationHeader::decode(&mut in_buff)?;
        let payload = in_buff;
        Some(Self { header, payload })
    }

    fn validate_length(&self) -> Option<EncapsulationError> {
        if self.payload.len() != self.header.length as usize {
            log::warn!(
                "Invalid payload length: expected {}, got {}",
                self.header.length,
                self.payload.len()
            );
            Some(EncapsulationError::InvalidLength)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};
    use command::EncapsulationCommand;

    #[test]
    fn decode_returns_encapsulation_for_valid_header_and_payload() {
        let test_header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 3,
            session_handle: 0x1122_3344,
            status: 0,
            context: [0u8; 8],
            options: 0,
        };

        let mut buffer = BytesMut::with_capacity(header::ENCAPSULATION_HEADER_SIZE + 3);
        test_header
            .encode(&mut buffer)
            .expect("header encode should succeed");
        buffer.put_slice(&[0x01u8, 0x02, 0x03]);
        let frozen = buffer.freeze();

        let parsed =
            Encapsulation::decode(frozen).expect("Encapsulation::decode should return Some");
        assert_eq!(parsed.header, test_header);
        assert_eq!(parsed.payload, Bytes::from(&[0x01u8, 0x02, 0x03][..]));
    }

    #[test]
    fn validate_length_returns_none_when_lengths_match() {
        let test_header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 2,
            session_handle: 0,
            status: 0,
            context: [0u8; 8],
            options: 0,
        };

        let mut buffer = BytesMut::with_capacity(header::ENCAPSULATION_HEADER_SIZE + 2);
        test_header
            .encode(&mut buffer)
            .expect("header encode should succeed");
        buffer.put_slice(&[0xAAu8, 0xBB]);
        let frozen = buffer.freeze();

        let parsed =
            Encapsulation::decode(frozen).expect("Encapsulation::decode should return Some");
        let validation = parsed.validate_length();
        assert!(validation.is_none());
    }

    #[test]
    fn validate_length_detects_mismatched_length() {
        let test_header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 5,
            session_handle: 0,
            status: 0,
            context: [0u8; 8],
            options: 0,
        };

        let mut buffer = BytesMut::with_capacity(header::ENCAPSULATION_HEADER_SIZE + 3);
        test_header
            .encode(&mut buffer)
            .expect("header encode should succeed");
        buffer.put_slice(&[0x10u8, 0x11, 0x12]);
        let frozen = buffer.freeze();

        let parsed =
            Encapsulation::decode(frozen).expect("Encapsulation::decode should return Some");
        let validation = parsed.validate_length();
        assert_eq!(validation, Some(EncapsulationError::InvalidLength));
    }
}
