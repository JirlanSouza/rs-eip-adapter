use bytes::{BufMut, BytesMut};

use crate::cip::{
    CipClassId, cip_identity::IdentityInstance, registry::Registry,
    tcp_ip_interface::TcpIpInterfaceInstance,
};
use crate::encap::{
    ENCAPSULATION_PROTOCOL_VERSION,
    cpf::{CpfEncoder, CpfItemId::IdentityItem},
};

pub fn list_identity(registry: &Registry, out_buf: &mut BytesMut) -> Result<(), String> {
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
