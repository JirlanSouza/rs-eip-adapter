use std::sync::Arc;

use crate::cip::registry::Registry;
use crate::common::binary::ToBytes;
use crate::encap::{
    Encapsulation, RawEncapsulation,
    command::{
        EncapsulationCommand, list_identity::ListIdentityHandler,
        register_session::RegisterSessionHandler,
    },
    error::{EncapsulationError, HandlerError, InternalError},
    header::EncapsulationHeader,
    payload::EncapsulationPayload,
    session_manager::SessionManager,
};

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
                        | EncapsulationCommand::RegisterSession
                        | EncapsulationCommand::UnregisterSession
                        | EncapsulationCommand::SendRRData
                        | EncapsulationCommand::SendUnitData
                )
            }
            TransportType::UDP(_) => {
                matches!(
                    command,
                    EncapsulationCommand::ListIdentity
                        | EncapsulationCommand::ListInterfaces
                        | EncapsulationCommand::ListServices
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
    pub transport_type: TransportType,
}

impl ConnectionContext {
    pub fn new(transport_type: TransportType) -> Self {
        Self {
            session_handle: None,
            transport_type,
        }
    }
}

pub struct EncapsulationHandler {
    _registry: Arc<Registry>,
    _session_manager: Arc<SessionManager>,
    list_identity_handler: ListIdentityHandler,
    register_session_handler: RegisterSessionHandler,
}

impl EncapsulationHandler {
    pub fn new(registry: Arc<Registry>, session_manager: Arc<SessionManager>) -> Self {
        Self {
            _registry: registry.clone(),
            _session_manager: session_manager.clone(),
            list_identity_handler: ListIdentityHandler::new(registry),
            register_session_handler: RegisterSessionHandler::new(session_manager),
        }
    }

    pub fn handle(
        &self,
        req: &mut RawEncapsulation,
        context: &mut ConnectionContext,
    ) -> Result<Encapsulation, InternalError> {
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
            return self.handle_error_reply(
                &req.header,
                EncapsulationError::InvalidOrUnsupportedCommand(req.header.command.into()),
            );
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
            Ok(encapsulation) => Ok(encapsulation),
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
    ) -> Result<Encapsulation, InternalError> {
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

        Ok(Encapsulation {
            header: reply_header,
            payload: reply_payload,
        })
    }
}

impl EncapsulationHandler {
    fn dispatch(
        &self,
        req: &Encapsulation,
        context: &mut ConnectionContext,
    ) -> Result<Encapsulation, HandlerError> {
        log::info!("Dispatching command {:?}", req.header.command);
        match req.header.command {
            EncapsulationCommand::Nop => Ok(Encapsulation {
                header: req.header.clone(),
                payload: EncapsulationPayload::None,
            }),
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
            _ => Err(HandlerError::from(
                EncapsulationError::InvalidOrUnsupportedCommand(req.header.command),
            )),
        }
    }
}
