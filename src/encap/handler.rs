use std::{net::SocketAddr, sync::Arc};

use super::{
    Encapsulation, RawEncapsulation,
    command::{
        EncapsulationCommand, list_identity::ListIdentityHandler,
        register_session::RegisterSessionHandler, unregister_session::UnregisterSessionHandler,
    },
    error::{EncapsulationError, HandlerError, InternalError},
    header::{EncapsulationHeader, EncapsulationStatus},
    payload::EncapsulationPayload,
    session_manager::SessionManager,
};
use crate::cip::registry::Registry;
use crate::common::binary::ToBytes;

#[derive(Debug, PartialEq)]
pub enum TransportType {
    TCP,
    UDP(CastMode),
}

impl TransportType {
    fn is_valid_command(&self, command: EncapsulationCommand) -> bool {
        match self {
            TransportType::TCP => {
                matches!(
                    command,
                    EncapsulationCommand::Nop
                        | EncapsulationCommand::ListServices
                        | EncapsulationCommand::ListIdentity
                        | EncapsulationCommand::ListInterfaces
                        | EncapsulationCommand::RegisterSession
                        | EncapsulationCommand::UnregisterSession
                        | EncapsulationCommand::SendRRData
                        | EncapsulationCommand::SendUnitData
                        | EncapsulationCommand::IndicateStatus
                        | EncapsulationCommand::Cancel
                )
            }
            TransportType::UDP(_) => {
                matches!(
                    command,
                    EncapsulationCommand::ListServices
                        | EncapsulationCommand::ListIdentity
                        | EncapsulationCommand::ListInterfaces
                )
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum CastMode {
    Unicast,
    Multicast,
    Broadcast,
}

pub struct ConnectionContext {
    pub session_handle: Option<u32>,
    pub peer_addr: SocketAddr,
    pub transport_type: TransportType,
}

impl ConnectionContext {
    pub fn new(peer_addr: SocketAddr, transport_type: TransportType) -> Self {
        Self {
            session_handle: None,
            peer_addr,
            transport_type,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum HandlerAction {
    Reply(Encapsulation),
    DropConnection,
    None,
}

pub struct EncapsulationHandler {
    _registry: Arc<Registry>,
    _session_manager: Arc<SessionManager>,
    list_identity_handler: ListIdentityHandler,
    register_session_handler: RegisterSessionHandler,
    unregister_session_handler: UnregisterSessionHandler,
}

impl EncapsulationHandler {
    pub fn new(registry: Arc<Registry>, session_manager: Arc<SessionManager>) -> Self {
        Self {
            _registry: registry.clone(),
            _session_manager: session_manager.clone(),
            list_identity_handler: ListIdentityHandler::new(registry),
            register_session_handler: RegisterSessionHandler::new(session_manager),
            unregister_session_handler: UnregisterSessionHandler,
        }
    }

    pub fn handle(
        &self,
        req: &mut RawEncapsulation,
        context: &mut ConnectionContext,
    ) -> Result<HandlerAction, InternalError> {
        log::debug!(
            "Received new request from transport: {:?}, header: {:?}, payload: {:?}",
            context.transport_type,
            req.header,
            req.payload
        );

        if !context.transport_type.is_valid_command(req.header.command) {
            if let TransportType::UDP(_) = &context.transport_type {
                log::warn!(
                    "Invalid or unsupported command for UDP (command: {:?})",
                    req.header.command
                );
                return Ok(HandlerAction::None);
            }

            return self.handle_error_reply(
                &req.header,
                EncapsulationError::InvalidOrUnsupportedCommand(req.header.command),
            );
        }

        if req.header.status != EncapsulationStatus::Success {
            log::warn!(
                "Invalid status for request (command: {:?}, status: {:?})",
                req.header.command,
                req.header.status
            );
            return Ok(HandlerAction::None);
        }

        if req.header.command == EncapsulationCommand::Nop {
            log::debug!("Received NOP command no reply to send");
            return Ok(HandlerAction::None);
        }

        let req_encapsulation = match Encapsulation::try_from(req) {
            Ok(encapsulation) => encapsulation,
            Err((error, header)) => return self.handle_error_reply(&header, error),
        };

        log::debug!(
            "Decoded raw encapsulation payload header: {:?}, payload: {:?}",
            req_encapsulation.header,
            req_encapsulation.payload
        );

        match self.dispatch(&req_encapsulation, context) {
            Ok(action) => Ok(action),
            Err(error) => match error {
                HandlerError::Protocol(p_error) => {
                    self.handle_error_reply(&req_encapsulation.header, p_error)
                }
                _ => Err(InternalError::from(error.to_string())),
            },
        }
    }

    fn handle_error_reply(
        &self,
        header: &EncapsulationHeader,
        error: EncapsulationError,
    ) -> Result<HandlerAction, InternalError> {
        log::warn!(
            "Handling error reply for command: {:?}, error: {:?}",
            header.command,
            error
        );

        let reply_payload = match error {
            EncapsulationError::UnsupportedProtocol(data) => {
                EncapsulationPayload::RegisterSession(data)
            }
            _ => EncapsulationPayload::None,
        };

        let reply_header =
            header.clone_with_error_and_length(error, reply_payload.encoded_len() as u16);

        log::debug!(
            "Sending error reply header: {:?}, payload: {:?}",
            reply_header,
            reply_payload
        );

        Ok(HandlerAction::Reply(Encapsulation {
            header: reply_header,
            payload: reply_payload,
        }))
    }

    fn dispatch(
        &self,
        req: &Encapsulation,
        context: &mut ConnectionContext,
    ) -> Result<HandlerAction, HandlerError> {
        log::debug!("Dispatching command {:?}", req.header.command);
        match req.header.command {
            EncapsulationCommand::ListIdentity => {
                if let EncapsulationPayload::None = req.payload {
                    return self.list_identity_handler.handle(&req.header);
                }

                Err(HandlerError::from(EncapsulationError::InvalidLength {
                    expected: 0,
                    actual: req.payload.encoded_len(),
                }))
            }
            EncapsulationCommand::RegisterSession => {
                if let EncapsulationPayload::RegisterSession(data) = req.payload {
                    self.register_session_handler
                        .handle(&req.header, &data, context)
                } else {
                    Err(HandlerError::from(InternalError::Other(
                        "Invalid payload to register session".to_string(),
                    )))
                }
            }
            EncapsulationCommand::UnregisterSession => {
                if let EncapsulationPayload::None = req.payload {
                    return self.unregister_session_handler.handle(&req.header, context);
                }

                Err(HandlerError::from(EncapsulationError::InvalidLength {
                    expected: 0,
                    actual: req.payload.encoded_len(),
                }))
            }
            _ => Err(HandlerError::from(
                EncapsulationError::InvalidOrUnsupportedCommand(req.header.command),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{error::Error, sync::Arc};

    use bytes::{Bytes, BytesMut};

    use super::*;
    use crate::cip::registry::Registry;
    use crate::common::binary::ToBytes;
    use crate::encap::{
        command::register_session::RegisterSessionData,
        header::{EncapsulationHeader, EncapsulationStatus},
        payload::EncapsulationPayload,
        session_manager::SessionManager,
    };

    fn setup_handler() -> EncapsulationHandler {
        let registry = Arc::new(Registry::new());
        let session_manager = Arc::new(SessionManager::new());
        EncapsulationHandler::new(registry, session_manager)
    }

    fn setup_context(transport: TransportType) -> ConnectionContext {
        ConnectionContext::new("127.0.0.1:12345".parse().unwrap(), transport)
    }

    #[test]
    fn transport_type_is_valid_command_tcp_success() {
        let tcp = TransportType::TCP;
        assert!(
            tcp.is_valid_command(EncapsulationCommand::ListIdentity),
            "TCP should support ListIdentity"
        );
        assert!(
            tcp.is_valid_command(EncapsulationCommand::RegisterSession),
            "TCP should support RegisterSession"
        );
    }

    #[test]
    fn transport_type_is_valid_command_udp_unicast_success() {
        let udp = TransportType::UDP(CastMode::Unicast);
        assert!(
            udp.is_valid_command(EncapsulationCommand::ListIdentity),
            "UDP unicast should support ListIdentity"
        );
        assert!(
            !udp.is_valid_command(EncapsulationCommand::RegisterSession),
            "UDP unicast should NOT support RegisterSession"
        );
    }

    #[test]
    fn handler_handle_nop_command_returns_none() -> Result<(), Box<dyn Error>> {
        let handler = setup_handler();
        let mut context = setup_context(TransportType::TCP);
        let mut req = RawEncapsulation {
            header: EncapsulationHeader {
                command: EncapsulationCommand::Nop,
                ..Default::default()
            },
            payload: Bytes::new(),
        };

        let result = handler.handle(&mut req, &mut context)?;
        assert_eq!(
            result,
            HandlerAction::None,
            "NOP command should return None action"
        );
        Ok(())
    }

    #[test]
    fn handler_handle_invalid_command_for_transport_returns_none_for_udp()
    -> Result<(), Box<dyn Error>> {
        let handler = setup_handler();
        let mut context = setup_context(TransportType::UDP(CastMode::Unicast));
        let mut req = RawEncapsulation {
            header: EncapsulationHeader {
                command: EncapsulationCommand::RegisterSession,
                ..Default::default()
            },
            payload: Bytes::new(),
        };

        let result = handler.handle(&mut req, &mut context)?;
        assert_eq!(
            result,
            HandlerAction::None,
            "Invalid command for UDP should return None action"
        );
        Ok(())
    }

    #[test]
    fn handler_handle_invalid_status_header_returns_none() -> Result<(), Box<dyn Error>> {
        let handler = setup_handler();
        let mut context = setup_context(TransportType::TCP);
        let mut req = RawEncapsulation {
            header: EncapsulationHeader {
                command: EncapsulationCommand::ListIdentity,
                status: EncapsulationStatus::IncorrectData,
                ..Default::default()
            },
            payload: Bytes::new(),
        };

        let result = handler.handle(&mut req, &mut context)?;
        assert_eq!(
            result,
            HandlerAction::None,
            "Non-success status in header should result in HandlerAction::None"
        );
        Ok(())
    }

    #[test]
    fn handler_handle_registration_success() -> Result<(), Box<dyn Error>> {
        let handler = setup_handler();
        let mut context = setup_context(TransportType::TCP);

        let reg_data = RegisterSessionData {
            protocol_version: 1,
            options: 0,
        };
        let mut payload_buf = BytesMut::with_capacity(reg_data.encoded_len());
        reg_data.encode(&mut payload_buf)?;

        let mut req = RawEncapsulation {
            header: EncapsulationHeader {
                command: EncapsulationCommand::RegisterSession,
                length: payload_buf.len() as u16,
                ..Default::default()
            },
            payload: payload_buf.freeze(),
        };

        let result = handler.handle(&mut req, &mut context)?;
        if let HandlerAction::Reply(encap) = result {
            assert_eq!(
                encap.header.command,
                EncapsulationCommand::RegisterSession,
                "Reply should be RegisterSession"
            );
            assert_eq!(
                encap.header.status,
                EncapsulationStatus::Success,
                "Reply status should be Success"
            );
            if let EncapsulationPayload::RegisterSession(data) = encap.payload {
                assert_eq!(data.protocol_version, 1, "Protocol version mismatch");
            } else {
                panic!("Expected RegisterSession payload");
            }
        } else {
            panic!("Expected HandlerAction::Reply");
        }

        assert!(
            context.session_handle.is_some(),
            "Session handle should be set in context"
        );
        Ok(())
    }

    #[test]
    fn handler_handle_unregistration_success() -> Result<(), Box<dyn Error>> {
        let handler = setup_handler();
        let mut context = setup_context(TransportType::TCP);
        context.session_handle = Some(123);

        let mut req = RawEncapsulation {
            header: EncapsulationHeader {
                command: EncapsulationCommand::UnregisterSession,
                session_handle: 123,
                ..Default::default()
            },
            payload: Bytes::new(),
        };

        let result = handler.handle(&mut req, &mut context)?;
        assert_eq!(
            result,
            HandlerAction::DropConnection,
            "UnregisterSession should return DropConnection action"
        );
        assert!(
            context.session_handle.is_none(),
            "Session handle should be cleared"
        );
        Ok(())
    }

    #[test]
    fn handler_handle_error_reply_formatting() -> Result<(), Box<dyn Error>> {
        let handler = setup_handler();
        let header = EncapsulationHeader {
            command: EncapsulationCommand::RegisterSession,
            ..Default::default()
        };

        let error_data = RegisterSessionData {
            protocol_version: 1,
            options: 0,
        };

        let result = handler
            .handle_error_reply(&header, EncapsulationError::UnsupportedProtocol(error_data));
        if let Ok(HandlerAction::Reply(encap)) = result {
            assert_eq!(
                encap.header.status,
                EncapsulationStatus::UnsupportedProtocol,
                "Status mismatch"
            );
            assert_eq!(encap.header.length, 4, "Length mismatch in header");
            if let EncapsulationPayload::RegisterSession(data) = encap.payload {
                assert_eq!(data.protocol_version, 1, "Payload data mismatch");
            } else {
                panic!("Expected RegisterSession payload data for UnsupportedProtocol error");
            }
        } else {
            panic!("Expected HandlerAction::Reply");
        }
        Ok(())
    }
}
