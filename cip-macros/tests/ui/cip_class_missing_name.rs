use cip_macros::{cip_class, cip_object_impl};

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipClass, CipObject, CipResult},
};

#[path = "../cip/mod.rs"]
mod cip;

#[cip_class(id = ClassCode::Identity)]
struct MyClass {}

#[cip_object_impl]
impl MyClass {}

fn main() {}
