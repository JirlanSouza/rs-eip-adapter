use bytes::{Buf, BufMut, Bytes, BytesMut};
use cip_macros::cip_object_impl;

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipObject, CipResult},
};

#[path = "../../cip/mod.rs"]
mod cip;

struct IdentityInstance {
    id: u16,
    class_id: ClassCode,
}

#[cip_object_impl]
impl IdentityInstance {
    #[service(0x01)]
    fn get_attribute_all(&self, _req: Bytes, resp: &mut BytesMut) -> CipResult {
        resp.reserve(4);
        resp.put_u16_le(self.id);
        resp.put_u16_le(self.class_id.into());
        Ok(())
    }
}

fn main() {
    let instance = IdentityInstance {
        id: 1,
        class_id: ClassCode::Identity,
    };

    let mut resp = BytesMut::new();
    let service_result = instance.execute_service(0x01, Bytes::new(), &mut resp);

    assert!(service_result.is_ok());
    assert_eq!(resp.len(), 4);

    let id = resp.get_u16_le();
    let class_id = resp.get_u16_le();
    assert_eq!(id, 1);
    assert_eq!(class_id, ClassCode::Identity.into());
}
