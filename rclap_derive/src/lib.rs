use std::{env, path::Path};

use proc_macro2::TokenStream;
use quote::quote;
use rclap_core::*;

#[proc_macro_attribute]
pub fn config(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config_path = if args.is_empty() {
        "config.toml".to_string()
    } else {
        args.to_string().trim_matches('"').to_string()
    };
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR not set - this should be available during compilation");

    let full_path = Path::new(&manifest_dir).join(config_path);
    let input_parsed = syn::parse_macro_input!(input as syn::ItemStruct);
    let struct_name = &input_parsed.ident;

    let config_spec: ConfigSpec = ConfigSpec::from_file(&full_path)
        .unwrap_or_else(|e| panic!("Failed to parse Toml config: {}", e));

    generate_struct(config_spec, struct_name).into()
}

fn generate_struct(
    config_spec: ConfigSpec,
    struct_name: &proc_macro2::Ident,
) -> proc_macro2::TokenStream {
    let mut all_structs = Vec::new();

    let main_struct = generate_single_struct(struct_name, &config_spec.fields, true);
    all_structs.push(main_struct);

    collect_subtypes(&config_spec.fields, &mut all_structs);

    quote! {
        #(#all_structs)*
    }
}

fn generate_single_struct(
    struct_ident: &proc_macro2::Ident,
    fields: &[FieldSpec],
    is_main: bool,
) -> TokenStream {
    let field_definitions: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
            let field_type: TokenStream = field.field_type.parse().expect("Invalid type in config");

            let mut attributes = vec![];

            if let Some(doc) = &field.doc {
                attributes.push(quote! { #[doc = #doc] });
            }

            let mut arg_params = vec![];
            let id = &field.id;
            arg_params.push(quote! { id = #id });
            if let Some(default) = &field.default {
                arg_params.push(quote! { default_value = #default });
            }

            if let Some(env) = &field.env {
                arg_params.push(quote! { env = #env });
            }
            if let Some(l) = &field.long_arg {
                arg_params.push(quote! { long = #l });
            }
            if let Some(s) = &field.short_arg {
                arg_params.push(quote! { short = #s });
            }
            if field.subtype.is_some() {
                attributes.push(quote! { #[command(flatten)] });
            } else {
                attributes.push(quote! { #[arg(#(#arg_params),*)] });
            }

            if let Some(true) = &field.optional {
                quote! {
                    #(#attributes)*
                    pub #field_name: Option<#field_type>,
                }
            } else {
                quote! {
                    #(#attributes)*
                    pub #field_name: #field_type,
                }
            }
        })
        .collect();

    let derives = quote! { #[derive(Debug, Clone, PartialEq, Default, Parser)] };

    let impl_block = if is_main {
        quote! {
            impl #struct_ident {
                pub fn new() -> Self {
                    Default::default()
                }
            }
        }
    } else {
        quote! {}
    };

    quote! {
        #derives
        pub struct #struct_ident {
            #(#field_definitions)*
        }

        #impl_block
    }
}

fn collect_subtypes(fields: &[FieldSpec], structs: &mut Vec<TokenStream>) {
    for field in fields {
        if let Some(subtype_spec) = &field.subtype {
            let struct_name = &field.field_type;

            let struct_ident = syn::Ident::new(struct_name, proc_macro2::Span::call_site());
            let subtype_struct = generate_single_struct(&struct_ident, &subtype_spec.fields, false);
            structs.push(subtype_struct);

            collect_subtypes(&subtype_spec.fields, structs);
        }
    }
}
