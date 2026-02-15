use crate::encap::{
    Encapsulation,
    error::{EncapsulationError, HandlerError},
    header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader},
};
use bytes::{BufMut, Bytes, BytesMut};

pub trait EncapsulationHandler {
    fn handle_request(&self, encapsulation: &mut Encapsulation) -> Option<Bytes> {
        let mut out_buf = self.alloc_response_buffer();
        let result = self.dispatch(encapsulation, &mut out_buf);
        self.handle_result(&mut encapsulation.header, result, out_buf)
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
        encapsulation: &mut Encapsulation,
        out_buf: &mut BytesMut,
    ) -> Result<(), HandlerError>;
}
