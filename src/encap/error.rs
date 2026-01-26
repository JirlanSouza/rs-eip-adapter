use std::fmt::Display;

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

impl Into<u32> for EncapsulationError {
    fn into(self) -> u32 {
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
