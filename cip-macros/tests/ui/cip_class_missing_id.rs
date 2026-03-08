#![allow(unused_imports)]
use bytes::Buf;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use cip_macros::CipClass;

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipClass, CipInstance, CipObject, CipResult},
};

#[path = "../cip/mod.rs"]
mod cip;

#[derive(CipClass)]
#[cip(name = "MyClass")]
struct MyClass {
    pub instances: RwLock<HashMap<u16, Arc<dyn CipInstance>>>,
}

fn main() {}
