use crate::{
    cip::registry::Registry,
    encap::{
        Encapsulation, command::EncapsulationCommand, error::EncapsulationError,
        error::HandlerError, handler::EncapsulationHandler, list_identity::list_identity,
    },
};
use bytes::Bytes;
use std::sync::Arc;

pub struct BroadcastHandler {
    registry: Arc<Registry>,
}

impl BroadcastHandler {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }

    pub fn handle(&self, encapsulation: &mut Encapsulation) -> Option<Bytes> {
        if !self.is_valid_command(encapsulation.header.command) {
            log::warn!(
                "Invalid or unsupported command: {:?} for udp broadcast request",
                encapsulation.header.command
            );
            return self.handle_error_reply(
                &mut encapsulation.header,
                EncapsulationError::InvalidOrUnsupportedCommand,
            );
        }

        self.handle_request(encapsulation)
    }

    fn is_valid_command(&self, command: EncapsulationCommand) -> bool {
        matches!(
            command,
            EncapsulationCommand::ListIdentity
                | EncapsulationCommand::ListInterfaces
                | EncapsulationCommand::ListServices
        )
    }
}

impl EncapsulationHandler for BroadcastHandler {
    fn dispatch(
        &self,
        encapsulation: &mut Encapsulation,
        out_buf: &mut bytes::BytesMut,
    ) -> Result<(), HandlerError> {
        log::info!("Dispatching command {:?}", encapsulation.header.command);
        match encapsulation.header.command {
            EncapsulationCommand::Nop => Ok(()),
            EncapsulationCommand::ListIdentity => {
                if !encapsulation.payload.is_empty() {
                    log::warn!(
                        "Invalid payload for ListIdentity command: payload_length: {}",
                        encapsulation.payload.len()
                    );
                    return Err(HandlerError::from(EncapsulationError::InvalidLength));
                }

                list_identity(&self.registry, out_buf).map_err(|err| {
                    log::error!("Failed to list identity: {}", err);
                    HandlerError::from(err)
                })
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
