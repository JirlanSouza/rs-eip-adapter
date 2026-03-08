#![allow(unused_imports)]
use cip_macros::cip_object_impl;

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipClass, CipInstance, CipObject, CipResult},
};

#[path = "../cip/mod.rs"]
mod cip;

struct MyObject;

#[cip_object_impl]
impl MyObject {
    #[service(256)]
    fn my_service(&mut self, _req: &mut bytes::Bytes, _resp: &mut bytes::BytesMut) -> CipResult {
        Ok(())
    }
}

fn main() {}
