use crate::cip::registry::Registry;
use crate::encap::{
    Encapsulation,
    command::EncapsulationCommand,
    error::{EncapsulationError, HandlerError},
    header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader},
    list_identity::list_identity,
};
use bytes::{BufMut, Bytes, BytesMut};
use std::sync::Arc;

pub struct EncapsulationHandler {
    registry: Arc<Registry>,
}

impl EncapsulationHandler {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }

    pub fn handle_udp_broadcast(&self, in_buff: Bytes) -> Option<Bytes> {
        log::info!("Received UDP broadcast packet");
        let mut encapsulation = Encapsulation::decode(in_buff)?;
        log::debug!("Decoded UDP broadcast packet {:?}", encapsulation.header);

        if let Some(err) = encapsulation.validate_length() {
            log::warn!("Invalid length for UDP broadcast packet: {:?}", err);
            return self.handle_error_reply(&mut encapsulation.header, err);
        }

        if !self.is_valid_broadcast_command(encapsulation.header.command) {
            log::warn!(
                "Invalid or unsupported command: {:?} for udp broadcast request",
                encapsulation.header.command
            );
            return self.handle_error_reply(
                &mut encapsulation.header,
                EncapsulationError::InvalidOrUnsupportedCommand,
            );
        }

        self.handle_request(&mut encapsulation.header, &mut encapsulation.payload)
    }

    fn is_valid_broadcast_command(&self, command: EncapsulationCommand) -> bool {
        matches!(
            command,
            EncapsulationCommand::ListIdentity
                | EncapsulationCommand::ListInterfaces
                | EncapsulationCommand::ListServices
        )
    }

    fn handle_request(
        &self,
        header: &mut EncapsulationHeader,
        in_buf: &mut Bytes,
    ) -> Option<Bytes> {
        let mut out_buf = self.alloc_response_buffer();
        let result = self.dispatch(header, in_buf, &mut out_buf);
        self.handle_result(header, result, out_buf)
    }

    fn handle_error_reply(
        &self,
        header: &mut EncapsulationHeader,
        err: EncapsulationError,
    ) -> Option<Bytes> {
        let out_buf = self.alloc_response_buffer();
        self.handle_result(header, Err(HandlerError::from(err)), out_buf)
    }

    fn alloc_response_buffer(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
        buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE);
        buf
    }

    fn handle_result(
        &self,
        header: &mut EncapsulationHeader,
        result: Result<(), HandlerError>,
        mut out_buf: BytesMut,
    ) -> Option<Bytes> {
        log::info!("Handling result for command {:?}", header.command);
        if let Err(err) = result {
            if let HandlerError::Internal(e) = &err {
                log::warn!("Error on encapsulation: {}", e);
                return None;
            }
            self.update_header_on_error(header, err, &mut out_buf);
        } else {
            header.status = EncapsulationError::Success.to_u32();
            header.length = (out_buf.len() - ENCAPSULATION_HEADER_SIZE) as u16;
        }

        self.finalize_response(header, out_buf)
    }

    fn update_header_on_error(
        &self,
        header: &mut EncapsulationHeader,
        err: HandlerError,
        out_buf: &mut BytesMut,
    ) {
        log::warn!("Failed to dispatch command: {}", err);
        if let HandlerError::Protocol(e) = err {
            log::warn!("Error on encapsulation layer: {}", e);
            header.status = e.to_u32();
        }

        header.length = 0;
        out_buf.truncate(ENCAPSULATION_HEADER_SIZE);
    }

    fn finalize_response(
        &self,
        header: &mut EncapsulationHeader,
        mut out_buf: BytesMut,
    ) -> Option<Bytes> {
        if out_buf.len() < ENCAPSULATION_HEADER_SIZE {
            log::warn!("Output buffer too small");
            out_buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE - out_buf.len());
        }

        let mut header_view = &mut out_buf[0..ENCAPSULATION_HEADER_SIZE];
        match header.encode(&mut header_view) {
            Ok(()) => {
                log::trace!("Success encode encapsulation header: {:?}", header);
                Some(out_buf.freeze())
            }
            Err(err) => {
                log::error!("Failed to encode encapsulation header: {}", err);
                None
            }
        }
    }

    fn dispatch(
        &self,
        header: &EncapsulationHeader,
        payload: &mut Bytes,
        out_buf: &mut BytesMut,
    ) -> Result<(), HandlerError> {
        log::info!("Dispatching command {:?}", header.command);
        match header.command {
            EncapsulationCommand::Nop => Ok(()),
            EncapsulationCommand::ListIdentity => {
                if !payload.is_empty() {
                    log::warn!(
                        "Invalid payload for ListIdentity command: payload_length: {}",
                        payload.len()
                    );
                    return Err(HandlerError::from(EncapsulationError::InvalidLength));
                }

                list_identity(&self.registry, out_buf).map_err(|err| {
                    log::error!("Failed to list identity: {}", err);
                    HandlerError::from(err)
                })
            }
            _ => {
                log::warn!("Unsupported command: {:?}", header.command);
                Err(HandlerError::from(
                    EncapsulationError::InvalidOrUnsupportedCommand,
                ))
            }
        }
    }
}
