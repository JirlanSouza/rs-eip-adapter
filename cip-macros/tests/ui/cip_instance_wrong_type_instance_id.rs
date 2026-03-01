use cip_macros::{cip_instance, cip_object_impl};

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipInstance, CipObject, CipResult},
};

#[path = "../cip/mod.rs"]
mod cip;

#[cip_instance]
struct MyInstance {
    id: u32,
    class_id: ClassCode,
}

#[cip_object_impl]
impl MyInstance {}

fn main() {}
