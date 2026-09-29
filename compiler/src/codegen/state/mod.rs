use crate::codegen::expression::Scope;
use crate::ir::{IrComponent, IrDerived, IrInput, IrState, IrStateKind};

/// Access path for a state name inside a component factory.
pub fn state_access(name: &str) -> String {
    format!("s.{}.value", name)
}

pub fn derived_access(name: &str) -> String {
    format!("d.{}.value", name)
}
pub fn function_access(name: &str) -> String {
    format!("f_{}", sanitize(name))
}

/// JavaScript-safe identifier for a function parameter. The Platipus name can
/// contain characters that are legal in neither position.
pub fn param_access(name: &str) -> String {
    format!("p_{}", sanitize(name))
}

pub fn input_access(name: &str) -> String {
    format!("inputs.{}", name)
}

pub fn local_access(name: &str) -> String {
    format!("t_{}", sanitize(name))
}

/// Replaces every character JavaScript will not accept in an identifier.
pub fn sanitize(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for (index, ch) in name.chars().enumerate() {
        let ok = if index == 0 {
            ch.is_alphabetic() || ch == '_' || ch == '$'
        } else {
            ch.is_alphanumeric() || ch == '_' || ch == '$'
        };
        if ok {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

/// The browser runtime functions a `plt` script can call by name inside a
/// component body. Each lowers to a call on the generated `plt` object. They
/// are bound before user declarations, so a state or function with the same
/// name shadows the builtin.
pub fn bind_builtins(scope: &mut Scope) {
    for (name, access) in [
        ("fetch", "plt.fetch"),
        ("writeClipboard", "plt.clipboardWrite"),
        ("readClipboard", "plt.clipboardRead"),
        ("openFile", "plt.openFile"),
        ("webSocket", "plt.webSocket"),
        ("receive", "plt.receive"),
        ("store", "plt.store"),
        ("load", "plt.load"),
        ("drop", "plt.drop"),
        ("canvas", "plt.canvas"),
        ("fill", "plt.fill"),
        ("clear", "plt.clear"),
        ("drawText", "plt.drawText"),
        ("nextFrame", "plt.nextFrame"),
        ("wait", "plt.wait"),
        ("exec", "plt.exec"),
        ("selection", "plt.selection"),
        ("indent", "plt.indent"),
        ("sortBy", "plt.sortBy"),
        ("page", "plt.page"),
    ] {
        scope.bind(name, access);
    }
}

/// Builds the name resolution scope for a component body.
pub fn scope_for(component: &IrComponent) -> Scope {
    let mut scope = Scope::new();
    bind_builtins(&mut scope);
    for input in &component.inputs {
        scope.bind(&input.name, input_access(&input.name));
    }
    for state in &component.states {
        scope.bind(&state.name, state_access(&state.name));
    }
    for derived in &component.derived {
        scope.bind(&derived.name, derived_access(&derived.name));
    }
    for function in &component.functions {
        scope.bind(&function.name, function_access(&function.name));
    }
    scope
}

/// Extends a scope with function parameter bindings and local declarations.
pub fn scope_with_locals(mut scope: Scope, parameters: &[IrInput], locals: &[IrState]) -> Scope {
    for parameter in parameters {
        scope.bind(&parameter.name, param_access(&parameter.name));
    }
    for local in locals {
        scope.bind(&local.name, local_access(&local.name));
    }
    scope
}

pub fn is_reactive(kind: IrStateKind) -> bool {
    kind.is_reactive()
}

pub fn storage_scope(kind: &IrStateKind) -> &'static str {
    match kind {
        IrStateKind::Local => "instance",
        IrStateKind::Shared => "shared",
        IrStateKind::Global => "global",
        IrStateKind::Persistent => "persistent",
        IrStateKind::Derived => "derived",
    }
}

pub fn state_initializer(state: &IrState) -> String {
    state
        .initializer
        .as_ref()
        .map(|initializer| initializer.value.clone())
        .unwrap_or_else(|| "null".to_string())
}

pub fn is_persistent(kind: &IrStateKind) -> bool {
    matches!(kind, IrStateKind::Persistent)
}

pub fn is_global(kind: &IrStateKind) -> bool {
    matches!(kind, IrStateKind::Global)
}

pub fn derived_dependencies(derived: &IrDerived) -> &[String] {
    &derived.dependencies
}
