use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Fields, ImplItem, ItemImpl, ItemStruct, LitInt, Meta, Path, Token, parse::Parser,
    parse_macro_input, punctuated::Punctuated, spanned::Spanned,
};

/// Define a CIP attribute that will be dispatched automatically in the `handle_get_attribute_single` method.
///
/// ### Example
/// ```rust
/// pub struct MyObject {
///     #[attribute(0x01)]
///     pub attribute: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn attribute(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let mut get_arms = Vec::new();

    if let Fields::Named(ref fields) = input.fields {
        for field in &fields.named {
            if let Some(attr) = field.attrs.iter().find(|a| a.path().is_ident("id")) {
                let id = attr.parse_args::<LitInt>().expect("id must be an integer");
                let field_name = &field.ident;
                get_arms.push(quote! {
                    #id => self.#field_name.encode(resp).map_err(|_| CipError::ResourceUnavailable)?,
                });
            }
        }
    }

    let expanded = quote! {
        #input
        impl #name {
            pub fn handle_get_attribute_single(&self, attr_id: u16, resp: &mut bytes::BytesMut) -> CipResult {
                match attr_id {
                    #( #get_arms )*
                    _ => return Err(CipError::AttributeNotSupported),
                }
                Ok(())
            }
        }
    };
    TokenStream::from(expanded)
}

/// Implement a CIP object and allow use #[service(0x01)] to define a service.
///
/// ### Example
/// ```rust
/// #[cip_object_impl]
/// impl MyObject {
///
///     #[service(0x01)]
///     fn get_attribute_all(&self, _req: bytes::Bytes, resp: &mut bytes::BytesMut) -> CipResult {
///        ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn cip_object_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);
    let struct_name = &input.self_ty;
    let mut service_arms = Vec::new();
    let mut errors = Vec::new();

    for item in &mut input.items {
        if let ImplItem::Fn(method) = item {
            let mut service_id: Option<u8> = None;

            method.attrs.retain(|attr| {
                if attr.path().is_ident("service") {
                    match attr.parse_args::<LitInt>() {
                        Ok(lit) => match lit.base10_parse::<u8>() {
                            Ok(id) => service_id = Some(id),
                            Err(_) => {
                                errors.push(
                                    syn::Error::new(
                                        lit.span(),
                                        "The service ID must be a u8 (0-255)",
                                    )
                                    .to_compile_error(),
                                );
                            }
                        },
                        Err(_) => {
                            errors.push(
                                syn::Error::new(
                                    attr.span(),
                                    "The #[service] attribute expects an ID of u8 (0-255) type. Ex: #[service(0x0E)]",
                                )
                                .to_compile_error(),
                            );
                        }
                    }
                    return false;
                }
                true
            });

            if let Some(id) = service_id {
                let method_name = &method.sig.ident;
                service_arms.push(quote! {
                    #id => self.#method_name(req, resp),
                });
            }
        }
    }

    let expanded = quote! {
        #( #errors )*

        #input

        impl CipObject for #struct_name {
            fn execute_service(
                &self,
                service_id: u8,
                req: bytes::Bytes,
                resp: &mut bytes::BytesMut
            ) -> CipResult {
                match service_id {
                    #( #service_arms )*
                    _ => Err(CipError::ServiceNotSupported),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

#[derive(Debug, FromMeta)]
struct ClassArgs {
    id: Path,
    name: String,
    #[darling(default)]
    singleton: bool,
}

/// Define a CIP class and implement the CipClass trait for it.
///
/// ### Example for singleton class
/// ```rust
/// #[cip_class(id = ClassCode::Identity, name = "Identity", singleton = true)]
/// pub struct IdentityClass {
///     instance: RwLock<Arc<IdentityInstance>>,
/// }
/// ```
///
/// Implementation of CipClass trait for the struct.
/// ```rust
/// impl CipClass for IdentityClass {
///     fn id(&self) -> u16 { ... }
///
///     fn name(&self) -> &str { ... }
///
///     fn get_instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError> {
///         ...
///     }
///
///     fn add_instance(&self, _instance: Arc<dyn CipInstance>) -> Result<(), CipError> {
///         ...
///     }
/// }
/// ```
///
/// ### Example for non-singleton class
/// ```rust
/// #[cip_class(id = ClassCode::TcpIpInterface, name = "TCP/IP Interface", singleton = false)]
/// pub struct TcpIpInterfaceClass {
///     instances: RwLock<HashMap<u16, Arc<dyn CipInstance>>>,
/// }
/// ```
///
/// Implementation of CipClass trait for the struct.
/// ```rust
/// impl CipClass for TcpIpInterfaceClass {
///     fn id(&self) -> u16 { ... }
///
///     fn name(&self) -> &str { ... }
///
///     fn get_instance(&self, instance_id: u16) -> Result<Arc<dyn CipInstance>, CipError> {
///         ...
///     }
///
///     fn add_instance(&self, _instance: Arc<dyn CipInstance>) -> Result<(), CipError> {
///         ...
///     }
/// }
/// ```
///
#[proc_macro_attribute]
pub fn cip_class(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = &input.ident;

    let attr_parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let meta_list = match attr_parser.parse(attr) {
        Ok(m) => m,
        Err(e) => return e.to_compile_error().into(),
    };

    let args = match ClassArgs::from_list(
        &meta_list
            .into_iter()
            .map(|m| NestedMeta::Meta(m))
            .collect::<Vec<NestedMeta>>(),
    ) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };
    let is_singleton = args.singleton;
    let id_path = args.id;
    let class_name = args.name;
    if let Fields::Named(ref mut fields) = input.fields {
        if is_singleton {
            fields.named.push(
                syn::Field::parse_named
                    .parse2(quote! {
                        instance: std::sync::RwLock<std::sync::Arc<dyn CipInstance>>
                    })
                    .unwrap(),
            );
        } else {
            fields.named.push(syn::Field::parse_named.parse2(quote! {
                instances: std::sync::RwLock<std::collections::HashMap<u16, std::sync::Arc<dyn CipInstance>>>
            }).unwrap());
        }
    }

    let new_impl = if is_singleton {
        quote! {
        impl #name {
            pub fn new(instance: std::sync::Arc<dyn CipInstance>) -> Self {
                let instance = std::sync::RwLock::new(instance);
                Self { instance }
            }
        }
        }
    } else {
        quote! {
            impl #name {
                pub fn new() -> Self {
                    let instances = std::sync::RwLock::new(std::collections::HashMap::new());
                    Self { instances }
                }
            }
        }
    };

    let instance_impl = if is_singleton {
        quote! {
            fn get_instance(&self, instance_id: u16) -> Result<std::sync::Arc<dyn CipInstance>, CipError> {
                if instance_id != 1 {
                    return Err(CipError::ObjectDoesNotExist);
                }

                let read_guard = self.instance.read().map_err(|_| {
                    log::error!("Failed to get read guard for IdentityClass instance");
                    CipError::GeneralError
                })?;

                let inst = std::sync::Arc::clone(&read_guard);
                Ok(inst as std::sync::Arc<dyn CipInstance>)
            }
        }
    } else {
        quote! {
            fn get_instance(&self, instance_id: u16) -> Result<std::sync::Arc<dyn CipInstance>, CipError> {
                self.instances
                    .read()
                    .map_err(|_| {
                        log::error!(
                            "Failed to get read guard for TcpIpInterface instance: {}",
                            instance_id
                        );
                        CipError::GeneralError
                    })?
                    .get(&instance_id)
                    .cloned()
                    .map(|ins| ins as std::sync::Arc<dyn CipInstance>)
                    .ok_or(CipError::ObjectDoesNotExist)
            }
        }
    };

    let add_instance_impl = if is_singleton {
        quote! {
            fn add_instance(&self, _instance: std::sync::Arc<dyn CipInstance>) -> Result<(), CipError> {
                Err(CipError::ResourceUnavailable)
            }
        }
    } else {
        quote! {
            fn add_instance(&self, instance: std::sync::Arc<dyn CipInstance>) -> Result<(), CipError> {
                if instance.class_id() != self.id() {
                    return Err(CipError::InvalidParameter);
                }

                self.instances
                    .write()
                    .map_err(|_| {
                        log::error!("Failed to get write guard for TcpIpInterface instances vector");
                        CipError::GeneralError
                    })?
                    .insert(instance.id(), instance);

                Ok(())
            }
        }
    };

    let expanded = quote! {
        #input

        #new_impl

        impl CipClass for #name {

            fn id(&self) -> crate::cip::ClassCode {
                #id_path
            }

            fn name(&self) -> &'static str { #class_name }

            #instance_impl

            #add_instance_impl
        }
    };

    TokenStream::from(expanded)
}

