use crate::encap::error::FrameError;
use bytes::Bytes;
use header::EncapsulationHeader;

pub mod broadcast_handler;
pub mod command;
mod cpf;
pub mod error;
pub mod handler;
pub mod header;
mod list_identity;
pub mod session_manager;

pub const ENCAPSULATION_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug)]
pub struct Encapsulation {
    header: EncapsulationHeader,
    payload: Bytes,
}

impl Encapsulation {
    pub fn new(header: EncapsulationHeader, payload: Bytes) -> Result<Self, FrameError> {
        let encapsulation = Self { header, payload };
        encapsulation.validate()?;
        Ok(encapsulation)
    }

    pub fn decode(mut in_buff: Bytes) -> Result<Self, FrameError> {
        let header = EncapsulationHeader::decode(&mut in_buff)?;
        let payload = in_buff;
        let encapsulation = Self { header, payload };
        encapsulation.validate()?;
        Ok(encapsulation)
    }

    fn validate(&self) -> Result<(), FrameError> {
        if self.payload.len() != self.header.length as usize {
            return Err(FrameError::InvalidLength(
                self.header.clone(),
                self.payload.len(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::encap::header::ENCAPSULATION_HEADER_SIZE;

    use super::*;
    use bytes::{BufMut, Bytes, BytesMut};
    use command::EncapsulationCommand;

    #[test]
    fn new_returns_encapsulation_for_valid_header_and_payload() {
        let test_header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 3,
            session_handle: 0x1122_3344,
            status: 0,
            context: [0u8; 8],
            options: 0,
        };

        let mut buffer = BytesMut::with_capacity(3);
        buffer.put_slice(&[0x01u8, 0x02, 0x03]);
        let frozen = buffer.freeze();

        let parsed = Encapsulation::new(test_header.clone(), frozen)
            .expect("Encapsulation::decode should return Some");
        assert_eq!(parsed.header, test_header);
        assert_eq!(parsed.payload, Bytes::from(&[0x01u8, 0x02, 0x03][..]));
    }

    #[test]
    fn new_returns_err_when_lengths_do_not_match() {
        let test_header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 2,
            session_handle: 0,
            status: 0,
            context: [0u8; 8],
            options: 0,
        };

        let mut buffer = BytesMut::with_capacity(6);
        buffer.put_slice(&[0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06]);
        let frozen = buffer.freeze();

        let parsed = Encapsulation::new(test_header.clone(), frozen);
        assert!(parsed.is_err());
        assert_eq!(
            parsed.unwrap_err(),
            FrameError::InvalidLength(test_header.clone(), 6)
        );
    }

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
    fn decode_returns_err_when_lengths_do_not_match() {
        let test_header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 2,
            session_handle: 0,
            status: 0,
            context: [0u8; 8],
            options: 0,
        };

        let mut buffer = BytesMut::with_capacity(header::ENCAPSULATION_HEADER_SIZE + 6);
        test_header
            .encode(&mut buffer)
            .expect("header encode should succeed");
        buffer.put_slice(&[0x01u8, 0x02, 0x03, 0x04, 0x05, 0x06]);
        let frozen = buffer.freeze();

        let parsed = Encapsulation::decode(frozen);
        assert!(parsed.is_err());
        assert_eq!(
            parsed.unwrap_err(),
            FrameError::InvalidLength(test_header.clone(), 6)
        );
    }

    #[test]
    fn decode_returns_err_when_buffer_is_too_small() {
        let test_header = EncapsulationHeader {
            command: EncapsulationCommand::ListIdentity,
            length: 2,
            session_handle: 0,
            status: 0,
            context: [0u8; 8],
            options: 0,
        };

        let mut buffer = BytesMut::with_capacity(header::ENCAPSULATION_HEADER_SIZE);
        test_header
            .encode(&mut buffer)
            .expect("header encode should succeed");
        let frozen = buffer.split_to(ENCAPSULATION_HEADER_SIZE - 3).freeze();

        let parsed = Encapsulation::decode(frozen);
        assert!(parsed.is_err());
        assert_eq!(
            parsed.unwrap_err(),
            FrameError::Inconplete(ENCAPSULATION_HEADER_SIZE - 3)
        );
    }
}
