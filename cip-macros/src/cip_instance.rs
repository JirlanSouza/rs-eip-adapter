use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data::Struct, DataStruct, DeriveInput, ItemStruct, parse_macro_input};

use crate::{cip_attribute::attributes_match, cip_utils};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(cip))]
struct InstanceDeriveArgs {
    #[darling(default)]
    custom_services: bool,
}

pub fn cip_instance_derive_impl(item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let struct_name = input.ident.clone();
    let struct_ident_span = input.ident.span();

    let args = match InstanceDeriveArgs::from_derive_input(&DeriveInput {
        attrs: input.attrs.clone(),
        vis: input.vis.clone(),
        ident: input.ident.clone(),
        generics: input.generics.clone(),
        data: Struct(DataStruct {
            struct_token: input.struct_token,
            fields: input.fields.clone(),
            semi_token: input.semi_token,
        }),
    }) {
        Ok(v) => v,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let mut compile_errors: Vec<TokenStream2> = Vec::new();

    if let Err(e) = cip_utils::ensure_field(
        struct_ident_span,
        &input.fields,
        "id",
        Some("u16"),
        "A struct must have a field 'id: u16' to use #[derive(CipInstance)]",
        Some("The field 'id' must be of type 'u16'"),
    ) {
        compile_errors.push(e.to_compile_error());
    }

    if let Err(e) = cip_utils::ensure_field(
        struct_ident_span,
        &input.fields,
        "class_id",
        Some("ClassCode"),
        "A struct must have a field 'class_id: ClassCode' to use #[derive(CipInstance)]",
        Some("The field 'class_id' must be of type 'ClassCode'"),
    ) {
        compile_errors.push(e.to_compile_error());
    }

    let attribute_services = attributes_match(&mut input);
    let object_impl = cip_utils::generate_default_cip_object(&struct_name, args.custom_services);

    let expanded = quote! {
        #( #compile_errors )*

        impl #struct_name {
            #attribute_services
        }

        #object_impl

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
