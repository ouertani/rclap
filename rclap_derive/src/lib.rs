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

    let main_struct = generate_single_struct(struct_name, &config_spec.fields);
    all_structs.push(main_struct);

    collect_subtypes(&config_spec.fields, &mut all_structs);

    quote! {
        #(#all_structs)*
    }
}

fn generate_single_struct(struct_ident: &proc_macro2::Ident, fields: &[Spec]) -> TokenStream {
    let field_definitions: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
            let field_type: TokenStream = field.field_type.parse().expect("Invalid type in config");

            let mut attributes = vec![];

            let mut arg_params = vec![];
            if let Some(doc) = &field.doc {
                attributes.push(quote! { #[doc = #doc] });
                arg_params.push(quote! { help = #doc });
            }

            let id = &field.id;
            let is_optional = field.optional;
            arg_params.push(quote! { id = #id });
            match &field.variant {
                GenericSpec::FieldSpec(f) => {
                    if let Some(default) = &f.default {
                        if field.field_type == "String"
                            || field.field_type == PATH_BUF
                            || is_optional
                        {
                            arg_params.push(quote! { default_value = #default });
                        } else if field.field_type == "char" {
                            let c = default.chars().next().unwrap();
                            arg_params.push(quote! { default_value_t = #c });
                        } else {
                            let default_lit: TokenStream =
                                default.parse().expect("Invalid default value");
                            arg_params.push(quote! { default_value_t = #default_lit });
                        }
                    }
                    if let Some(env) = &f.env {
                        arg_params.push(quote! { env = #env });
                    }
                    if let Some(l) = &f.long_arg {
                        arg_params.push(quote! { long = #l });
                    } else {
                        arg_params.push(quote! { long = #id })
                    }
                    if let Some(s) = &f.short_arg {
                        arg_params.push(quote! { short = #s });
                    }

                    attributes.push(quote! { #[arg(#(#arg_params),*)] });
                }
                GenericSpec::EnumSpec(e) => {
                    arg_params.push(quote! { value_enum });

                    if let Some(env) = &e.env {
                        arg_params.push(quote! { env = #env });
                    }
                    if let Some(l) = &e.long_arg {
                        arg_params.push(quote! { long = #l });
                    } else {
                        arg_params.push(quote! { long = #id })
                    }
                    if let Some(s) = &e.short_arg {
                        arg_params.push(quote! { short = #s });
                    }

                    attributes.push(quote! { #[arg(#(#arg_params),*)] });
                }
                GenericSpec::SubtypeSpec(_) => {
                    attributes.push(quote! { #[command(flatten)] });
                }
                GenericSpec::ExternalSpec(_) => {
                    attributes.push(quote! { #[command(flatten)] });
                }
            }

            if is_optional {
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

    let derives = quote! { #[derive(Debug, Clone, PartialEq,  Parser)] };

    quote! {
        #derives
        pub struct #struct_ident {
            #(#field_definitions)*
        }
    }
}

fn collect_subtypes(fields: &[Spec], structs: &mut Vec<TokenStream>) {
    for field in fields {
        if let GenericSpec::SubtypeSpec(subtype_spec) = &field.variant {
            let struct_name = &field.field_type;

            let struct_ident = syn::Ident::new(struct_name, proc_macro2::Span::call_site());
            let subtype_struct = generate_single_struct(&struct_ident, subtype_spec);
            structs.push(subtype_struct);

            collect_subtypes(subtype_spec, structs);
        }
    }
}
