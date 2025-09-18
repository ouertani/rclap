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
    parent_id: Option<String>,
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
    let id = match parent_id {
        None => name.clone(),
        Some(pname) => format!("{pname}.{name}").to_string(),
    };
    let reserved_keys = ["type", "default", "doc", "env", "optional", "long", "short"];

    let mut subtype_fields = Vec::new();
    for (sub_name, sub_value) in table {
        if !reserved_keys.contains(&sub_name.as_str())
            && let toml::Value::Table(sub_table) = sub_value
            && let Some(sub_field) =
                table_to_field_spec(sub_name.clone(), sub_table, Some(id.clone()))
        {
            subtype_fields.push(sub_field);
        }
    }
    let field_type = get_field_type(table, !subtype_fields.is_empty(), name.clone());

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
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;
    impl ConfigSpec {
        fn get_field(&self, name: &str) -> Option<&FieldSpec> {
            self.fields.iter().find(|f| f.name == name)
        }
    }
    impl SubtypeSpec {
        pub fn get_field(&self, name: &str) -> Option<&FieldSpec> {
            self.fields.iter().find(|f| f.name == name)
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
port = { type = "u16", default = "8080", doc = "Server port", env = "PORT" }
name = { type = "String", default = "test", long = "name", short = "n" }
"#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 2);

        let port_field = config_spec.get_field("port").unwrap();
        assert_eq!(port_field.name, "port");
        assert_eq!(port_field.field_type, "u16");
        assert_eq!(port_field.default, Some("8080".to_string()));
        assert_eq!(port_field.doc, Some("Server port".to_string()));
        assert_eq!(port_field.env, Some("PORT".to_string()));
        assert_eq!(port_field.id, "port");
        assert!(port_field.subtype.is_none());

        let name_field = config_spec.get_field("name").unwrap();
        assert_eq!(name_field.name, "name");
        assert_eq!(name_field.field_type, "String");
        assert_eq!(name_field.long_arg, Some("name".to_string()));
        assert_eq!(name_field.short_arg, Some('n'));
    }

    #[test]
    fn test_nested_field_parsing() {
        let toml_content = r#"
            [database]
            type = "DatabaseConfig"
            doc = "Database configuration"

            [database.host]
            type = "String"
            default = "localhost"
            env = "DB_HOST"
            
            [database.port]
            type = "u16"
            default = "5432"
            env = "DB_PORT"
            "#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 1);

        let db_field = &config_spec.fields[0];
        assert_eq!(db_field.name, "database");
        assert_eq!(db_field.field_type, "DatabaseConfig");
        assert_eq!(db_field.id, "database");

        let subtype = db_field.subtype.as_ref().unwrap();
        assert_eq!(subtype.fields.len(), 2);

        let host_field = &subtype.fields[0];
        assert_eq!(host_field.name, "host");
        assert_eq!(host_field.field_type, "String");
        assert_eq!(host_field.default, Some("localhost".to_string()));
        assert_eq!(host_field.env, Some("DB_HOST".to_string()));
        assert_eq!(host_field.id, "database.host");

        let port_field = &subtype.fields[1];
        assert_eq!(port_field.name, "port");
        assert_eq!(port_field.field_type, "u16");
        assert_eq!(port_field.default, Some("5432".to_string()));
        assert_eq!(port_field.id, "database.port");
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
            port = { type = "u16", default = "8080" }
            host = { type = "String", default = "localhost" }
            
            [app.server.tls]
            type = "TlsConfig"
            cert = { type = "String", env = "TLS_CERT" }
        "#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 1);

        let app_field = config_spec.get_field("app").unwrap();
        assert_eq!(app_field.name, "app");
        assert_eq!(app_field.field_type, "AppConfig");

        let sub_fields = app_field.subtype.as_ref().unwrap();
        let server_field = sub_fields.get_field("server").unwrap();
        assert_eq!(server_field.name, "server");
        assert_eq!(server_field.field_type, "ServerConfig");
        assert_eq!(server_field.id, "app.server");

        let server_subtype = server_field.subtype.as_ref().unwrap();
        assert_eq!(server_subtype.fields.len(), 2);

        let http_field = &server_subtype.fields[0];
        assert_eq!(http_field.name, "http");
        assert_eq!(http_field.field_type, "HttpConfig");
        assert_eq!(http_field.id, "app.server.http");

        let http_subtype = http_field.subtype.as_ref().unwrap();
        assert_eq!(http_subtype.fields.len(), 2);

        let port_field = &http_subtype.get_field("port").unwrap();
        assert_eq!(port_field.name, "port");
        assert_eq!(port_field.id, "app.server.http.port");
    }

    #[test]
    fn test_optional_fields() {
        let toml_content = r#"
        port = { type = "u16", default = "8080", optional = true }
        host = { type = "String", env = "HOST" }
        debug = { type = "bool", optional = false }
        "#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 3);

        let port_field = config_spec.get_field("port").unwrap();
        assert_eq!(port_field.optional, Some(true));

        let host_field = config_spec.get_field("host").unwrap();
        assert_eq!(host_field.optional, None); // Not specified

        let debug_field = config_spec.get_field("debug").unwrap();
        assert_eq!(debug_field.optional, Some(false));
    }

    #[test]
    fn test_short_arg_validation() {
        let toml_content = r#"
port = { type = "u16", short = "p" }
host = { type = "String", short = "h" }
invalid_short = { type = "String", short = "invalid" }
empty_short = { type = "String", short = "" }
"#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 4);

        let port_field = config_spec.get_field("port").unwrap();
        assert_eq!(port_field.short_arg, Some('p'));

        let host_field = config_spec.get_field("host").unwrap();
        assert_eq!(host_field.short_arg, Some('h'));

        let invalid_field = config_spec.get_field("invalid_short").unwrap();
        assert_eq!(invalid_field.short_arg, None); // Invalid multi-char

        let empty_field = config_spec.get_field("empty_short").unwrap();
        assert_eq!(empty_field.short_arg, None); // Empty string
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
            get_field_type(&table, false, "test".to_string()),
            "CustomType"
        );

        // Test auto-generated type with subfields
        table.clear();
        assert_eq!(
            get_field_type(&table, true, "redis_config".to_string()),
            "Redis_configConfig"
        );

        // Test default string type
        assert_eq!(
            get_field_type(&table, false, "simple".to_string()),
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
                                            app_name = { type = "String", default = "MyApp", env = "APP_NAME" }
                                            debug = { type = "bool", default = "false", short = "d" }

                                            # Nested configuration
                                            [database]
                                            type = "DatabaseConfig"
                                            doc = "Database settings"

                                            [database.primary]
                                            type = "ConnectionConfig"
                                            url = { type = "String", env = "DB_PRIMARY_URL", doc = "Primary DB URL" }
                                            pool_size = { type = "u32", default = "10" }
                                            
                                            [database.replica]
                                            type = "ConnectionConfig" 
                                            url = { type = "String", env = "DB_REPLICA_URL", optional = true }
                                            pool_size = { type = "u32", default = "5" }

                                            # Another top-level section  
                                            [logging]
                                            doc = "Logging configuration"
                                            level = { type = "String", default = "info", env = "LOG_LEVEL", short = "l" }
                                            file = { type = "String", env = "LOG_FILE", optional = true }
                                            "#;
        let config_spec = ConfigSpec::load_toml_config(toml_content);

        assert_eq!(config_spec.fields.len(), 4);

        // Check simple fields
        let app_name = config_spec.get_field("app_name").unwrap();
        assert_eq!(app_name.name, "app_name");
        assert_eq!(app_name.field_type, "String");
        assert_eq!(app_name.env, Some("APP_NAME".to_string()));

        // Check nested database config
        let database = config_spec.get_field("database").unwrap();
        assert_eq!(database.name, "database");
        assert_eq!(database.field_type, "DatabaseConfig");

        let db_subtype = database.subtype.as_ref().unwrap();
        assert_eq!(db_subtype.fields.len(), 2);

        let primary = db_subtype.get_field("primary").unwrap();
        assert_eq!(primary.name, "primary");
        assert_eq!(primary.id, "database.primary");

        let primary_subtype = primary.subtype.as_ref().unwrap();
        let url_field = primary_subtype.get_field("url").unwrap();
        assert_eq!(url_field.id, "database.primary.url");

        // Check auto-generated type for logging
        let logging = config_spec.get_field("logging").unwrap();
        assert_eq!(logging.name, "logging");
        assert_eq!(logging.field_type, "LoggingConfig");

        let logging_subtype = logging.subtype.as_ref().unwrap();
        let level_field = logging_subtype.get_field("level").unwrap();
        assert_eq!(level_field.name, "level");
        assert_eq!(level_field.short_arg, Some('l'));
    }

    #[test]
    fn test_generic_config_spec_conversion() {
        let toml_content = r#"
port = { type = "u16", default = "8080" }
[database]
host = { type = "String", default = "localhost" }
"#;

        let generic: GenericConfigSpec = toml::from_str(toml_content).unwrap();
        assert_eq!(generic.fields.len(), 2);
        assert!(generic.fields.contains_key("port"));
        assert!(generic.fields.contains_key("database"));

        let config_spec: ConfigSpec = generic.into();
        assert_eq!(config_spec.fields.len(), 2);
    }
}
