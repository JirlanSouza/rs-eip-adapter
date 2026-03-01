use cip_macros::cip_object_impl;

use crate::cip::{
    error::CipError,
    object::{CipObject, CipResult},
};

#[path = "../cip/mod.rs"]
mod cip;

struct MyObject;

#[cip_object_impl]
impl MyObject {
    #[service("not_a_number")]
    fn my_service(&self, _req: bytes::Bytes, _resp: &mut bytes::BytesMut) -> CipResult {
        Ok(())
    }
}

fn main() {}
