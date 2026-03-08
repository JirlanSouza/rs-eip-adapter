use std::sync::Arc;

use bytes::{Buf, BufMut};

use crate::{
    common::binary::{BinaryError, FromBytes, ToBytes},
    encap::{
        Encapsulation, EncapsulationHeader,
        error::{EncapsulationError, HandlerError},
        handler::{ConnectionContext, HandlerAction},
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
    ) -> Result<HandlerAction, HandlerError> {
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
        Ok(HandlerAction::Reply(Encapsulation {
            header: EncapsulationHeader {
                status: EncapsulationStatus::Success,
                session_handle,
                ..*req_header
            },
            payload: EncapsulationPayload::RegisterSession(reply_payload),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};

    use bytes::{Bytes, BytesMut};

    use super::*;
    use crate::{
        common::binary::BinaryError,
        encap::{
            command::EncapsulationCommand, error::EncapsulationError, handler::TransportType,
            header::EncapsulationStatus,
        },
    };

    #[test]
    fn register_session_data_decode_success() {
        let raw_bytes: [u8; 4] = [
            0x01, 0x00, // Protocol version 1
            0x00, 0x00, // Options 0
        ];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = RegisterSessionData::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(decoded.protocol_version, 1);
        assert_eq!(decoded.options, 0);

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");

        assert_eq!(
            buffer.as_ref(),
            &raw_bytes,
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn register_session_data_decode_truncated_buffer_fails() {
        let raw_bytes: [u8; 3] = [0x01, 0x00, 0x00];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let result = RegisterSessionData::decode(&mut cursor);

        assert!(matches!(
            result,
            Err(BinaryError::Truncated {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn register_session_data_encode_small_buffer_fails() {
        let data = RegisterSessionData {
            protocol_version: 1,
            options: 0,
        };
        let mut raw = [0u8; 3];
        let mut buffer = &mut raw[..];

        let result = data.encode(&mut buffer);

        assert!(matches!(
            result,
            Err(BinaryError::BufferTooSmall {
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn register_session_handler_success() {
        let session_manager = Arc::new(SessionManager::new());
        let handler = RegisterSessionHandler::new(session_manager);

        let header = EncapsulationHeader {
            command: EncapsulationCommand::RegisterSession,
            length: 4,
            session_handle: 0,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let payload = RegisterSessionData {
            protocol_version: 1,
            options: 0,
        };

        let peer_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 44818);
        let mut context = ConnectionContext::new(peer_addr, TransportType::TCP);

        let action = handler
            .handle(&header, &payload, &mut context)
            .expect("Failed to handle command");

        match action {
            HandlerAction::Reply(reply) => {
                assert_eq!(reply.header.status, EncapsulationStatus::Success);
                assert!(reply.header.session_handle > 0);
                assert_eq!(context.session_handle, Some(reply.header.session_handle));

                match reply.payload {
                    EncapsulationPayload::RegisterSession(reply_payload) => {
                        assert_eq!(reply_payload.protocol_version, 1);
                        assert_eq!(reply_payload.options, 0);
                    }
                    _ => panic!("Expected EncapsulationPayload::RegisterSession"),
                }
            }
            _ => panic!("Expected HandlerAction::Reply"),
        }
    }

    #[test]
    fn register_session_handler_unsupported_protocol_fails() {
        let session_manager = Arc::new(SessionManager::new());
        let handler = RegisterSessionHandler::new(session_manager);

        let header = EncapsulationHeader {
            command: EncapsulationCommand::RegisterSession,
            length: 4,
            session_handle: 0,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let payload = RegisterSessionData {
            protocol_version: Encapsulation::VERSION + 1,
            options: 0,
        };

        let peer_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 44818);
        let mut context = ConnectionContext::new(peer_addr, TransportType::TCP);

        let result = handler.handle(&header, &payload, &mut context);

        if let Err(HandlerError::Protocol(EncapsulationError::UnsupportedProtocol(data))) = result {
            assert_eq!(data.protocol_version, Encapsulation::VERSION);
            assert_eq!(data.options, 0);
        } else {
            panic!("Expected UnsupportedProtocol error");
        }
    }

    #[test]
    fn register_session_handler_unsupported_options_fails() {
        let session_manager = Arc::new(SessionManager::new());
        let handler = RegisterSessionHandler::new(session_manager);

        let header = EncapsulationHeader {
            command: EncapsulationCommand::RegisterSession,
            length: 4,
            session_handle: 0,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let payload = RegisterSessionData {
            protocol_version: Encapsulation::VERSION,
            options: 1,
        };

        let peer_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 44818);
        let mut context = ConnectionContext::new(peer_addr, TransportType::TCP);

        let result = handler.handle(&header, &payload, &mut context);

        if let Err(HandlerError::Protocol(EncapsulationError::UnsupportedProtocol(data))) = result {
            assert_eq!(data.protocol_version, Encapsulation::VERSION);
            assert_eq!(data.options, 0);
        } else {
            panic!("Expected UnsupportedProtocol error");
        }
    }
}
