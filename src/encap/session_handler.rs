use crate::{
    cip::registry::Registry,
    encap::{
        Encapsulation,
        command::EncapsulationCommand,
        error::{EncapsulationError, HandlerError},
        handler::EncapsulationHandler,
        session_manager::SessionManager,
    },
};
use bytes::Bytes;
use std::sync::Arc;

pub struct SessionHandler {
    registry: Arc<Registry>,
    session_manager: Arc<SessionManager>,
}

impl SessionHandler {
    pub fn new(registry: Arc<Registry>, session_manager: Arc<SessionManager>) -> Self {
        Self {
            registry,
            session_manager,
        }
    }

    pub fn handle(&self, mut encapsulation: Encapsulation) -> Option<Bytes> {
        if !self.is_valid_command(encapsulation.header.command) {
            log::warn!(
                "Invalid or unsupported command: {:?} for tcp message request",
                encapsulation.header.command
            );
            return self.handle_error_reply(
                &mut encapsulation.header,
                EncapsulationError::InvalidOrUnsupportedCommand,
            );
        }

        self.handle_request(&mut encapsulation)
    }

    fn is_valid_command(&self, command: EncapsulationCommand) -> bool {
        matches!(
            command,
            EncapsulationCommand::Nop
                | EncapsulationCommand::RegisterSession
                | EncapsulationCommand::UnregisterSession
                | EncapsulationCommand::SendRRData
                | EncapsulationCommand::SendUnitData
        )
    }
}

impl EncapsulationHandler for SessionHandler {
    fn dispatch(
        &self,
        encapsulation: &mut Encapsulation,
        out_buf: &mut bytes::BytesMut,
    ) -> Result<(), HandlerError> {
        log::info!("Dispatching command {:?}", encapsulation.header.command);
        match encapsulation.header.command {
            EncapsulationCommand::Nop => Ok(()),
            EncapsulationCommand::RegisterSession => {
                if !encapsulation.payload.is_empty() {
                    log::warn!(
                        "Invalid payload for RegisterSession command: payload_length: {}",
                        encapsulation.payload.len()
                    );
                    return Err(HandlerError::from(EncapsulationError::InvalidLength));
                }

                let session_handle = self
                    .session_manager
                    .register_session(&mut encapsulation.payload, out_buf)?;
                encapsulation.header.session_handle = session_handle;
                Ok(())
            }
            _ => {
                log::warn!("Unsupported command: {:?}", encapsulation.header.command);
                Err(HandlerError::from(
                    EncapsulationError::InvalidOrUnsupportedCommand,
                ))
            }
        }
    }
}
