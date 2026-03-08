use bytes::{Buf, BufMut, Bytes, BytesMut};
use cip_macros::{CipInstance, cip_object_impl};

use crate::cip::{
    ClassCode,
    error::CipError,
    object::{CipInstance, CipObject, CipResult},
};

#[path = "../../cip/mod.rs"]
mod cip;

#[derive(CipInstance)]
#[cip(custom_services = true)]
struct IdentityInstance {
    id: u16,
    class_id: ClassCode,
}

#[cip_object_impl]
impl IdentityInstance {
    #[service(0x01)]
    fn get_attribute_all(&mut self, _req: &mut Bytes, resp: &mut BytesMut) -> CipResult {
        resp.reserve(4);
        resp.put_u16_le(self.id);
        resp.put_u16_le(self.class_id.into());
        Ok(())
    }
}

fn main() {
    let mut instance = IdentityInstance {
        id: 1,
        class_id: ClassCode::Identity,
    };

    let mut resp = BytesMut::new();
    let mut req = Bytes::new();
    let service_result = instance.execute_service(0x01, &mut req, &mut resp);

    assert!(service_result.is_ok());
    assert_eq!(resp.len(), 4);

    let id = resp.get_u16_le();
    let class_id = resp.get_u16_le();
    assert_eq!(id, 1);
    assert_eq!(class_id, ClassCode::Identity.into());
}
