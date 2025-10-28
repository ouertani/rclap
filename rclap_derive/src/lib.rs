mod config_attr;
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use rclap_core::*;
use syn::parse_macro_input;

use crate::config_attr::ConfigAttr;
#[proc_macro_attribute]
pub fn config(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let config_attr = parse_macro_input!(args as ConfigAttr);
    let input_parsed = parse_macro_input!(input as syn::ItemStruct);
    let struct_name = &input_parsed.ident;

    let config_spec: ConfigSpec = ConfigSpec::from_file(&config_attr.full_path())
        .unwrap_or_else(|e| panic!("Failed to parse Toml config: {}", e));

    generate_struct(config_spec, struct_name, &config_attr).into()
}

fn generate_struct(
    config_spec: ConfigSpec,
    struct_name: &proc_macro2::Ident,
    config_attr: &ConfigAttr,
) -> proc_macro2::TokenStream {
    let mut all_structs = Vec::new();

    let main_struct = generate_single_struct(struct_name, &config_spec.fields);
    all_structs.push(main_struct);

    collect_subtypes(&config_spec.fields, &mut all_structs);
    let private_mod_name = syn::Ident::new(
        &struct_name.to_string().to_lowercase().to_string(),
        proc_macro2::Span::call_site(),
    );
    let export = if config_attr.export {
        quote! {
           pub use #private_mod_name::*;
        }
    } else {
        quote! {}
    };
    quote! {

      pub mod #private_mod_name {
            use clap::{Parser, ValueEnum};
            #(#all_structs)*

        impl #struct_name {
            pub fn parse() -> Self {
                <Self as Parser>::parse()
            }

            pub fn try_parse() -> Result<Self, clap::Error> {
                <Self as Parser>::try_parse()
            }

            pub fn parse_from<I, T>(itr: I) -> Self
            where
                I: IntoIterator<Item = T>,
                T: Into<std::ffi::OsString> + Clone,
            {
                <Self as Parser>::parse_from(itr)
            }}
        }

       pub use #private_mod_name::#struct_name;
       #export
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
                GenericSpec::VecSpec(f) => {
                    if let Some(default) = &f.default {
                        let default = default.as_array().unwrap();
                        if field.field_type == "Vec<String>" || field.field_type == PATH_BUF {
                            let default_strings: Vec<String> = default
                                .iter()
                                .map(|v| v.as_str().unwrap().to_string())
                                .collect();
                            let default_tokens = default_strings.iter().map(|s| {
                                quote! { #s.to_string() }
                            });
                            arg_params
                                .push(quote! { default_values_t = vec![#(#default_tokens),*] });
                        } else if field.field_type == "Vec<char>" {
                            let default_strings: Vec<char> = default
                                .iter()
                                .map(|v| v.as_str().unwrap().chars().next().unwrap())
                                .collect();
                            let default_tokens = default_strings.iter().map(|s| {
                                quote! { #s }
                            });
                            arg_params
                                .push(quote! { default_values_t = vec![#(#default_tokens),*] });
                        } else if field.field_type == "Vec<i64>" {
                            let defaults: Vec<i64> =
                                default.iter().map(|v| v.as_integer().unwrap()).collect();
                            let default_tokens = defaults.iter().map(|s| {
                                let lit = Literal::i64_unsuffixed(*s);
                                quote! { #lit }
                            });
                            arg_params.push(quote! { default_values_t = [#(#default_tokens),*] });
                        } else if field.field_type == "Vec<f64>" {
                            let defaults: Vec<f64> =
                                default.iter().map(|v| v.as_float().unwrap()).collect();
                            let default_tokens = defaults.iter().map(|s| {
                                let lit = Literal::f64_unsuffixed(*s);
                                quote! { #lit }
                            });
                            arg_params.push(quote! { default_values_t = [#(#default_tokens),*] });
                        } else if field.field_type == "Vec<bool>" {
                            let defaults: Vec<bool> =
                                default.iter().map(|v| v.as_bool().unwrap()).collect();
                            let default_tokens = defaults.iter().map(|s| {
                                quote! { #s }
                            });
                            arg_params.push(quote! { default_values_t = [#(#default_tokens),*] });
                        } else if field.field_type == "Vec<usize>" {
                            let defaults: Vec<usize> = default
                                .iter()
                                .map(|v| v.as_integer().unwrap().try_into().unwrap())
                                .collect();
                            let default_tokens = defaults.iter().map(|s| {
                                quote! { #s }
                            });
                            arg_params.push(quote! { default_values_t = [#(#default_tokens),*] });
                        } else {
                            panic!("Unsupported Vec default type");
                        }
                    }
                    if let Some(env) = &f.env {
                        arg_params.push(quote! { env = #env });
                        arg_params.push(quote! { value_delimiter = ',' });
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
                    if let Some(default) = &e.default {
                        if field.field_type.contains("::") {
                            let enum_name = &field.field_type;
                            let field_type_ident: proc_macro2::TokenStream =
                                enum_name.parse().expect("Invalid enum path");
                            let default_variant: proc_macro2::TokenStream =
                                default.parse().expect("Invalid enum path");

                            arg_params.push(
                                quote! { default_value_t = #field_type_ident::#default_variant },
                            );
                        } else {
                            let field_type_ident: proc_macro2::Ident =
                                syn::parse_str(&field.field_type).expect("Invalid field type");
                            let default_variant =
                                syn::Ident::new(default, proc_macro2::Span::call_site());
                            arg_params.push(
                                quote! { default_value_t = #field_type_ident::#default_variant },
                            );
                        }
                    }
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

fn collect_subtypes(fields: &[Spec], items: &mut Vec<TokenStream>) {
    for field in fields {
        match &field.variant {
            GenericSpec::SubtypeSpec(subtype_spec) => {
                let struct_name = &field.field_type;
                let struct_ident = syn::Ident::new(struct_name, proc_macro2::Span::call_site());
                let subtype_struct = generate_single_struct(&struct_ident, subtype_spec);
                items.push(subtype_struct);
                collect_subtypes(subtype_spec, items);
            }
            GenericSpec::EnumSpec(enum_spec) if enum_spec.variants.is_empty() => {}
            GenericSpec::EnumSpec(enum_spec) => {
                let enum_name = &field.field_type;

                let enum_ident = syn::Ident::new(enum_name, proc_macro2::Span::call_site());
                let enum_item = generate_enum(&enum_ident, enum_spec);
                items.push(enum_item);
            }
            _ => {}
        }
    }
}
fn generate_enum(enum_ident: &proc_macro2::Ident, enum_spec: &EnumField) -> TokenStream {
    let variants: Vec<TokenStream> = enum_spec
        .variants
        .iter()
        .map(|variant_name| {
            let variant_ident = syn::Ident::new(variant_name, proc_macro2::Span::call_site());

            quote! {
                #variant_ident,
            }
        })
        .collect();

    let derives = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
    };
    //TODO: make rename_all configurable
    let enum_attributes = quote! {
        #[clap(rename_all = "verbatim")]
    };

    quote! {
            #derives
    #enum_attributes
            pub enum #enum_ident {
                #(#variants)*
            }
        }
}
