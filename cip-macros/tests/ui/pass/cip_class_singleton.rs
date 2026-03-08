use std::sync::{Arc, RwLock};

use bytes::Buf;
use cip_macros::{CipClass, CipInstance, cip_object_impl};

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipClass, CipInstance, CipObject, CipResult},
};

#[path = "../../cip/mod.rs"]
mod cip;

#[derive(CipClass)]
#[cip(id = ClassCode::Identity, name = "Identity", singleton = true, custom_services = true)]
pub struct IdentityClass {
    pub instance: RwLock<Arc<dyn CipInstance>>,
}

#[cip_object_impl]
impl IdentityClass {
    pub fn new(instance: Arc<dyn CipInstance>) -> Self {
        Self {
            instance: RwLock::new(instance),
        }
    }
}

#[derive(CipInstance)]
#[cip(custom_services = true)]
pub struct IdentityInstance {
    id: u16,
    class_id: ClassCode,
}

#[cip_object_impl]
impl IdentityInstance {}

fn main() {
    let instances = vec![
        Arc::new(IdentityInstance {
            id: 1,
            class_id: ClassCode::Identity,
        }),
        Arc::new(IdentityInstance {
            id: 2,
            class_id: ClassCode::Identity,
        }),
    ];

    let identity_class = IdentityClass::new(instances[0].clone());

    assert_eq!(identity_class.id(), ClassCode::Identity);
    assert_eq!(identity_class.name(), "Identity");

    assert!(identity_class.add_instance(instances[0].clone()).is_err());
    assert!(identity_class.add_instance(instances[1].clone()).is_err());

    let get_instance_1_result = identity_class.get_instance(instances[0].id());
    assert!(get_instance_1_result.is_ok());

    let geted_instance_1 = get_instance_1_result.unwrap();
    assert_eq!(geted_instance_1.id(), instances[0].id());
    assert_eq!(geted_instance_1.class_id(), instances[0].class_id());

    assert!(identity_class.get_instance(instances[1].id()).is_err());
}
