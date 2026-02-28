use bytes::{Bytes, BytesMut};

use super::error::CipError;

pub struct CipService {
    pub id: u8,
    pub name: String,
    handler: fn(&Bytes, &mut BytesMut) -> Result<(), CipError>,
}

impl CipService {
    pub fn new(
        id: u8,
        name: String,
        handler: fn(&Bytes, &mut BytesMut) -> Result<(), CipError>,
    ) -> Self {
        Self { id, name, handler }
    }

    pub fn execute(&self, data: &Bytes, response: &mut BytesMut) -> Result<(), CipError> {
        (self.handler)(data, response)
    }
}
