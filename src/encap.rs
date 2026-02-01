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
