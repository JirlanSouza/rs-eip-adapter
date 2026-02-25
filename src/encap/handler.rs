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
        log::info!(
            "Received new request from transport: {:?}, command: {:?}",
            context.transport_type,
            req.header.command
        );
        log::debug!(
            "Received new request from transport: {:?}, header: {:?}, payload: {:?}",
            context.transport_type,
            req.header,
            req.payload
        );

        if !context.transport_type.is_valid_command(req.header.command) {
            if context.transport_type == TransportType::UDP(CastMode::Broadcast) {
                log::warn!(
                    "Invalid or unsupported command for UDP broadcast (command: {:?})",
                    req.header.command
                );
                return Ok(HandlerAction::None);
            }

            return self.handle_error_reply(
                &req.header,
                EncapsulationError::InvalidOrUnsupportedCommand(req.header.command.into()),
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
            log::info!("Received NOP command no reply to send");
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
                    return self.handle_error_reply(&req_encapsulation.header, p_error);
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
        log::info!("Dispatching command {:?}", req.header.command);
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
