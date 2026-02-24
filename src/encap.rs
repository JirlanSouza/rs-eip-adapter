use bytes::{BufMut, Bytes};

use self::{
    error::EncapsulationError,
    header::EncapsulationHeader,
    payload::{EncapsulationPayload, EncapsulationPayloadFromBytes},
};
use crate::common::binary::{BinaryError, ToBytes};

pub mod command;
pub mod cpf;
pub mod error;
pub mod handler;
pub mod header;
pub mod payload;
pub mod session_manager;

pub use handler::{CastMode, ConnectionContext, EncapsulationHandler, TransportType};

#[derive(Debug)]
pub struct RawEncapsulation {
    pub header: EncapsulationHeader,
    pub payload: Bytes,
}

impl RawEncapsulation {
    pub fn new(header: EncapsulationHeader, payload: Bytes) -> Self {
        Self { header, payload }
    }
}

impl TryFrom<&mut RawEncapsulation> for Encapsulation {
    type Error = (EncapsulationError, EncapsulationHeader);

    fn try_from(raw: &mut RawEncapsulation) -> Result<Self, Self::Error> {
        let payload = EncapsulationPayload::decode(raw.header.command, &mut raw.payload)
            .inspect_err(|err| log::warn!("Error decoding payload: {}", err))
            .map_err(|err| (err.into(), raw.header))?;

        Encapsulation::new(raw.header, payload).map_err(|err| (err, raw.header))
    }
}

#[derive(Debug)]
pub struct Encapsulation {
    pub header: EncapsulationHeader,
    pub payload: EncapsulationPayload,
}

impl Encapsulation {
    pub const VERSION: u16 = 1;

    pub fn new(
        header: EncapsulationHeader,
        payload: EncapsulationPayload,
    ) -> Result<Self, EncapsulationError> {
        let encapsulation = Self { header, payload };
        encapsulation.validate()?;
        Ok(encapsulation)
    }

    fn validate(&self) -> Result<(), EncapsulationError> {
        let payload_len = self.payload.encoded_len();
        let header_len = self.header.length as usize;

        if payload_len != header_len {
            return Err(EncapsulationError::InvalidLength {
                expected: header_len,
                actual: payload_len,
            });
        }
        Ok(())
    }
}

impl ToBytes for Encapsulation {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        let total_len = self.encoded_len();

        if buffer.remaining_mut() < total_len {
            return Err(BinaryError::BufferTooSmall {
                expected: total_len,
                actual: buffer.remaining_mut(),
            });
        }

        self.header.encode(buffer)?;
        self.payload.encode(buffer)?;
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        EncapsulationHeader::LEN + self.payload.encoded_len()
    }
}
