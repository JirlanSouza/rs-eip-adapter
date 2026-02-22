use bytes::{Buf, BufMut};
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum BinaryError {
    #[error("Buffer too small (expected: {expected}, actual: {actual})")]
    BufferTooSmall { expected: usize, actual: usize },

    #[error("Data truncated (expected: {expected}, actual: {actual})")]
    Truncated { expected: usize, actual: usize },

    #[error("Invalid data: {message} (expected: {expected}, actual: {actual})")]
    InvalidData {
        message: String,
        expected: String,
        actual: String,
    },
}

pub trait FromBytes: Sized {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError>;
}

pub trait ToBytes {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError>;

    fn encoded_len(&self) -> usize;
}
