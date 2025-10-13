use std::ops::Deref;

use toml::Value;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Spec {
    pub toml_tag_name: String,
    pub id: String,
    pub field_type: String,
    pub doc: Option<String>,
    pub variant: GenericSpec,
    pub name: String,
    pub optional: bool,
}
#[derive(serde::Deserialize, Clone, Debug)]
pub enum GenericSpec {
    FieldSpec(Field),
    SubtypeSpec(SubField),
    ExternalSpec(ExternalStruct),
    EnumSpec(EnumField),
    VecSpec(VecField),
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Field {
    pub default: Option<String>,
    pub env: Option<String>,
    pub long_arg: Option<String>,
    pub short_arg: Option<char>,
    pub optional: bool,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct SubField(pub Vec<Spec>);
impl Deref for SubField {
    type Target = Vec<Spec>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(serde::Deserialize, Clone, Debug)]
pub struct ExternalStruct {
    pub long_arg: Option<String>,
    pub short_arg: Option<char>,
}
#[derive(serde::Deserialize, Clone, Debug)]
pub struct EnumField {
    pub env: Option<String>,
    pub long_arg: Option<String>,
    pub short_arg: Option<char>,
    pub optional: bool,
    pub enum_name: String,
    pub variants: Vec<String>,
    pub default: Option<String>,
}
#[derive(serde::Deserialize, Clone, Debug)]
pub struct VecField {
    pub default: Option<Value>,
    pub env: Option<String>,
    pub long_arg: Option<String>,
    pub short_arg: Option<char>,
    pub optional: bool,
}
impl Spec {
    pub fn new(
        toml_tag_name: String,
        id: String,
        field_type: String,
        doc: Option<String>,
        variant: GenericSpec,
    ) -> Self {
        let name = toml_tag_name.clone();
        let optional = match &variant {
            GenericSpec::FieldSpec(f) => f.optional,
            GenericSpec::SubtypeSpec { .. } => false,
            GenericSpec::ExternalSpec(_) => false,
            GenericSpec::EnumSpec(f) => f.optional,
            GenericSpec::VecSpec(f) => f.optional,
        };
        Spec {
            toml_tag_name,
            id,
            field_type,
            doc,
            variant,
            name,
            optional,
        }
    }
}
