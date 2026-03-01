use std::sync::Arc;

use cip_macros::{cip_instance, cip_object_impl};

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipInstance, CipObject, CipResult},
};

#[path = "../../cip/mod.rs"]
mod cip;

#[cip_instance]
pub struct IdentityInstance {
    id: u16,
    class_id: ClassCode,
}

#[cip_object_impl]
impl IdentityInstance {}

fn main() {
    let instance = IdentityInstance {
        id: 1,
        class_id: ClassCode::Identity,
    };
    assert_eq!(instance.id(), 1);
    assert_eq!(instance.class_id(), ClassCode::Identity);

    let instace_arc = Arc::new(instance);

    let instance_any = instace_arc.as_any_arc();
    let instance_downcasted_opt = instance_any.downcast_ref::<IdentityInstance>();
    let instance_downcasted = instance_downcasted_opt.expect("Failed to downcast");
    
    assert_eq!(instance_downcasted.id(), 1);
    assert_eq!(instance_downcasted.class_id(), ClassCode::Identity);
}
