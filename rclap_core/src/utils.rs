const NATIVE_TYPES: [&str; 19] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "bool",
    "f32", "f64", "f128", "String", "OsString", "PathBuf",
];
pub(crate) fn is_native_type(ty: &str) -> bool {
    NATIVE_TYPES.contains(&ty)
}
