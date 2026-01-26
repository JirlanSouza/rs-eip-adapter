use crate::cip::registry::Registry;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use command::EncapsulationCommand;
use error::EncapsulationError;
use list_identity::list_identity;
use std::{io, sync::Arc};

pub mod command;
mod cpf;
pub mod error;
mod list_identity;

pub const ENCAPSULATION_PROTOCOL_VERSION: u16 = 1;
pub const ENCAPSULATION_HEADER_SIZE: usize = 24;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncapsulationHeader {
    pub command: EncapsulationCommand,
    pub length: u16,
    pub session_handle: u32,
    pub status: u32,
    pub context: [u8; 8],
    pub options: u32,
}

impl EncapsulationHeader {
    pub fn decode(in_buff: &mut Bytes) -> Option<Self> {
        if in_buff.remaining() < ENCAPSULATION_HEADER_SIZE {
            return None;
        }

        let command = EncapsulationCommand::from_u16(in_buff.get_u16_le());
        let length = in_buff.get_u16_le();
        let session_handle = in_buff.get_u32_le();
        let status = in_buff.get_u32_le();
        let mut context = [0u8; 8];
        in_buff.copy_to_slice(&mut context);
        let options = in_buff.get_u32_le();
        Some(Self {
            command,
            length,
            session_handle,
            status,
            context,
            options,
        })
    }

    pub fn encode<T: BufMut>(&self, out_buff: &mut T) -> io::Result<()> {
        if out_buff.remaining_mut() < ENCAPSULATION_HEADER_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Insufficient buffer space",
            ));
        }

        out_buff.put_u16_le(self.command.to_u16());
        out_buff.put_u16_le(self.length);
        out_buff.put_u32_le(self.session_handle);
        out_buff.put_u32_le(self.status);
        out_buff.put_slice(&self.context);
        out_buff.put_u32_le(self.options);
        Ok(())
    }
}

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
