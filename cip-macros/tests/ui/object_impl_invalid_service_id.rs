use cip_macros::object_impl;

trait CipObject {
    fn execute_service(
        &self,
        service_id: u8,
        req: bytes::Bytes,
        resp: &mut bytes::BytesMut,
    ) -> Result<(), ()>;
}

struct MyObject;

#[object_impl]
impl MyObject {
    #[service("not_a_number")]
    fn my_service(&self, _req: bytes::Bytes, _resp: &mut bytes::BytesMut) -> Result<(), ()> {
        Ok(())
    }
}

fn main() {}
