use std::{collections::HashMap, sync::Arc};

use super::ClassCode;
use super::common::object::CipClass;

pub struct Registry {
    classes: HashMap<u16, Arc<dyn CipClass>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
        }
    }

    pub fn register(&mut self, class: Arc<dyn CipClass>) {
        self.classes.insert(class.id().into(), class);
    }

    pub fn get(&self, class_id: u16) -> Option<Arc<dyn CipClass>> {
        log::debug!("Getting class with id {}", class_id);
        self.classes.get(&class_id).cloned()
    }

    pub fn get_instance<T: 'static + Send + Sync>(
        &self,
        class_id: ClassCode,
        instance_id: u16,
    ) -> Result<Arc<T>, String> {
        log::debug!(
            "Getting instance of class {} with id {}",
            class_id,
            instance_id
        );
        let class = self
            .get(class_id.into())
            .ok_or(format!("Class {} not found", class_id))?;
        let instance_ptr = class
            .get_instance(instance_id)
            .map_err(|_| format!("Instance {} for class {} not found", instance_id, class_id))?;

        let any_arc = instance_ptr.as_any_arc();
        any_arc
            .downcast::<T>()
            .map_err(|_| format!("Failed to downcast class {} to requested type", class_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cip::cip_identity::{IdentityClass, IdentityInfo, IdentityInstance};
    use crate::cip::tcp_ip_interface::{TcpIpInterfaceClass, TcpIpInterfaceInstance};
    use std::net::Ipv4Addr;
    use std::sync::Arc;

    #[test]
    fn register_and_get_class_by_id() {
        let mut registry = Registry::new();
        let identity_class_id = ClassCode::Identity;
        let identity_info = IdentityInfo {
            vendor_id: 0x1111,
            device_type: 0x2222,
            product_code: 0x3333,
            revision_major: 1,
            revision_minor: 0,
            serial_number: 0xDEAD_BEEF,
            product_name: "DeviceA",
        };
        let identity_class = IdentityClass::with_default_instance(&identity_info);
        registry.register(identity_class.clone());

        let retrieved_class = registry
            .get(identity_class_id.into())
            .expect("class should be present");

        assert_eq!(retrieved_class.id(), identity_class_id);
        assert_eq!(retrieved_class.name(), "Identity");
    }

    #[test]
    fn register_and_get_identity_instance_success() {
        let mut registry = Registry::new();
        let identity_info = IdentityInfo {
            vendor_id: 0x1234,
            device_type: 0x5678,
            product_code: 0x9ABC,
            revision_major: 1,
            revision_minor: 2,
            serial_number: 0xDEADBEEF,
            product_name: "TestDevice",
        };
        let identity_class = IdentityClass::with_default_instance(&identity_info);
        registry.register(identity_class.clone());

        let identity_instance = registry
            .get_instance::<IdentityInstance>(ClassCode::Identity, 1)
            .expect("expected identity instance");

        assert_eq!(identity_instance.vendor_id, identity_info.vendor_id);
        assert_eq!(
            identity_instance.product_name.value(),
            identity_info.product_name
        );
    }

    #[test]
    fn get_instance_missing_class_returns_error() {
        let registry = Registry::new();

        let error_message = registry
            .get_instance::<IdentityInstance>(ClassCode::Identity, 1)
            .unwrap_err();

        assert!(error_message.contains("not found"));
    }

    #[test]
    fn get_instance_missing_instance_returns_error() {
        let mut registry = Registry::new();
        let identity_info = IdentityInfo {
            vendor_id: 0x0001,
            device_type: 0x0002,
            product_code: 0x0003,
            revision_major: 0,
            revision_minor: 0,
            serial_number: 0,
            product_name: "X",
        };
        let identity_class = IdentityClass::with_default_instance(&identity_info);
        registry.register(identity_class.clone());

        let error_message = registry
            .get_instance::<IdentityInstance>(ClassCode::Identity, 2)
            .unwrap_err();

        assert!(error_message.contains("Instance"));
    }

    #[test]
    fn get_instance_downcast_failure_returns_error() {
        let mut registry = Registry::new();
        let tcp_class = Arc::new(TcpIpInterfaceClass::new());
        let tcp_instance = Arc::new(TcpIpInterfaceInstance::new(1, Ipv4Addr::LOCALHOST));
        tcp_class.add_instance(tcp_instance).unwrap();
        registry.register(tcp_class.clone());

        let error_message = registry
            .get_instance::<IdentityInstance>(ClassCode::TcpIpInterface, 1)
            .unwrap_err();

        assert!(error_message.contains("downcast"));
    }
}
