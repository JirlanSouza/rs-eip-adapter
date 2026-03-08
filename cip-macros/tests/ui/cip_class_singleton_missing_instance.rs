#![allow(unused_imports)]
use bytes::Buf;
use cip_macros::CipClass;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipClass, CipInstance, CipObject, CipResult},
};

#[path = "../cip/mod.rs"]
mod cip;

#[derive(CipClass)]
#[cip(id = ClassCode::Identity, name = "Singleton Class", singleton = true)]
struct MySingletonClass {
    // Missing `instance` field
    pub other_field: u16,
}

fn main() {}
