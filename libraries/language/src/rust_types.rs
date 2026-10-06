use crate::ast::rust::RustExport;

/// Types the FFI boundary supports. Anything else is reported as unsupported.
pub const SUPPORTED: &[&str] = &[
    "i64", "i32", "i16", "i8", "u64", "u32", "u16", "u8", "f64", "f32", "bool", "String",
];

/// Validates that every parameter and the return type are FFI-safe scalars.
pub fn check_signature(export: &RustExport) -> Result<(), (String,)> {
    for (_, ty) in &export.params {
        if !SUPPORTED.contains(&ty.as_str()) {
            return Err((ty.clone(),));
        }
    }
    if let Some(ty) = &export.return_type {
        if !SUPPORTED.contains(&ty.as_str()) {
            return Err((ty.clone(),));
        }
    }
    Ok(())
}
