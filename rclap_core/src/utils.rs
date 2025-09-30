use crate::PATH_BUF;

pub const NATIVE_TYPES: [&str; 6] = ["int", "float", "bool", "string", "path", "char"];
fn is_native_type(ty: &str) -> bool {
    NATIVE_TYPES.contains(&ty.to_lowercase().as_str())
        || ty == "i64"
        || ty == PATH_BUF
        || ty == "f64"
        || ty == "String"
}
fn to_type(ty: &str) -> String {
    let ty_lower = ty.to_lowercase();
    match ty_lower.as_str() {
        "int" => "i64".to_string(),
        "float" => "f64".to_string(),
        "bool" => "bool".to_string(),
        "string" => "String".to_string(),
        "path" => PATH_BUF.to_string(),
        "char" => "char".to_string(),
        _ => ty.to_string(),
    }
}
pub(crate) fn get_field_type(
    table: &toml::map::Map<String, toml::Value>,
    has_sub: bool,
    field_name: String,
) -> RawField {
    let field_type = table.get("type").and_then(|v| v.as_str()).map(to_type);
    let enum_type = table.get("enum").and_then(|v| v.as_str());
    if let Some(ft) = field_type {
        if ft.starts_with('[') && ft.ends_with(']') {
            let inner_type = &ft[1..ft.len() - 1].trim();
            if is_native_type(inner_type) {
                return RawField {
                    type_name: format!("Vec<{}>", to_type(inner_type)),
                    is_native: true,
                    is_vec: true,
                    is_enum: false,
                };
            } else {
                // TODO:
                panic!("Non-native inner types in Vec are not supported yet");
                // return format!("Vec<{}Config>", to_pascal_case(inner_type));
            }
        }
        return RawField {
            type_name: ft.to_string(),
            is_native: is_native_type(&ft),
            is_vec: false,
            is_enum: false,
        };
    }
    if let Some(et) = enum_type {
        return RawField {
            type_name: et.to_string(),
            is_native: false,
            is_vec: false,
            is_enum: true,
        };
    }
    if has_sub {
        RawField {
            type_name: format!("{}Config", to_pascal_case(&field_name)),
            is_native: false,
            is_vec: false,
            is_enum: false,
        }
    } else {
        RawField {
            type_name: "String".to_string(),
            is_native: true,
            is_vec: false,
            is_enum: false,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct RawField {
    pub type_name: String,
    pub is_native: bool,
    pub is_vec: bool,
    pub is_enum: bool,
}
pub(crate) fn to_pascal_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
