use crate::encap::{
    ConnectionContext, EncapsulationError, EncapsulationHeader, HandlerError,
    handler::HandlerAction,
};

pub struct UnregisterSessionHandler;

impl UnregisterSessionHandler {
    pub fn handle(
        &self,
        header: &EncapsulationHeader,
        context: &mut ConnectionContext,
    ) -> Result<HandlerAction, HandlerError> {
        if header.length != 0 {
            return Err(EncapsulationError::InvalidLength {
                expected: 0,
                actual: header.length as usize,
            }
            .into());
        }

        context.session_handle = None;
        log::info!("Session unregistered: {}", header.session_handle);

        Ok(HandlerAction::DropConnection)
    }
}
