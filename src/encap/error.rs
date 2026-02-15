use std::{fmt::Display, io};

use crate::encap::header::{ENCAPSULATION_HEADER_SIZE, EncapsulationHeader};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum EncapsulationError {
    Success = 0x0000,
    InvalidOrUnsupportedCommand = 0x0001,
    InsufficientMemory = 0x0002,
    IncorrectData = 0x0003,
    InvalidSessionHandle = 0x0064,
    InvalidLength = 0x0065,
    UnsupportedProtocol = 0x0069,
}

impl EncapsulationError {
    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

impl Display for EncapsulationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncapsulationError::Success => write!(f, "Success"),
            EncapsulationError::InvalidOrUnsupportedCommand => {
                write!(f, "Invalid or unsupported command")
            }
            EncapsulationError::InsufficientMemory => write!(f, "Insufficient memory"),
            EncapsulationError::IncorrectData => write!(f, "Incorrect data"),
            EncapsulationError::InvalidSessionHandle => write!(f, "Invalid session handle"),
            EncapsulationError::InvalidLength => write!(f, "Invalid length"),
            EncapsulationError::UnsupportedProtocol => write!(f, "Unsupported protocol"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FrameError {
    Inconplete(usize),
    InvalidLength(EncapsulationHeader, usize),
}

impl Display for FrameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameError::Inconplete(length) => write!(
                f,
                "Incomplete frame, min expected: {}, got: {}",
                ENCAPSULATION_HEADER_SIZE, length
            ),
            FrameError::InvalidLength(header, payload_length) => write!(
                f,
                "Invalid length header length field: {}, payload length: {}",
                header.length, payload_length
            ),
        }
    }
}

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

impl Display for HandlerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandlerError::Protocol(err) => write!(f, "Protocol error: {}", err),
            HandlerError::Internal(err) => write!(f, "Internal error: {}", err),
        }
    }
}
