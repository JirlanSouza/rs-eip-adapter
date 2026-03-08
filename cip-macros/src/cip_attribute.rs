use std::collections::HashSet;

use darling::{FromField, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, ItemStruct, spanned::Spanned};

#[derive(Debug, FromMeta, PartialEq, Clone)]
#[darling(rename_all = "lowercase")]
enum AttributeAccess {
    Get,
    Set,
}

#[derive(Debug, FromField)]
#[darling(attributes(attribute))]
struct CipAttributeArgs {
    pub id: Option<u16>,
    #[darling(default)]
    pub access: Option<AttributeAccess>,
}

struct ParsedAttribute {
    id: u16,
    access: Option<AttributeAccess>,
    ident: proc_macro2::Ident,
}

pub fn attributes_match(item: &mut ItemStruct) -> TokenStream {
    let mut parsed_attrs = Vec::new();
    let mut darling_errors = Vec::new();
    let mut syn_errors = Vec::new();

    let mut ids = HashSet::<u16>::new();

    if let Fields::Named(ref mut fields) = item.fields {
        for field in &mut fields.named {
            let has_attribute = field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("attribute"));

            if !has_attribute {
                continue;
            }

            match CipAttributeArgs::from_field(field) {
                Ok(args) => {
                    let field_ident = field.ident.clone().expect("Field must have an identifier");
                    let attr_id = match args.id {
                        Some(id) => id,
                        None => {
                            syn_errors.push(syn::Error::new(
                                field.span(),
                                "The #[attribute] attribute expects an ID of u16 (0-65535) type. Ex: #[attribute(id = 0x0E)]",
                            ));
                            continue;
                        }
                    };

                    if !ids.insert(attr_id) {
                        syn_errors.push(syn::Error::new(
                            field.span(),
                            format!("Duplicate attribute ID: {}", attr_id),
                        ));
                        continue;
                    }

                    parsed_attrs.push(ParsedAttribute {
                        id: attr_id,
                        access: args.access,
                        ident: field_ident,
                    });
                }
                Err(e) => {
                    darling_errors.push(e.write_errors());
                }
            }

            field
                .attrs
                .retain(|attr| !attr.path().is_ident("attribute"));
        }
    }

    let mut get_arms = Vec::new();
    let mut set_arms = Vec::new();

    for attr in parsed_attrs {
        let attr_id = attr.id;
        let field_ident = attr.ident;

        // Currently, all attributes with ID support GET
        get_arms.push(quote! {
            #attr_id => {
                self.#field_ident.encode(resp)?;
                Ok(())
            }
        });

        // Add SET support if specifically requested
        if attr.access == Some(AttributeAccess::Set) {
            set_arms.push(quote! {
                #attr_id => {
                    self.#field_ident = FromBytes::decode(req)?;
                    Ok(())
                }
            });
        }
    }

    let syn_compile_errors = syn_errors.iter().map(|e| e.to_compile_error());

    let expanded = quote! {

        #( #darling_errors )*
        #( #syn_compile_errors )*

        pub fn execute_attribute_service(&mut self,
            service_id: u8,
            req: &mut bytes::Bytes,
            resp: &mut bytes::BytesMut
        ) -> CipResult {
            match service_id {
                0x0E => {
                    let attr_id = req.get_u16_le();
                    match attr_id {
                        #( #get_arms )*
                        _ => Err(CipError::AttributeNotSupported),
                    }
                }
                0x10 => {
                    let attr_id = req.get_u16_le();
                    match attr_id {
                        #( #set_arms )*
                        _ => Err(CipError::AttributeNotSupported),
                    }
                }
                _ => Err(CipError::ServiceNotSupported),
            }
        }
    };

    TokenStream::from(expanded)
}
