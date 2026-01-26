use std::{
    any::Any,
    sync::{Arc, Weak},
};

use crate::cip::cip_error::CipError;

pub trait CipClass: Send + Sync {
    fn class_id(&self) -> u16;
    fn class_name(&self) -> &'static str;
    fn instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError>;
    fn add_instance(&self, instance: Arc<dyn CipInstance>) -> Result<(), CipError>;
}

pub trait CipInstance: Send + Sync {
    fn class(&self) -> Weak<dyn CipClass>;
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}
