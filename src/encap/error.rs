use std::io;

use thiserror::Error;

use super::{
    EncapsulationStatus,
    command::{EncapsulationCommand, register_session::RegisterSessionData},
};
use crate::cip::registry_error::RegistryError;
use crate::common::binary::BinaryError;

#[derive(Debug, Copy, Clone, PartialEq, Error)]
pub enum EncapsulationError {
    #[error("invalid or unsupported command: {0}")]
    InvalidOrUnsupportedCommand(EncapsulationCommand),

    #[error("insufficient memory")]
    InsufficientMemory,

    #[error("incorrect data")]
    IncorrectData,

    #[error("invalid session handle: {0}")]
    InvalidSessionHandle(u32),

    #[error("invalid length (expected: {expected}, actual: {actual})")]
    InvalidLength { expected: usize, actual: usize },

    #[error("unsupported protocol ({0})")]
    UnsupportedProtocol(RegisterSessionData),
}

impl From<EncapsulationError> for EncapsulationStatus {
    fn from(value: EncapsulationError) -> Self {
        match value {
            EncapsulationError::InvalidOrUnsupportedCommand(_) => Self::InvalidOrUnsupportedCommand,
            EncapsulationError::InsufficientMemory => Self::InsufficientMemory,
            EncapsulationError::IncorrectData => Self::IncorrectData,
            EncapsulationError::InvalidSessionHandle(_) => Self::InvalidSessionHandle,
            EncapsulationError::InvalidLength { .. } => Self::InvalidLength,
            EncapsulationError::UnsupportedProtocol(_) => Self::UnsupportedProtocol,
        }
    }
}

impl From<BinaryError> for EncapsulationError {
    fn from(err: BinaryError) -> Self {
        match err {
            BinaryError::BufferTooSmall { expected, actual } => {
                Self::InvalidLength { expected, actual }
            }
            BinaryError::InvalidData {
                message: _,
                expected: _,
                actual: _,
            } => Self::IncorrectData,
            BinaryError::Truncated { expected, actual } => Self::InvalidLength { expected, actual },
        }
    }
}

#[derive(Debug, Error)]
pub enum InternalError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("other error: {0}")]
    Other(String),
}

impl From<String> for InternalError {
    fn from(err: String) -> Self {
        InternalError::Other(err)
    }
}

#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("protocol error: {0}")]
    Protocol(#[from] EncapsulationError),

    #[error("internal error: {0}")]
    Internal(#[from] InternalError),
}

impl From<String> for HandlerError {
    fn from(err: String) -> Self {
        HandlerError::Internal(InternalError::Other(err))
    }
}

impl From<RegistryError> for HandlerError {
    fn from(err: RegistryError) -> Self {
        HandlerError::Internal(InternalError::Other(err.to_string()))
    }
}
