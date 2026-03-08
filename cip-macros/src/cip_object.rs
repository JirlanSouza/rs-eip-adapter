use proc_macro::TokenStream;
use quote::quote;
use syn::{ImplItem, ItemImpl, LitInt, parse_macro_input, spanned::Spanned};

pub fn cip_object_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);
    let struct_name = &input.self_ty;
    let mut service_arms = Vec::new();
    let mut errors = Vec::new();

    for item in &mut input.items {
        let method = match item {
            ImplItem::Fn(method) => method,
            _ => continue,
        };

        let mut service_id: Option<u8> = None;

        method.attrs.retain(|attr| {
            if !attr.path().is_ident("service") {
                return true;
            }
            
            let lit = match attr.parse_args::<LitInt>() {
                Ok(lit) => lit,
                Err(_) => {
                    errors.push(
                        syn::Error::new(
                            attr.span(),
                            "The #[service] attribute expects an ID of u8 (0-255) type. Ex: #[service(0x0E)]",
                        )
                        .to_compile_error(),
                    );
                    return false;
                }
            };

            match lit.base10_parse::<u8>() {
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
            }

            return false;
        });

        if let Some(id) = service_id {
            let method_name = &method.sig.ident;
            service_arms.push(quote! {
                #id => self.#method_name(req, resp),
            });
        }
    }

    let expanded = quote! {
        #( #errors )*

        #input

        impl CipObject for #struct_name {
            fn execute_service(
                &mut self,
                service_id: u8,
                req: &mut bytes::Bytes,
                resp: &mut bytes::BytesMut
            ) -> CipResult {
                match service_id {
                    #( #service_arms )*
                    _ => self.execute_attribute_service(service_id, req, resp),
                }
            }
        }
    };

    TokenStream::from(expanded)
}
