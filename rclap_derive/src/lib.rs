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
    let struct_def: String = quote! {#struct_name}.to_string().to_lowercase();
    let config_spec: ConfigSpec = ConfigSpec::from_file(&config_attr.full_path(), &struct_def)
        .unwrap_or_else(|e| panic!("Failed to parse Toml config: {}", e));

    generate_struct(config_spec, struct_name, &config_attr).into()
}

fn generate_struct(
    config_spec: ConfigSpec,
    struct_name: &proc_macro2::Ident,
    config_attr: &ConfigAttr,
) -> proc_macro2::TokenStream {
    let mut all_structs = Vec::new();
    let mut all_iter_map_impls = Vec::new();

    let main_struct = generate_single_struct(
        struct_name,
        &config_spec.fields,
        config_attr.extra_derives.clone(),
    );
    all_structs.push(main_struct);

    let main_iter_map = generate_iter_map_impl(struct_name, &config_spec.fields);
    all_iter_map_impls.push(main_iter_map);

    collect_subtypes(
        &config_spec.fields,
        &mut all_structs,
        config_attr.extra_derives.clone(),
        &mut all_iter_map_impls,
    );
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
            use rclap::Secret;
            #(#all_structs)*
            #(#all_iter_map_impls)*

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

fn generate_single_struct(
    struct_ident: &proc_macro2::Ident,
    fields: &[Spec],
    extra_derives: Vec<syn::Path>,
) -> TokenStream {
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
            let field_type = if field.secret {
                quote! { Secret<#field_type> }
            } else {
                field_type
            };

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
    let extra_derives = if extra_derives.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(#(#extra_derives),*)]
        }
    };
    quote! {
        #derives
        #extra_derives
        pub struct #struct_ident {
            #(#field_definitions)*
        }
    }
}

fn collect_subtypes(
    fields: &[Spec],
    items: &mut Vec<TokenStream>,
    extra_derives: Vec<syn::Path>,
    iter_map_impls: &mut Vec<TokenStream>,
) {
    for field in fields {
        match &field.variant {
            GenericSpec::SubtypeSpec(subtype_spec) => {
                let struct_name = &field.field_type;
                let struct_ident = syn::Ident::new(struct_name, proc_macro2::Span::call_site());
                let subtype_struct =
                    generate_single_struct(&struct_ident, subtype_spec, extra_derives.clone());
                items.push(subtype_struct);
                let iter_map = generate_iter_map_impl(&struct_ident, subtype_spec);
                iter_map_impls.push(iter_map);
                collect_subtypes(subtype_spec, items, extra_derives.clone(), iter_map_impls);
            }
            GenericSpec::EnumSpec(enum_spec) if enum_spec.variants.is_empty() => {}
            GenericSpec::EnumSpec(enum_spec) => {
                let enum_name = &field.field_type;

                let enum_ident = syn::Ident::new(enum_name, proc_macro2::Span::call_site());
                let enum_item = generate_enum(&enum_ident, enum_spec, extra_derives.clone());
                items.push(enum_item);
            }
            _ => {}
        }
    }
}
fn generate_enum(
    enum_ident: &proc_macro2::Ident,
    enum_spec: &EnumField,
    extra_derives: Vec<syn::Path>,
) -> TokenStream {
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

    let display_arms: Vec<TokenStream> = enum_spec
        .variants
        .iter()
        .map(|variant_name| {
            let variant_ident = syn::Ident::new(variant_name, proc_macro2::Span::call_site());
            quote! {
                #enum_ident::#variant_ident => write!(f, #variant_name),
            }
        })
        .collect();

    let derives = quote! {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
    };
    let extra_derives = if extra_derives.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(#(#extra_derives),*)]
        }
    };
    //TODO: make rename_all configurable
    let enum_attributes = quote! {
        #[clap(rename_all = "verbatim")]
    };

    quote! {
           #derives
           #extra_derives
           #enum_attributes
           pub enum #enum_ident {
               #(#variants)*
           }
        impl std::fmt::Display for #enum_ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                   match self {
                       #(#display_arms)*
                   }
               }
           }
       }
}
fn generate_iter_map_impl(struct_ident: &proc_macro2::Ident, fields: &[Spec]) -> TokenStream {
    let entries: Vec<TokenStream> = fields
        .iter()
        .map(|field| {
            let field_name = syn::Ident::new(&field.name, proc_macro2::Span::call_site());
            let key = &field.name;

            match &field.variant {
                // Flatten subtypes recursively
                GenericSpec::SubtypeSpec(_) | GenericSpec::ExternalSpec(_) => {
                    quote! {
                        for (k, v) in self.#field_name.iter_map() {
                            map.insert(format!("{}.{}", #key, k), v);
                        }
                    }
                }
                // Vec fields: join with comma
                GenericSpec::VecSpec(_) => {
                    quote! {
                        map.insert(
                            #key.to_string(),
                            self.#field_name
                                .iter()
                                .map(|v| v.to_string())
                                .collect::<Vec<_>>()
                                .join(","),
                        );
                    }
                }
                GenericSpec::EnumSpec(_) => {
                    quote! {
                        map.insert(
                            #key.to_string(),
                            clap::ValueEnum::to_possible_value(&self.#field_name)
                                .expect("no skipped variants")
                                .get_name()
                                .to_string(),
                        );
                    }
                }
                // Optional fields
                _ if field.optional => {
                    quote! {
                        map.insert(
                            #key.to_string(),
                            self.#field_name
                                .as_ref()
                                .map(|v| v.to_string())
                                .unwrap_or_default(),
                        );
                    }
                }
                _ if field.field_type == PATH_BUF => {
                    quote! {
                        map.insert(
                            #key.to_string(),
                            self.#field_name.display().to_string(),
                        );
                    }
                }
                // All other scalar / enum fields
                _ => {
                    quote! {
                        map.insert(#key.to_string(), self.#field_name.to_string());
                    }
                }
            }
        })
        .collect();

    quote! {
        impl #struct_ident {
            pub fn iter_map(&self) -> std::collections::HashMap<String, String> {
                let mut map = std::collections::HashMap::new();
                #(#entries)*
                map
            }
        }
    }
}
