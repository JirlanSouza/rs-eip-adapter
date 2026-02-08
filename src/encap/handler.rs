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
        let mut encapsulation = Encapsulation::decode(in_buff)?;

        if let Some(err) = encapsulation.validate_length() {
            return self.handle_error_reply(&mut encapsulation.header, err);
        }

        if encapsulation.header.command != EncapsulationCommand::ListIdentity
            && encapsulation.header.command != EncapsulationCommand::ListInterfaces
            && encapsulation.header.command != EncapsulationCommand::ListServices
        {
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

    fn handle_request(
        &self,
        header: &mut EncapsulationHeader,
        mut in_buf: &Bytes,
    ) -> Option<Bytes> {
        let mut out_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
        out_buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE);
        let result = self.dispatch(&header, &mut in_buf, &mut out_buf);
        self.handle_result(header, result, out_buf)
    }

    fn handle_error_reply(
        &self,
        header: &mut EncapsulationHeader,
        err: EncapsulationError,
    ) -> Option<Bytes> {
        let mut out_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
        out_buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE);
        return self.handle_result(header, Err(HandlerError::from(err)), out_buf);
    }

    fn handle_result(
        &self,
        header: &mut EncapsulationHeader,
        result: Result<(), HandlerError>,
        mut out_buf: BytesMut,
    ) -> Option<Bytes> {
        if let Err(err) = result {
            log::warn!("Failed to dispatch command: {}", err);

            match err {
                HandlerError::Internal(err) => {
                    log::warn!("Error on encapsulation: {}", err);
                    return None;
                }
                HandlerError::Protocol(err) => {
                    log::warn!("Error on encapssulation layer: {}", err);
                    header.status = err.to_u32();
                }
            }

            header.length = 0;
            out_buf.truncate(ENCAPSULATION_HEADER_SIZE);
        } else {
            header.status = EncapsulationError::Success.to_u32();
            header.length = (out_buf.len() - ENCAPSULATION_HEADER_SIZE) as u16;
        }

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
        payload: &Bytes,
        out_buf: &mut BytesMut,
    ) -> Result<(), HandlerError> {
        match header.command {
            EncapsulationCommand::Nop => Ok(()),
            EncapsulationCommand::ListIdentity => {
                if !payload.is_empty() {
                    log::warn!("Invalid payload for ListIdentity command: payload_length: {}", payload.len());
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
