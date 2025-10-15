use std::{
    env,
    path::{Path, PathBuf},
};

use syn::{Token, parse::Parse, parse::ParseStream};

#[derive(Debug)]
pub(crate) struct ConfigAttr {
    path: String,
    pub export: bool,
}
impl ConfigAttr {
    pub(crate) fn full_path(&self) -> PathBuf {
        let manifest_dir = env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR not set - this should be available during compilation");

        Path::new(&manifest_dir).join(self.path.clone())
    }
}
impl Default for ConfigAttr {
    fn default() -> Self {
        let path = "config.toml".to_string();
        Self { path, export: true }
    }
}

impl Parse for ConfigAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut config = ConfigAttr::default();

        if input.peek(syn::LitStr) {
            let path_lit: syn::LitStr = input.parse()?;
            config.path = path_lit.value();
            return Ok(config);
        }

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;

            match ident.to_string().as_str() {
                "path" => {
                    let _eq: Token![=] = input.parse()?;
                    let path_lit: syn::LitStr = input.parse()?;
                    config.path = path_lit.value();
                }
                "export" => {
                    let _eq: Token![=] = input.parse()?;
                    let export_lit: syn::LitBool = input.parse()?;
                    config.export = export_lit.value();
                }
                _ => {
                    return Err(syn::Error::new(ident.span(), "unknown parameter"));
                }
            }

            if input.peek(Token![,]) {
                let _comma: Token![,] = input.parse()?;
            }
        }
        Ok(config)
    }
}
