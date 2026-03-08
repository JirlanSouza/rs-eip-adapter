#![allow(unused_imports)]
use bytes::Buf;
use cip_macros::CipInstance;

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipInstance, CipObject, CipResult},
};

#[path = "../cip/mod.rs"]
mod cip;

#[derive(CipInstance)]
struct MyInstance {
    id: u16,
    class_id: ClassCode,

    #[attribute(id = "invalid_id")]
    first_attr: u32,
}

fn main() {}