/// Implement a CIP instance for a struct.
///
/// ### Example
/// ```rust
/// #[cip_instance]
/// struct MyInstance {
///     id: u16,
///     class_id: ClassCode,
///     ... // Attributes
/// }
/// ```
#[proc_macro_attribute]
pub fn cip_instance(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;
    let mut compile_errors = Vec::new();

    let mut has_id = false;
    let mut has_class_id = false;

    if let Fields::Named(ref fields) = input.fields {
        for field in &fields.named {
            let field_ident = field.ident.as_ref().unwrap();
            let field_type = &field.ty;

            if field_ident == "id" {
                has_id = true;
                let type_str = quote!(#field_type).to_string();
                if type_str != "u16" {
                    compile_errors.push(
                        syn::Error::new(field_type.span(), "The field 'id' must be of type 'u16'")
                            .to_compile_error(),
                    );
                }
            }

            if field_ident == "class_id" {
                has_class_id = true;
                let type_str = quote!(#field_type).to_string();
                if !type_str.contains("ClassCode") {
                    compile_errors.push(
                        syn::Error::new(
                            field_type.span(),
                            "The field 'class_id' must be of type 'ClassCode'",
                        )
                        .to_compile_error(),
                    );
                }
            }
        }
    }

    if !has_id {
        compile_errors.push(
            syn::Error::new(
                struct_name.span(),
                "A struct must have a field 'id: u16' to use #[cip_instance]",
            )
            .to_compile_error(),
        );
    }
    if !has_class_id {
        compile_errors.push(
            syn::Error::new(
                struct_name.span(),
                "A struct must have a field 'class_id: ClassCode' to use #[cip_instance]",
            )
            .to_compile_error(),
        );
    }

    let expanded = quote! {
        #( #compile_errors )*

        #input

        impl CipInstance for #struct_name {
            fn id(&self) -> u16 {
                self.id
            }

            fn class_id(&self) -> ClassCode {
                self.class_id
            }

            fn as_any_arc(self: std::sync::Arc<Self>) -> std::sync::Arc<dyn std::any::Any + Send + Sync> {
                self
            }
        }
    };

    TokenStream::from(expanded)
}
