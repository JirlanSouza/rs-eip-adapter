use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data::Struct, DataStruct, DeriveInput, ItemStruct, Path, parse_macro_input};

use crate::{cip_attribute::attributes_match, cip_utils};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(cip))]
struct ClassDeriveArgs {
    id: Path,
    name: String,
    #[darling(default)]
    singleton: bool,
    #[darling(default)]
    custom_services: bool,
}

pub fn cip_class_derive_impl(item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemStruct);
    let name = input.ident.clone();
    let struct_ident_span = input.ident.span();

    let args = match ClassDeriveArgs::from_derive_input(&DeriveInput {
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

    let is_singleton = args.singleton;
    let mut errors: Vec<TokenStream2> = Vec::new();

    if is_singleton {
        if let Err(e) = cip_utils::ensure_field(
            struct_ident_span,
            &input.fields,
            "instance",
            None,
            "Singleton CipClass must have an 'instance: RwLock<Arc<dyn CipInstance>>' field",
            None,
        ) {
            errors.push(e.to_compile_error());
        }
    } else {
        if let Err(e) = cip_utils::ensure_field(
            struct_ident_span,
            &input.fields,
            "instances",
            None,
            "Non-singleton CipClass must have an 'instances: RwLock<HashMap<u16, Arc<dyn CipInstance>>>' field",
            None,
        ) {
            errors.push(e.to_compile_error());
        }
    }

    let id_path = args.id;
    let class_name = args.name;

    let attribute_services = attributes_match(&mut input);
    let object_impl = cip_utils::generate_default_cip_object(&name, args.custom_services);

    let instance_impl = if is_singleton {
        quote! {
            fn get_instance(&self, instance_id: u16) -> Result<std::sync::Arc<dyn CipInstance>, CipError> {
                if instance_id != 1 {
                    return Err(CipError::ObjectDoesNotExist);
                }

                let read_guard = self.instance.read().map_err(|_| {
                    log::error!(concat!("Failed to get read guard for ", stringify!(#name), " instance"));
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
                            concat!("Failed to get read guard for ", stringify!(#name), " instance: {}"),
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
                        log::error!(concat!("Failed to get write guard for ", stringify!(#name), " instances"));
                        CipError::GeneralError
                    })?
                    .insert(instance.id(), instance);

                Ok(())
            }
        }
    };

    let expanded = quote! {
        #( #errors )*

        impl #name {
            #attribute_services
        }

        #object_impl

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
