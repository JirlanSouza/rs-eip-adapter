use bytes::{BufMut, BytesMut};

use crate::cip::{
    CipClassId, cip_identity::IdentityInstance, registry::Registry,
    tcp_ip_interface::TcpIpInterfaceInstance,
};
use crate::encap::{
    ENCAPSULATION_PROTOCOL_VERSION,
    cpf::{CpfEncoder, CpfItemId::IdentityItem},
    error::InternalError,
};

pub fn list_identity(registry: &Registry, out_buf: &mut BytesMut) -> Result<(), InternalError> {
    let identity = registry.get_instance::<IdentityInstance>(CipClassId::IdentityClassId, 1)?;
    let tcp_ip_if =
        registry.get_instance::<TcpIpInterfaceInstance>(CipClassId::TcpIpInterfaceClassId, 1)?;

    let mut cpf_encoder = CpfEncoder::new(out_buf);
    let buf = cpf_encoder.add_item_start(IdentityItem);

    buf.put_u16_le(ENCAPSULATION_PROTOCOL_VERSION);
    buf.put_u16(tcp_ip_if.sin_family());
    buf.put_u16(tcp_ip_if.sin_port());
    buf.put_slice(&tcp_ip_if.sin_addr());
    buf.put_slice(&tcp_ip_if.sin_zero());

    buf.put_u16_le(identity.vendor_id);
    buf.put_u16_le(identity.device_type);
    buf.put_u16_le(identity.product_code);
    buf.put_u8(identity.revision_major);
    buf.put_u8(identity.revision_minor);
    buf.put_u16_le(identity.status);
    buf.put_u32_le(identity.serial_number);
    buf.put_u8(identity.product_name.len() as u8);
    buf.put_slice(identity.product_name.as_bytes());
    buf.put_u8(identity.state);
    cpf_encoder.finish();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{cip::{
        cip_class::CipClass,
        cip_identity::{IdentityClass, IdentityInfo},
        tcp_ip_interface::TcpIpInterfaceClass,
    }, encap::cpf::CpfItemId};
    use std::{iter::Product, net::Ipv4Addr};
    use std::sync::Arc;

    #[test]
    fn list_identity_encodes_correct_binary_structure() {
        let mut registry = Registry::new();

        let identity_info = IdentityInfo {
            vendor_id: 0x1234,
            device_type: 0x0002,
            product_code: 0x0007,
            revision_major: 1,
            revision_minor: 5,
            serial_number: 0xDEADBEEF,
            product_name: "Test".to_string(),
        };
        registry.register(IdentityClass::new(&identity_info));

        let tcp_ip_class = Arc::new(TcpIpInterfaceClass::new());
        let tcp_ip_instance = Arc::new(TcpIpInterfaceInstance::new(
            Arc::downgrade(&(tcp_ip_class.clone() as Arc<dyn CipClass>)),
            Ipv4Addr::new(192, 168, 0, 10),
        ));
        tcp_ip_class.add_instance(tcp_ip_instance).unwrap();
        registry.register(tcp_ip_class);

        let mut output_buffer = BytesMut::new();
        list_identity(&registry, &mut output_buffer).expect("list_identity failed");

        let result_bytes = output_buffer.freeze();
        let cpf_item_id = u16::from_le_bytes([result_bytes[2], result_bytes[3]]);
        assert_eq!(cpf_item_id, CpfItemId::IdentityItem.into());
        
        let cpf_item_length = u16::from_le_bytes([result_bytes[4], result_bytes[5]]);
        assert_eq!(cpf_item_length, 38);
        
        let encapsulation_protocol_version = u16::from_le_bytes([result_bytes[6], result_bytes[7]]);
        assert_eq!(encapsulation_protocol_version, 1);
        
        let sin_family = u16::from_be_bytes([result_bytes[8], result_bytes[9]]);
        assert_eq!(sin_family, 2);
        
        let sin_port = u16::from_be_bytes([result_bytes[10], result_bytes[11]]);
        assert_eq!(sin_port, 0xAF12);
        
        let sin_addr = u32::from_be_bytes([result_bytes[12], result_bytes[13], result_bytes[14], result_bytes[15]]);
        assert_eq!(sin_addr, Ipv4Addr::new(192, 168, 0, 10).to_bits());
        
        let sin_zero = &result_bytes[16..24];
        assert_eq!(sin_zero, &[0; 8]);
        

        let vendor_id = u16::from_le_bytes([result_bytes[24], result_bytes[25]]);
        assert_eq!(vendor_id, identity_info.vendor_id);
        
        let device_type = u16::from_le_bytes([result_bytes[26], result_bytes[27]]);
        assert_eq!(device_type, identity_info.device_type);
        
        let product_code = u16::from_le_bytes([result_bytes[28], result_bytes[29]]);
        assert_eq!(product_code, identity_info.product_code);
        
        let revision_major = result_bytes[30];
        assert_eq!(revision_major, identity_info.revision_major);
        
        let revision_minor = result_bytes[31];
        assert_eq!(revision_minor, identity_info.revision_minor);
        
        let status = result_bytes[32];
        assert_eq!(status, 0x00);
        
        let serial_number = u32::from_le_bytes([
            result_bytes[34],
            result_bytes[35],
            result_bytes[36],
            result_bytes[37],
        ]);
        assert_eq!(serial_number, identity_info.serial_number);

        let name_length = result_bytes[38];
        assert_eq!(name_length, identity_info.product_name.len() as u8);
        assert_eq!(&result_bytes[39..43], identity_info.product_name.as_bytes());
        
        let state = result_bytes[43];
        assert_eq!(state, 0x00);
    }

    #[test]
    fn list_identity_fails_when_registry_is_empty() {
        let empty_registry = Registry::new();
        let mut buffer = BytesMut::new();
        let result = list_identity(&empty_registry, &mut buffer);
        assert!(result.is_err());
    }
}
