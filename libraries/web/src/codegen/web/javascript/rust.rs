
pub fn render_rust_bridge(rust: &platipus_ir::RustBridge) -> String {
    let mut out = String::from("// Native Rust functions compiled to wasm\nconst __pltRust = (() => {\n");
    match &rust.wasm_b64 {
        Some(b64) => {
            out.push_str(&format!(
                "  const bin = atob({b64:?});\n  const bytes = new Uint8Array(bin.length);\n  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);\n  const mod = new WebAssembly.Module(bytes);\n  const inst = new WebAssembly.Instance(mod, {{}});\n  return {{\n"
            ));
            for export in &rust.exports {
                out.push_str(&format!(
                    "    {}: {},\n",
                    export.name,
                    rust_export_wrapper(export)
                ));
            }
            out.push_str("  };\n})();\n\n");
        }
        None => {
            out.push_str("  return {\n");
            for export in &rust.exports {
                out.push_str(&format!(
                    "    {}: () => {{ throw new Error(\"Rust function `{}` was not compiled to wasm\"); }},\n",
                    export.name, export.name
                ));
            }
            out.push_str("  };\n})();\n\n");
        }
    }
    out
}

/// Builds a JS callable for one Rust export, converting between JS types and
/// the wasm ABI. Scalars map to `BigInt`/`number`/`boolean`; `String` params
/// are written into the wasm heap with `__pltAlloc`, and `String` returns are
/// read back from the heap as packed `(ptr << 32) | len`.
fn rust_export_wrapper(export: &platipus_language::ast::rust::RustExport) -> String {
    if export.params.iter().any(|(_, ty)| ty == "String")
        || export.return_type.as_deref() == Some("String")
    {
        return rust_string_wrapper(export);
    }
    let mut params = Vec::new();
    let mut args = Vec::new();
    for (name, ty) in &export.params {
        params.push(name.clone());
        if is_i64(ty) {
            args.push(format!("BigInt({name})"));
        } else {
            args.push(name.clone());
        }
    }
    let call = format!("inst.exports.{}({})", export.name, args.join(", "));
    let body = match &export.return_type {
        Some(ty) if is_i64(ty) => format!("Number({call})"),
        _ => call,
    };
    format!("({}) => {}", params.join(", "), body)
}

fn rust_string_wrapper(export: &platipus_language::ast::rust::RustExport) -> String {
    let params = export
        .params
        .iter()
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    let mut setup = String::new();
    let mut args: Vec<String> = Vec::new();
    for (name, ty) in &export.params {
        if ty == "String" {
            setup.push_str(&format!(
                "  const {name}Bytes = new TextEncoder().encode({name});\n  const {name}Ptr = inst.exports.__pltAlloc({name}Bytes.length);\n  new Uint8Array(inst.exports.memory.buffer, {name}Ptr, {name}Bytes.length).set({name}Bytes);\n"
            ));
            args.push(format!("{name}Ptr"));
            args.push(format!("{name}Bytes.length"));
        } else if is_i64(ty) {
            args.push(format!("BigInt({name})"));
        } else {
            args.push(name.clone());
        }
    }
    let call = format!("inst.exports.{}({})", export.name, args.join(", "));
    let is_string_ret = export.return_type.as_deref() == Some("String");
    let body = if is_string_ret {
        format!(
            "  const raw = {call};\n  const ptr = Number(raw >> 32n);\n  const len = Number(raw & 0xffffffffn);\n  return new TextDecoder().decode(new Uint8Array(inst.exports.memory.buffer, ptr, len));\n"
        )
    } else {
        match &export.return_type {
            Some(ty) if is_i64(ty) => format!("  return Number({call});\n"),
            Some(ty) if ty == "bool" => format!("  return Boolean({call});\n"),
            Some(_) => format!("  return {call};\n"),
            None => format!("  {call};\n"),
        }
    };
    format!("({}) => {{\n{}{}}}", params, setup, body)
}

pub fn is_i64(ty: &str) -> bool {
    matches!(ty, "i64" | "u64")
}

