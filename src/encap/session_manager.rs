use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::{
    collections::HashMap,
    io,
    sync::{Arc, RwLock},
};

use crate::encap::{ENCAPSULATION_PROTOCOL_VERSION, error::{EncapsulationError, FrameError, HandlerError, InternalError}};

pub const REGISTER_SESSION_DATA_SIZE: usize = 4;

pub struct Session {
    pub session_handle: u32,
}

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<u32, Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register_session(
        &self,
        in_buff: &mut Bytes,
        out_buff: &mut BytesMut,
    ) -> Result<u32, HandlerError> {
        let mut register_session_data = RegisterSessionData::decode(in_buff).map_err(|_| {
            HandlerError::Internal(InternalError::Other("Failed to decode register session data".to_string()))
        })?;

        if register_session_data.protocol_version > ENCAPSULATION_PROTOCOL_VERSION || register_session_data.options != 0 {
            register_session_data.protocol_version = ENCAPSULATION_PROTOCOL_VERSION;
            register_session_data.options = 0;
            register_session_data.encode(out_buff).map_err(|_| {
                HandlerError::Internal(InternalError::Other("Failed to encode register session data".to_string()))
            })?;
            return Err(HandlerError::from(EncapsulationError::UnsupportedProtocol));
        }

        self.create_session()
    }

    pub fn unregister_session(&self, session_handle: u32) -> Result<(), HandlerError> {
        let mut sessions = self.sessions.write().map_err(|_| {
            HandlerError::Internal(InternalError::Other("Failed to acquire lock".to_string()))
        })?;
        sessions.remove(&session_handle);
        Ok(())
    }

    pub fn is_valid_session(&self, session_handle: u32) -> Result<bool, HandlerError> {
        let sessions = self.sessions.read().map_err(|_| {
            HandlerError::Internal(InternalError::Other("Failed to acquire lock".to_string()))
        })?;
        Ok(sessions.get(&session_handle).is_some())
    }

    pub fn create_session(&self) -> Result<u32, HandlerError> {
        let mut sessions = self.sessions.write().map_err(|_| {
            HandlerError::Internal(InternalError::Other("Failed to acquire lock".to_string()))
        })?;
        let mut session_handle = sessions.len() as u32;

        while sessions.contains_key(&session_handle) || session_handle == 0 {
            session_handle += 1;
        }
        let session = Session { session_handle };
        sessions.insert(session_handle, session);
        Ok(session_handle)
    }
}

#[derive(Debug, PartialEq)]
pub struct RegisterSessionData {
    pub protocol_version: u16,
    pub options: u16,
}

impl RegisterSessionData {
    pub fn encode(&self, buf: &mut BytesMut) -> io::Result<()> {
        if buf.remaining_mut() < REGISTER_SESSION_DATA_SIZE + 4 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not enough space in buffer",
            ));
        }

        buf.put_u16_le(self.protocol_version);
        buf.put_u16_le(self.options);
        Ok(())
    }

    pub fn decode(buf: &mut Bytes) -> Result<Self, FrameError> {
        if buf.remaining() < REGISTER_SESSION_DATA_SIZE + 4 {
            return Err(FrameError::Inconplete(buf.remaining()));
        }

        let protocol_version = buf.get_u16_le();
        let options = buf.get_u16_le();
        Ok(Self {
            protocol_version,
            options,
        })
    }
}
