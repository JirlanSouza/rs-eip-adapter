use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Fields, spanned::Spanned};

/// Checks if a struct has a required field and if its type matches a condition.
/// Returns `Ok(())` if the field is present and valid,
/// or `Err` with a compile error if the field is missing or invalid.
pub fn ensure_field(
    struct_ident_span: proc_macro2::Span,
    fields: &Fields,
    field_name: &str,
    expected_type_contains: Option<&str>,
    missing_error_msg: &str,
    type_error_msg: Option<&str>,
) -> Result<(), Error> {
    let named_fields = match fields {
        Fields::Named(named_fields) => named_fields,
        _ => return Err(Error::new(struct_ident_span, missing_error_msg)),
    };

    for field in &named_fields.named {
        if field.ident.as_ref().map_or(true, |i| i != field_name) {
            continue;
        }

        if let Some(expected) = expected_type_contains {
            let type_str = quote!(#field.ty).to_string().replace(" ", "");
            let expected_no_spaces = expected.replace(" ", "");

            if type_str.contains(&expected_no_spaces) {
                return Ok(());
            }

            return Err(Error::new(
                field.ty.span(),
                type_error_msg.unwrap_or("Invalid field type"),
            ));
        }

        return Ok(());
    }

    Err(Error::new(struct_ident_span, missing_error_msg))
}

/// Generates the default `CipObject` implementation routing to `execute_attribute_service`.
/// Returns an empty `TokenStream` if `custom_services` is true.
pub fn generate_default_cip_object(struct_name: &syn::Ident, custom_services: bool) -> TokenStream {
    if custom_services {
        return quote! {};
    }

    quote! {
        impl CipObject for #struct_name {
            fn execute_service(
                &mut self,
                service_id: u8,
                req: &mut bytes::Bytes,
                resp: &mut bytes::BytesMut
            ) -> CipResult {
                self.execute_attribute_service(service_id, req, resp)
            }
        }
    }
}
