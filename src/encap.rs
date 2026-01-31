use crate::cip::registry::Registry;
use bytes::{BufMut, Bytes, BytesMut};
use command::EncapsulationCommand;
use error::EncapsulationError;
use header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader};
use list_identity::list_identity;
use std::sync::Arc;

pub mod command;
mod cpf;
pub mod error;
pub mod header;
mod list_identity;

pub const ENCAPSULATION_PROTOCOL_VERSION: u16 = 1;

struct Encapsulation {
    header: EncapsulationHeader,
    payload: Bytes,
}

impl Encapsulation {
    fn decode(mut in_buff: Bytes) -> Option<Self> {
        let header = EncapsulationHeader::decode(&mut in_buff)?;
        let payload = in_buff;
        Some(Self { header, payload })
    }
}

pub struct EncapsulationHandler {
    registry: Arc<Registry>,
}

impl EncapsulationHandler {
    pub fn new(registry: Arc<Registry>) -> Self {
        Self { registry }
    }

    pub fn handle_udp_broadcast(&self, in_buff: Bytes) -> Option<Bytes> {
        let mut encapsulation = Encapsulation::decode(in_buff)?;

        if let Some(out_buf) =
            self.validate_length(&mut encapsulation.header, encapsulation.payload.len())
        {
            return Some(out_buf);
        }

        if encapsulation.header.command != EncapsulationCommand::ListIdentity
            && encapsulation.header.command != EncapsulationCommand::ListInterfaces
            && encapsulation.header.command != EncapsulationCommand::ListServices
        {
            log::warn!(
                "Invalid or unsupported command: {:?} for udp broadcast request",
                encapsulation.header.command
            );
            let mut out_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
            out_buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE);
            let result = Err(Some(EncapsulationError::InvalidOrUnsupportedCommand));
            return self.handle_result(&mut encapsulation.header, result, out_buf);
        }

        self.handle_request(&mut encapsulation.header, &mut encapsulation.payload)
    }

    fn validate_length(
        &self,
        header: &mut EncapsulationHeader,
        payload_len: usize,
    ) -> Option<Bytes> {
        if payload_len != header.length as usize {
            log::warn!(
                "Invalid payload length: expected {}, got {}",
                header.length,
                payload_len
            );
            let mut out_buf = BytesMut::with_capacity(ENCAPSULATION_HEADER_SIZE);
            out_buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE);
            let result = Err(Some(EncapsulationError::InvalidOrUnsupportedCommand));
            return self.handle_result(header, result, out_buf);
        } else {
            None
        }
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

    fn handle_result(
        &self,
        header: &mut EncapsulationHeader,
        result: Result<(), Option<EncapsulationError>>,
        mut out_buf: BytesMut,
    ) -> Option<Bytes> {
        if let Err(err) = result {
            if let Some(err) = err {
                log::warn!("Failed to dispatch command: {}", err);
                header.status = err.into();
            } else {
                log::error!("Failed to dispatch command: Unknown error");
                return None;
            }

            header.length = 0;
            out_buf.truncate(ENCAPSULATION_HEADER_SIZE);
        } else {
            header.status = EncapsulationError::Success.into();
            header.length = (out_buf.len() - ENCAPSULATION_HEADER_SIZE) as u16;
        }

        if out_buf.len() < ENCAPSULATION_HEADER_SIZE {
            log::warn!("Output buffer too small");
            out_buf.put_bytes(0, ENCAPSULATION_HEADER_SIZE - out_buf.len());
        }

        let mut header_view = &mut out_buf[0..ENCAPSULATION_HEADER_SIZE];
        match header.encode(&mut header_view) {
            Ok(()) => Some(out_buf.freeze()),
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
    ) -> Result<(), Option<EncapsulationError>> {
        match header.command {
            EncapsulationCommand::Nop => Ok(()),
            EncapsulationCommand::ListIdentity => {
                if header.length != 0 || payload.len() != 0 {
                    log::warn!(
                        "Invalid payload length for ListIdentity command: expected 0, got {}",
                        payload.len()
                    );
                    return Err(Some(EncapsulationError::InvalidLength));
                }

                list_identity(&self.registry, out_buf).map_err(|err| {
                    log::error!("Failed to list identity: {}", err);
                    None
                })
            }
            _ => {
                log::warn!("Unsupported command: {:?}", header.command);
                Err(Some(EncapsulationError::InvalidOrUnsupportedCommand))
            }
        }
    }
}
