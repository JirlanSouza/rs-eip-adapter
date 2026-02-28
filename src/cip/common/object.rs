use std::{
    any::Any,
    sync::{Arc, Weak},
};

use bytes::{Bytes, BytesMut};

use super::error::CipError;

pub type CipResult = Result<(), CipError>;

pub trait CipObject: Send + Sync {
    fn execute_service(&self, service_id: u8, req: Bytes, resp: &mut BytesMut) -> CipResult;
}

pub trait CipClass: CipObject {
    fn class_id(&self) -> u16;
    fn class_name(&self) -> &'static str;
    fn instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError>;
    fn add_instance(&self, instance: Arc<dyn CipInstance>) -> Result<(), CipError>;
}

pub trait CipInstance: CipObject {
    fn instance_id(&self) -> u16;
    fn class(&self) -> Weak<dyn CipClass>;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}
