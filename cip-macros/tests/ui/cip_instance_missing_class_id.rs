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
}

fn main() {}
