use std::{collections::HashMap, path::PathBuf};

use serde::Deserialize;
use toml::map::Map;

#[derive(serde::Deserialize)]
pub struct ConfigSpec {
    pub fields: Vec<FieldSpec>,
}
impl ConfigSpec {
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            let spec = Self::load_toml_config(&content);
            Ok(spec)
        } else {
            Err("Unsupported file format. Only .toml and .json are supported.".into())
        }
    }
    fn load_toml_config(toml_content: &str) -> ConfigSpec {
        let generic_config_spec: GenericConfigSpec = toml::from_str(toml_content)
            .unwrap_or_else(|e| panic!("Failed to parse TOML config: {}", e));
        generic_config_spec.into()
    }
}

#[derive(serde::Deserialize)]
pub struct FieldSpec {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub default: Option<String>,
    pub doc: Option<String>,
    pub env: Option<String>,
    pub optional: Option<bool>,
    pub subtype: Option<SubtypeSpec>,
    #[serde(rename = "long")]
    pub long_arg: Option<String>,
    #[serde(rename = "short")]
    pub short_arg: Option<char>,
    pub id: String,
}
#[derive(serde::Deserialize)]
pub struct SubtypeSpec {
    pub fields: Vec<FieldSpec>,
}
#[derive(Debug, Deserialize)]
pub struct GenericConfigSpec {
    #[serde(flatten)]
    pub fields: HashMap<String, toml::Value>,
}
impl From<GenericConfigSpec> for ConfigSpec {
    fn from(generic: GenericConfigSpec) -> Self {
        let mut fields = Vec::new();

        for (field_name, value) in generic.fields {
            match value {
                toml::Value::Table(table) => {
                    if let Some(field_spec) = table_to_field_spec(field_name.clone(), &table, None)
                    {
                        fields.push(field_spec);
                    } else {
                        eprintln!(
                            "Warning: Skipping field '{}' - no type specified",
                            field_name
                        );
                    }
                }
                _ => {
                    eprintln!("Warning: Skipping non-table field '{}'", field_name);
                }
            }
        }

        ConfigSpec { fields }
    }
}
fn table_to_field_spec(
    name: String,
    table: &toml::value::Table,
    parent_name: Option<String>,
) -> Option<FieldSpec> {
    let default = table
        .get("default")
        .and_then(|v| v.as_str())
        .map(String::from);
    let doc = table.get("doc").and_then(|v| v.as_str()).map(String::from);
    let env = table.get("env").and_then(|v| v.as_str()).map(String::from);
    let long_arg = table.get("long").and_then(|v| v.as_str()).map(String::from);
    let short_arg = table
        .get("short")
        .and_then(|v| v.as_str())
        .filter(|s| s.chars().count() == 1)
        .and_then(|s| s.chars().next());
    let optional = table.get("optional").and_then(|v| v.as_bool());

    let reserved_keys = ["type", "default", "doc", "env", "optional", "long", "short"];

    let mut subtype_fields = Vec::new();
    for (sub_name, sub_value) in table {
        if !reserved_keys.contains(&sub_name.as_str())
            && let toml::Value::Table(sub_table) = sub_value
            && let Some(sub_field) =
                table_to_field_spec(sub_name.clone(), sub_table, Some(name.clone()))
        {
            subtype_fields.push(sub_field);
        }
    }
    let field_type = get_field_type(table, !subtype_fields.is_empty(), name.clone());
    let id = match parent_name {
        None => name.clone(),
        Some(pname) => format!("{pname}.{name}").to_string(),
    };
    Some(FieldSpec {
        name,
        field_type: field_type.to_string(),
        default,
        doc,
        env,
        optional,
        long_arg,
        short_arg,
        id,
        subtype: if subtype_fields.is_empty() {
            None
        } else {
            Some(SubtypeSpec {
                fields: subtype_fields,
            })
        },
    })
}
fn get_field_type(table: &Map<String, toml::Value>, has_sub: bool, field_name: String) -> String {
    let field_type = table.get("type").and_then(|v| v.as_str());
    if let Some(ft) = field_type {
        return ft.to_string();
    }
    if has_sub {
        format!("{}Config", to_pascal_case(&field_name))
    } else {
        "String".to_string()
    }
}
fn to_pascal_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
