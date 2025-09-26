use crate::PATH_BUF;

const NATIVE_TYPES: [&str; 20] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "bool",
    "f32", "f64", "f128", "String", "OsString", PATH_BUF, "char",
];
pub(crate) fn is_native_type(ty: &str) -> bool {
    NATIVE_TYPES.contains(&ty)
}
pub(crate) fn get_field_type(
    table: &toml::map::Map<String, toml::Value>,
    has_sub: bool,
    field_name: String,
) -> String {
    let field_type = table.get("type").and_then(|v| v.as_str());
    let enum_type = table.get("enum").and_then(|v| v.as_str());
    if let Some(ft) = field_type {
        return ft.to_string();
    }
    if let Some(et) = enum_type {
        return et.to_string();
    }
    if has_sub {
        format!("{}Config", to_pascal_case(&field_name))
    } else {
        "String".to_string()
    }
}
pub(crate) fn to_pascal_case(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
