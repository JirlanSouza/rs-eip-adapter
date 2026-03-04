use bytes::{Buf, BufMut};

use crate::common::binary::{BinaryError, FromBytes, ToBytes};

mod ascii;
pub mod epath;
pub mod short_string;
pub mod string;
#[macro_use]
mod primitive;

pub use short_string::ShortString;
pub use string::CipString;

impl_cip_primitive!(Byte, u8);
impl_cip_primitive!(Word, u16);
impl_cip_primitive!(DWord, u32);
impl_cip_primitive!(USInt, u8);
impl_cip_primitive!(UInt, u16);
impl_cip_primitive!(UDInt, u32);
impl_cip_primitive!(ULInt, u64);
