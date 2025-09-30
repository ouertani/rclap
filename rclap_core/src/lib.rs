pub mod ast;
pub use ast::{EnumField, ExternalStruct, Field, GenericSpec, Spec, SubField};
mod utils;
use std::{collections::HashMap, path::PathBuf};

use crate::{ast::VecField, utils::get_field_type};
use serde::Deserialize;

pub const PATH_BUF: &str = "std::path::PathBuf";
#[derive(serde::Deserialize, Debug)]
pub struct ConfigSpec {
    pub fields: Vec<Spec>,
}
impl ConfigSpec {
    pub fn from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            let spec = Self::load_toml_config(&content);
            Ok(spec)
        } else {
            Err("Unsupported file format. Only .toml is supported.".into())
        }
    }
    fn load_toml_config(toml_content: &str) -> ConfigSpec {
        let generic_config_spec: GenericConfigSpec = toml::from_str(toml_content)
            .unwrap_or_else(|e| panic!("Failed to parse TOML config: {}", e));
        generic_config_spec.into()
    }
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
                    let field_spec = table_to_field_spec(field_name.clone(), &table, None);

                    fields.push(field_spec);
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
    toml_tag_name: String,
    table: &toml::value::Table,
    parent_id: Option<String>,
) -> Spec {
    let doc = table.get("doc").and_then(|v| v.as_str()).map(String::from);
    let enum_name = table.get("enum").and_then(|v| v.as_str()).map(String::from);
    let env = table.get("env").and_then(|v| v.as_str()).map(String::from);
    let long_arg = table.get("long").and_then(|v| v.as_str()).map(String::from);
    let short_arg = table
        .get("short")
        .and_then(|v| v.as_str())
        .filter(|s| s.chars().count() == 1)
        .and_then(|s| s.chars().next());
    let name = &toml_tag_name;
    let id = match parent_id {
        None => name.clone(),
        Some(pname) => format!("{pname}.{name}").to_string(),
    };
    let reserved_keys = ["type", "default", "doc", "env", "optional", "long", "short"];

    let mut subtype_fields = Vec::new();
    for (sub_name, sub_value) in table {
        if !reserved_keys.contains(&sub_name.as_str())
            && let toml::Value::Table(sub_table) = sub_value
        {
            let sub_field = table_to_field_spec(sub_name.clone(), sub_table, Some(id.clone()));
            subtype_fields.push(sub_field);
        }
    }
    let field_type = get_field_type(table, !subtype_fields.is_empty(), name.clone());
    let optional = table
        .get("optional")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if field_type.is_vec {
        let default = table.get("default").cloned();
        let variant = GenericSpec::VecSpec(VecField {
            default,
            env,
            long_arg,
            short_arg,
            optional,
        });
        return Spec::new(toml_tag_name, id, field_type.type_name, doc, variant);
    }
    let default = table
        .get("default")
        .and_then(|v| v.as_str())
        .map(String::from);
    let variant = if subtype_fields.is_empty() && field_type.is_native {
        GenericSpec::FieldSpec(Field {
            default,
            env,
            long_arg,
            short_arg,
            optional,
        })
    } else if !subtype_fields.is_empty() {
        GenericSpec::SubtypeSpec(SubField(subtype_fields.clone()))
    } else {
        match enum_name {
            Some(enum_name) => GenericSpec::EnumSpec(EnumField {
                env,
                long_arg,
                short_arg,
                optional,
                enum_name,
            }),
            None => GenericSpec::ExternalSpec(ExternalStruct {
                long_arg,
                short_arg,
            }),
        }
    };

    Spec::new(toml_tag_name, id, field_type.type_name, doc, variant)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::GenericSpec;
    use crate::utils::to_pascal_case;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use toml::map::Map;

    fn get_field<'a>(fields: &'a [Spec], name: &str) -> Option<&'a Spec> {
        fields.iter().find(|f| f.name == name)
    }
    impl ConfigSpec {
        fn get_field(&self, name: &str) -> Option<&Spec> {
            self.fields.iter().find(|f| f.name == name)
        }
    }
    impl Spec {
        fn as_field_spec(&self) -> &Field {
            if let GenericSpec::FieldSpec(f) = &self.variant {
                f
            } else {
                panic!("Not a FieldSpec variant");
            }
        }
        fn as_subtype_spec(&self) -> &SubField {
            if let GenericSpec::SubtypeSpec(s) = &self.variant {
                s
            } else {
                panic!("Not a SubtypeSpec variant");
            }
        }
    }

    // Helper function to create a temporary TOML file
    fn create_temp_toml(content: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_config.toml");
        fs::write(&file_path, content).unwrap();
        (temp_dir, file_path)
    }

    #[test]
    fn test_simple_field_parsing() {
        let toml_content = r#"
port = { type = "int", default = "8080", doc = "Server port", env = "PORT" }
name = { type = "String", default = "test", long = "name", short = "n" }
"#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 2);

        let port_field = config_spec.get_field("port").unwrap();

        assert_eq!(port_field.name, "port");
        assert_eq!(port_field.field_type, "i64");
        assert_eq!(port_field.doc, Some("Server port".to_string()));
        assert_eq!(port_field.id, "port");
        let port = port_field.as_field_spec();

        assert_eq!(port.default, Some("8080".to_string()));
        assert_eq!(port.env, Some("PORT".to_string()));
        assert_eq!(port.long_arg, None);
        assert_eq!(port.short_arg, None);

        let name_field = config_spec.get_field("name").unwrap();

        assert_eq!(name_field.name, "name");
        assert_eq!(name_field.field_type, "String");

        let name = name_field.as_field_spec();

        assert_eq!(name.default, Some("test".to_string()));
        assert_eq!(name.env, None);
        assert_eq!(name.long_arg, Some("name".to_string()));
        assert_eq!(name.short_arg, Some('n'));
    }

    #[test]
    fn test_nested_field_parsing() {
        let toml_content = r#"
        [database]
        type = "DatabaseConfig"
        doc = "Database configuration"

        [database.host]
        default = "localhost"
        env = "DB_HOST"
        
        [database.port]
        type = "int"
        default = "5432"
        env = "DB_PORT"
        "#;

        let config_spec = ConfigSpec::load_toml_config(toml_content);
        assert_eq!(config_spec.fields.len(), 1);

        // Test database field
        let db_field = &config_spec.fields[0];
        assert_eq!(db_field.name, "database");
        assert_eq!(db_field.field_type, "DatabaseConfig");
        assert_eq!(db_field.id, "database");

        // Extract and test database subtype

        let fields = db_field.as_subtype_spec();
        assert_eq!(fields.len(), 2);

        // Test host field
        let host_field = &fields[0];
        assert_eq!(host_field.name, "host");
        assert_eq!(host_field.field_type, "String");
        assert_eq!(host_field.id, "database.host");

        let host = host_field.as_field_spec();

        assert_eq!(host.default, Some("localhost".to_string()));
        assert_eq!(host.env, Some("DB_HOST".to_string()));
        assert_eq!(host.long_arg, None);
        assert_eq!(host.short_arg, None);

        // Test port field
        let port_field = &fields[1];
        assert_eq!(port_field.name, "port");
        assert_eq!(port_field.field_type, "i64");
        assert_eq!(port_field.id, "database.port");

        let port = port_field.as_field_spec();
        assert_eq!(port.default, Some("5432".to_string()));
        assert_eq!(port.env, Some("DB_PORT".to_string()));
        assert_eq!(port.long_arg, None);
        assert_eq!(port.short_arg, None);
    }

    #[test]
    fn test_deep_nested_structure() {
        let toml_content = r#"
        [app]
        type = "AppConfig"

        [app.server]
        type = "ServerConfig"
        
            [app.server.http]
            type = "HttpConfig"
            port = { type = "int", default = "8080" }
            host = {  default = "localhost" }
            
            [app.server.tls]
            type = "TlsConfig"
            cert = {  env = "TLS_CERT" }
        "#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 1);

        let app_field = config_spec.get_field("app").unwrap();
        assert_eq!(app_field.name, "app");
        assert_eq!(app_field.field_type, "AppConfig");

        let fields = app_field.as_subtype_spec();
        let server_field = get_field(fields, "server").unwrap();
        assert_eq!(server_field.name, "server");
        assert_eq!(server_field.field_type, "ServerConfig");
        assert_eq!(server_field.id, "app.server");

        let fields = server_field.as_subtype_spec();
        assert_eq!(fields.len(), 2);

        let http_field = &fields[0];
        assert_eq!(http_field.name, "http");
        assert_eq!(http_field.field_type, "HttpConfig");
        assert_eq!(http_field.id, "app.server.http");

        let fields = http_field.as_subtype_spec();
        assert_eq!(fields.len(), 2);

        let port_field = get_field(fields, "port").unwrap();
        assert_eq!(port_field.name, "port");
        assert_eq!(port_field.id, "app.server.http.port");
    }

    #[test]
    fn test_optional_fields() {
        let toml_content = r#"
        port = { type = "int", default = "8080", optional = true }
        host = { type = "String", env = "HOST" }
        debug = { type = "bool", optional = false }
        "#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 3);

        let port_field = config_spec.get_field("port").unwrap().as_field_spec();
        assert!(port_field.optional);

        let host_field = config_spec.get_field("host").unwrap().as_field_spec();
        assert!(!host_field.optional); // Not specified

        let debug_field = config_spec.get_field("debug").unwrap().as_field_spec();
        assert!(!debug_field.optional);
    }

    #[test]
    fn test_short_arg_validation() {
        let toml_content = r#"
port = { type = "int", short = "p" }
host = { type = "String", short = "h" }
invalid_short = { type = "String", short = "invalid" }
empty_short = { type = "String", short = "" }
"#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 4);

        let port_field = config_spec.get_field("port").unwrap();
        let port = port_field.as_field_spec();
        assert_eq!(port.short_arg, Some('p'));

        let host_field = config_spec.get_field("host").unwrap();
        let host = host_field.as_field_spec();
        assert_eq!(host.short_arg, Some('h'));

        let invalid_field = config_spec.get_field("invalid_short").unwrap();
        let invalid_short = invalid_field.as_field_spec();
        assert_eq!(invalid_short.short_arg, None); // Invalid multi-char

        let empty_field = config_spec.get_field("empty_short").unwrap();
        let empty_short = empty_field.as_field_spec();
        assert_eq!(empty_short.short_arg, None); // Empty string
    }

    #[test]
    fn test_from_file_toml() {
        let toml_content = r#"
port = { type = "u16", default = "8080" }
host = { type = "String", default = "localhost" }
"#;
        let (_temp_dir, file_path) = create_temp_toml(toml_content);

        let config_spec = ConfigSpec::from_file(&file_path).unwrap();
        assert_eq!(config_spec.fields.len(), 2);
    }

    #[test]
    fn test_from_file_unsupported_format() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("config.yaml");
        fs::write(&file_path, "port: 8080").unwrap();

        let result = ConfigSpec::from_file(&file_path);
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.to_string().contains("Unsupported file format")),
            Ok(_) => panic!("Expected an error"),
        }
    }

    #[test]
    fn test_from_file_nonexistent() {
        let file_path = PathBuf::from("nonexistent.toml");
        let result = ConfigSpec::from_file(&file_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello"), "Hello");
        assert_eq!(to_pascal_case("hello_world"), "Hello_world");
        assert_eq!(to_pascal_case(""), "");
        assert_eq!(to_pascal_case("a"), "A");
        assert_eq!(to_pascal_case("API"), "API");
        assert_eq!(to_pascal_case("snake_case_name"), "Snake_case_name");
    }

    #[test]
    fn test_get_field_type() {
        let mut table = Map::new();

        // Test explicit type
        table.insert(
            "type".to_string(),
            toml::Value::String("CustomType".to_string()),
        );
        assert_eq!(
            get_field_type(&table, false, "test".to_string()).type_name,
            "CustomType"
        );

        // Test auto-generated type with subfields
        table.clear();
        assert_eq!(
            get_field_type(&table, true, "redis_config".to_string()).type_name,
            "Redis_configConfig"
        );

        // Test default string type
        assert_eq!(
            get_field_type(&table, false, "simple".to_string()).type_name,
            "String"
        );
    }

    #[test]
    #[should_panic(expected = "Failed to parse TOML config")]
    fn test_invalid_toml_parsing() {
        let invalid_toml = r#"
                                            invalid toml content
                                            port = 
                                            "#;
        ConfigSpec::load_toml_config(invalid_toml);
    }

    #[test]
    fn test_complex_mixed_structure() {
        let toml_content = r#"
                                            # Simple top-level fields
                                            app_name = { type = "string", default = "MyApp", env = "APP_NAME" }
                                            debug = { type = "bool", default = "false", short = "d" }

                                            # Nested configuration
                                            [database]
                                            type = "DatabaseConfig"
                                            doc = "Database settings"

                                            [database.primary]
                                            type = "ConnectionConfig"
                                            url = { type = "string", env = "DB_PRIMARY_URL", doc = "Primary DB URL" }
                                            pool_size = { type = "int", default = "10" }
                                            
                                            [database.replica]
                                            type = "ConnectionConfig" 
                                            url = { type = "string", env = "DB_REPLICA_URL", optional = true }
                                            pool_size = { type = "int", default = "5" }

                                            # Another top-level section  
                                            [logging]
                                            doc = "Logging configuration"
                                            level = { type = "string", default = "info", env = "LOG_LEVEL", short = "l" }
                                            file = { type = "string", env = "LOG_FILE", optional = true }
                                            "#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 4);

        // Check simple fields
        let app_name = config_spec.get_field("app_name").unwrap();
        assert_eq!(app_name.name, "app_name");
        assert_eq!(app_name.field_type, "String");

        let app_name = app_name.as_field_spec();
        assert_eq!(app_name.env, Some("APP_NAME".to_string()));

        // Check nested database config
        let database = config_spec.get_field("database").unwrap();
        assert_eq!(database.name, "database");
        assert_eq!(database.field_type, "DatabaseConfig");

        let fields = database.as_subtype_spec();
        assert_eq!(fields.len(), 2);

        let primary = get_field(fields, "primary").unwrap();
        assert_eq!(primary.name, "primary");
        assert_eq!(primary.id, "database.primary");

        let primary_subtype = primary.as_subtype_spec();
        let url_field = get_field(primary_subtype, "url").unwrap();
        assert_eq!(url_field.id, "database.primary.url");

        let logging = config_spec.get_field("logging").unwrap();
        assert_eq!(logging.name, "logging");
        assert_eq!(logging.field_type, "LoggingConfig");

        let logging_subtype = logging.as_subtype_spec();
        let level_field = get_field(logging_subtype, "level").unwrap();
        assert_eq!(level_field.name, "level");
        let level = level_field.as_field_spec();
        assert_eq!(level.short_arg, Some('l'));
    }
}
