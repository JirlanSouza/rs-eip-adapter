use bytes::{Buf, BufMut};
use std::sync::Arc;

use crate::{
    common::binary::{BinaryError, FromBytes, ToBytes},
    encap::{
        Encapsulation, EncapsulationHeader,
        error::{EncapsulationError, HandlerError},
        handler::ConnectionContext,
        header::EncapsulationStatus,
        payload::EncapsulationPayload,
        session_manager::SessionManager,
    },
};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct RegisterSessionData {
    pub protocol_version: u16,
    pub options: u16,
}

impl RegisterSessionData {
    const LEN: usize = 4;
}

impl FromBytes for RegisterSessionData {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < RegisterSessionData::LEN {
            return Err(BinaryError::Truncated {
                expected: RegisterSessionData::LEN,
                actual: buffer.remaining(),
            });
        }

        let protocol_version = buffer.get_u16_le();
        let options = buffer.get_u16_le();
        Ok(Self {
            protocol_version,
            options,
        })
    }
}

impl ToBytes for RegisterSessionData {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < RegisterSessionData::LEN {
            return Err(BinaryError::BufferTooSmall {
                expected: RegisterSessionData::LEN,
                actual: buffer.remaining_mut(),
            });
        }

        buffer.put_u16_le(self.protocol_version);
        buffer.put_u16_le(self.options);
        Ok(())
    }

    fn encoded_len(&self) -> usize {
        RegisterSessionData::LEN
    }
}

pub struct RegisterSessionHandler {
    session_manager: Arc<SessionManager>,
}

impl RegisterSessionHandler {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    pub fn handle(
        &self,
        req_header: &EncapsulationHeader,
        req_payload: &RegisterSessionData,
        context: &mut ConnectionContext,
    ) -> Result<Encapsulation, HandlerError> {
        if req_payload.protocol_version > Encapsulation::VERSION || req_payload.options != 0 {
            let reply_payload = RegisterSessionData {
                protocol_version: Encapsulation::VERSION,
                options: 0,
            };
            return Err(HandlerError::from(EncapsulationError::UnsupportedProtocol(
                reply_payload,
            )));
        }

        let session_handle = self.session_manager.new_session();
        let reply_payload = RegisterSessionData {
            protocol_version: req_payload.protocol_version,
            options: 0,
        };

        context.session_handle = Some(session_handle);
        Ok(Encapsulation {
            header: EncapsulationHeader {
                status: EncapsulationStatus::Success,
                session_handle,
                ..req_header.clone()
            },
            payload: EncapsulationPayload::RegisterSession(reply_payload),
        })
    }
}
