#![allow(unused_imports)]
use crate::cip::ClassCode;
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
    id: u32,
    class_id: ClassCode,
}

fn main() {}
