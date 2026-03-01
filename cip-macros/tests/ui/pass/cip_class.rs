use std::sync::Arc;

use cip_macros::{cip_class, cip_instance, cip_object_impl};

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipClass, CipInstance, CipObject, CipResult},
};

#[path = "../../cip/mod.rs"]
mod cip;

#[cip_class(id = ClassCode::TcpIpInterface, name = "TCP/IP Interface", singleton = false)]
pub struct TcpIpInterfaceClass {}

#[cip_object_impl]
impl TcpIpInterfaceClass {}

#[cip_instance]
pub struct TcpIpInstance {
    id: u16,
    class_id: ClassCode,
}

#[cip_object_impl]
impl TcpIpInstance {}

fn main() {
    let instances = vec![
        Arc::new(TcpIpInstance {
            id: 1,
            class_id: ClassCode::TcpIpInterface,
        }),
        Arc::new(TcpIpInstance {
            id: 2,
            class_id: ClassCode::TcpIpInterface,
        }),
    ];

    let tcp_ip_class = TcpIpInterfaceClass::new();

    assert_eq!(tcp_ip_class.id(), ClassCode::TcpIpInterface);
    assert_eq!(tcp_ip_class.name(), "TCP/IP Interface");

    assert!(tcp_ip_class.get_instance(1).is_err());
    assert!(tcp_ip_class.add_instance(instances[0].clone()).is_ok());
    assert!(tcp_ip_class.add_instance(instances[1].clone()).is_ok());

    for instance in instances.iter() {
        let get_instance_result = tcp_ip_class.get_instance(instance.id());
        assert!(get_instance_result.is_ok());

        let geted_instance = get_instance_result.unwrap();
        assert_eq!(geted_instance.id(), instance.id());
        assert_eq!(geted_instance.class_id(), instance.class_id());
    }

    let invalid_instance = Arc::new(TcpIpInstance {
        id: 3,
        class_id: ClassCode::Identity,
    });

    assert!(tcp_ip_class.add_instance(invalid_instance).is_err());
}
