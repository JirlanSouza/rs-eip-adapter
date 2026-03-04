use crate::cip::data_types::ascii::AsciiString;

pub type CipString<const N: usize> = AsciiString<u16, N>;