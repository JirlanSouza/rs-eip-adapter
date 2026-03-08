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

#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};

    use super::*;
    use crate::encap::{
        command::EncapsulationCommand, handler::TransportType, header::EncapsulationStatus,
    };

    #[test]
    fn unregister_session_handler_success() {
        let handler = UnregisterSessionHandler;

        let header = EncapsulationHeader {
            command: EncapsulationCommand::UnregisterSession,
            length: 0,
            session_handle: 12345,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let peer_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 44818);
        let mut context = ConnectionContext::new(peer_addr, TransportType::TCP);
        context.session_handle = Some(12345);

        let action = handler
            .handle(&header, &mut context)
            .expect("Failed to handle command");

        assert_eq!(action, HandlerAction::DropConnection);
        assert_eq!(context.session_handle, None);
    }

    #[test]
    fn unregister_session_handler_invalid_length_fails() {
        let handler = UnregisterSessionHandler;

        let header = EncapsulationHeader {
            command: EncapsulationCommand::UnregisterSession,
            length: 4, // Invalid length, must be 0
            session_handle: 12345,
            status: EncapsulationStatus::Success,
            context: [0; 8],
            options: 0,
        };

        let peer_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 44818);
        let mut context = ConnectionContext::new(peer_addr, TransportType::TCP);
        context.session_handle = Some(12345);

        let result = handler.handle(&header, &mut context);

        if let Err(HandlerError::Protocol(EncapsulationError::InvalidLength { expected, actual })) =
            result
        {
            assert_eq!(expected, 0);
            assert_eq!(actual, 4);
        } else {
            panic!("Expected InvalidLength error");
        }
    }
}
