use std::{fmt::Display, io};

use super::{
    EncapsulationStatus,
    command::{EncapsulationCommand, register_session::RegisterSessionData},
};
use crate::common::binary::BinaryError;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum EncapsulationError {
    InvalidOrUnsupportedCommand(EncapsulationCommand),
    InsufficientMemory,
    IncorrectData,
    InvalidSessionHandle(u32),
    InvalidLength { expected: usize, actual: usize },
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

impl Display for EncapsulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncapsulationError::InvalidOrUnsupportedCommand(_) => {
                write!(f, "Invalid or unsupported command")
            }
            EncapsulationError::InsufficientMemory => write!(f, "Insufficient memory"),
            EncapsulationError::IncorrectData => write!(f, "Incorrect data"),
            EncapsulationError::InvalidSessionHandle(_) => write!(f, "Invalid session handle"),
            EncapsulationError::InvalidLength { expected, actual } => {
                write!(
                    f,
                    "Invalid length: expected {}, actual: {}",
                    expected, actual
                )
            }
            EncapsulationError::UnsupportedProtocol(_) => write!(f, "Unsupported protocol"),
        }
    }
}

impl std::error::Error for EncapsulationError {}

#[derive(Debug)]
pub enum InternalError {
    Io(io::Error),
    Other(String),
}

impl From<io::Error> for InternalError {
    fn from(err: io::Error) -> Self {
        InternalError::Io(err)
    }
}

impl From<String> for InternalError {
    fn from(err: String) -> Self {
        InternalError::Other(err)
    }
}

impl Display for InternalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalError::Io(err) => write!(f, "I/O error: {}", err),
            InternalError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for InternalError {}

#[derive(Debug)]
pub enum HandlerError {
    Protocol(EncapsulationError),
    Internal(InternalError),
}

impl From<EncapsulationError> for HandlerError {
    fn from(err: EncapsulationError) -> Self {
        HandlerError::Protocol(err)
    }
}

impl From<InternalError> for HandlerError {
    fn from(err: InternalError) -> Self {
        HandlerError::Internal(err)
    }
}

impl From<String> for HandlerError {
    fn from(err: String) -> Self {
        HandlerError::Internal(InternalError::Other(err))
    }
}

impl Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::Protocol(err) => write!(f, "Protocol error: {}", err),
            HandlerError::Internal(err) => write!(f, "Internal error: {}", err),
        }
    }
}

impl std::error::Error for HandlerError {}
