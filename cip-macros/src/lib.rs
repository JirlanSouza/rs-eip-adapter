use proc_macro::TokenStream;

mod cip_attribute;
mod cip_class;
mod cip_instance;
mod cip_object;
mod cip_utils;

/// Implement a CIP object and allow use #[service(0x01)] to define a service.
///
/// ### Example
/// ```rust,ignore
/// #[cip_object_impl]
/// impl MyObject {
///
///     #[service(0x01)]
///     fn get_attribute_all(&mut self, _req: &mut bytes::Bytes, resp: &mut bytes::BytesMut) -> CipResult {
///        ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn cip_object_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    cip_object::cip_object_impl(_attr, item)
}

/// Define a CIP class and implement the CipClass trait for it.
///
/// ### Example for singleton class
/// ```rust,ignore
/// #[derive(CipClass)]
/// #[cip(id = ClassCode::Identity, name = "Identity", singleton = true)]
/// pub struct IdentityClass {
///     instance: RwLock<Arc<dyn CipInstance>>,
/// }
/// ```
///
/// ### Example for non-singleton class
/// ```rust,ignore
/// #[derive(CipClass)]
/// #[cip(id = ClassCode::TcpIpInterface, name = "TCP/IP Interface", singleton = false)]
/// pub struct TcpIpInterfaceClass {
///     instances: RwLock<HashMap<u16, Arc<dyn CipInstance>>>,
/// }
/// ```
#[proc_macro_derive(CipClass, attributes(cip))]
pub fn cip_class_derive(item: TokenStream) -> TokenStream {
    cip_class::cip_class_derive_impl(item)
}

/// Implement a CIP instance.
///
/// ### Example
/// ```rust,ignore
/// #[derive(CipInstance)]
/// #[cip(custom_services = true)]
/// pub struct MyInstance {
///     #[attribute(id = 0x01, access = Get)]
///     my_attribute: u16,
/// }
/// ```
#[proc_macro_derive(CipInstance, attributes(attribute, cip))]
pub fn cip_instance_derive(item: TokenStream) -> TokenStream {
    cip_instance::cip_instance_derive_impl(item)
}
