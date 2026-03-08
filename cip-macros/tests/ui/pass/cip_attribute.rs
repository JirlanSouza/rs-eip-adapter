#![allow(unused_imports)]
use bytes::{Buf, BufMut, Bytes, BytesMut};
use cip_macros::CipInstance;

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipInstance, CipObject, CipResult},
};

#[path = "../../cip/mod.rs"]
mod cip;

// Dummy ToBytes / FromBytes for testing
pub trait ToBytes {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), CipError>;
}

pub trait FromBytes: Sized {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, CipError>;
}

impl ToBytes for u32 {
    fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), CipError> {
        buffer.put_u32_le(*self);
        Ok(())
    }
}

impl FromBytes for u32 {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, CipError> {
        if buffer.remaining() < 4 {
            return Err(CipError::GeneralError);
        }
        Ok(buffer.get_u32_le())
    }
}

#[derive(CipInstance)]
pub struct MyAttributeInstance {
    id: u16,
    class_id: ClassCode,

    #[attribute(id = 0x01, access = "get")]
    get_attr: u32,

    #[attribute(id = 0x02, access = "set")]
    set_attr: u32,
}

fn main() {
    let mut instance = MyAttributeInstance {
        id: 1,
        class_id: ClassCode::Identity,
        get_attr: 42,
        set_attr: 100,
    };

    // Test GetAttributeSingle (0x0E) on get_attr (0x01)
    let mut get_resp = BytesMut::new();
    let mut get_req = BytesMut::new();
    get_req.put_u16_le(0x01); // attr id
    let mut get_req_bytes = get_req.freeze();

    assert!(
        instance
            .execute_service(0x0E, &mut get_req_bytes, &mut get_resp)
            .is_ok()
    );
    assert_eq!(get_resp.get_u32_le(), 42);

    // Test GetAttributeSingle (0x0E) on set_attr (0x02) - should work because Get is default generated for all unless we skipped it
    // Actually, the macro currently generates GET arms for ALL attributes that have an ID.
    let mut get_resp_2 = BytesMut::new();
    let mut get_req_2 = BytesMut::new();
    get_req_2.put_u16_le(0x02); // attr id
    let mut get_req_bytes_2 = get_req_2.freeze();

    assert!(
        instance
            .execute_service(0x0E, &mut get_req_bytes_2, &mut get_resp_2)
            .is_ok()
    );
    assert_eq!(get_resp_2.get_u32_le(), 100);

    // Test SetAttributeSingle (0x10) on set_attr (0x02)
    let mut set_req = BytesMut::new();
    set_req.put_u16_le(0x02); // attr id
    set_req.put_u32_le(999); // new value
    let mut set_req_bytes = set_req.freeze();
    let mut set_resp = BytesMut::new();

    assert!(
        instance
            .execute_service(0x10, &mut set_req_bytes, &mut set_resp)
            .is_ok()
    );
    assert_eq!(instance.set_attr, 999);

    // Test SetAttributeSingle (0x10) on get_attr (0x01) - should fail
    let mut set_req_2 = BytesMut::new();
    set_req_2.put_u16_le(0x01); // attr id
    set_req_2.put_u32_le(555); // new value
    let mut set_req_2_bytes = set_req_2.freeze();
    let mut set_resp_2 = BytesMut::new();

    let res = instance.execute_service(0x10, &mut set_req_2_bytes, &mut set_resp_2);
    assert!(matches!(res, Err(CipError::AttributeNotSupported)));
}
