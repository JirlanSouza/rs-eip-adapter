use std::{any::Any, sync::Arc};

use bytes::{Bytes, BytesMut};
use super::{ClassCode, error::CipError};

pub type CipResult = Result<(), CipError>;

pub trait CipObject: Send + Sync {
    fn execute_service(
        &mut self,
        service_id: u8,
        req: &mut Bytes,
        resp: &mut BytesMut,
    ) -> CipResult;
}

pub trait CipClass: CipObject {
    fn id(&self) -> ClassCode;
    fn name(&self) -> &'static str;
    fn get_instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError>;
    fn add_instance(&self, instance: Arc<dyn CipInstance>) -> Result<(), CipError>;
}

pub trait CipInstance: CipObject {
    fn id(&self) -> u16;
    fn class_id(&self) -> ClassCode;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}
