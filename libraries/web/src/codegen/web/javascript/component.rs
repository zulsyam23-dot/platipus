use crate::codegen::components;
use crate::codegen::expression::{Scope, member_access, rewrite};
use crate::codegen::state::{is_global, is_persistent, is_reactive, scope_for, state_initializer, storage_scope};
use crate::codegen::statement::{declare_inputs, render_function};
use platipus_ir::{IrComponent, IrState};


pub fn render_component(component: &IrComponent, module: &platipus_ir::IrModule, rust: Option<&platipus_ir::RustBridge>) -> String {
    let mut scope = scope_for(component, module);
    let channels = event_channels(&component.emits);
    for event in &component.emits {
        let key = event.strip_prefix("plt:").unwrap_or(event.as_str());
        scope.bind(event, member_access("events", key));
    }
    if let Some(rust) = rust {
        for export in &rust.exports {
            scope.bind(&export.name, format!("__pltRust.{}", export.name));
        }
    }
    let mut out = String::new();
    out.push_str(&format!("function {}(inputs) {{\n", component.name));
    // The defaults are written into the caller's object so the closure over
    // `inputs` keeps seeing whatever the parent passes on the next render.
    out.push_str("  inputs = inputs ?? {};\n");
    out.push_str(&declare_inputs(&component.inputs));
    out.push('\n');
    out.push_str(&format!("  const events = {channels};\n"));
    out.push_str("  const s = {\n");
    for state in &component.states {
        out.push_str(&render_state(state, &scope));
    }
    out.push_str("  };\n");
    out.push_str("  const d = {};\n");
    for derived in &component.derived {
        out.push_str(&format!(
            "  d.{} = plt.computed(() => ({}), [{}], s, d);\n",
            derived.name,
            rewrite(&derived.value, &scope),
            derived
                .dependencies
                .iter()
                .map(|dependency| format!("{dependency:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    for function in &component.functions {
        out.push_str("  ");
        out.push_str(&render_function(function, &scope));
        out.push('\n');
    }
    out.push_str("  const render = () => ");
    // A component body may hold several sibling roots, or none at all when it is
    // only a branch. Taking just the first element would drop everything after
    // it, and a body of only branches would render nothing, so every root is
    // emitted. More than one is wrapped in a tagless fragment, which the runtime
    // flattens: that keeps the roots as direct children of the parent instead
    // of inserting a wrapper element that would change the layout.
    let roots = components::render_items(&component.body, &scope);
    match roots.len() {
        0 => out.push_str("null"),
        1 => out.push_str(&roots[0]),
        _ => out.push_str(&format!("plt.el(null, {{}}, [{}])", roots.join(", "))),
    }
    out.push_str(";\n");
    out.push_str("  const vnode = render();\n");
    // A lifecycle handler declared on the component itself is called by the
    // runtime, not bound to a DOM event.
    let lifecycle = components::merge_handlers(&components::lifecycle_entries(
        &components::lifecycle(&component.handlers),
        &scope,
    ));
    match lifecycle {
        Some(lifecycle) => out.push_str(&format!("  const life = {lifecycle};\n")),
        None => out.push_str("  const life = {};\n"),
    }
    out.push_str("  return { s, d, events, render, vnode, life, inputs, inputDefaults };\n");
    out.push_str("}\n");
    out.push_str(&format!(
        "$components[{:?}] = {};\n",
        component.name, component.name
    ));
    out
}

fn render_state(state: &IrState, scope: &Scope) -> String {
    let value = rewrite(&state_initializer(state), scope);
    if is_global(&state.kind) {
        return format!(
            "    {}: plt.global({:?}, {}), // {}\n",
            state.name,
            state.name,
            value,
            storage_scope(&state.kind)
        );
    }
    if is_persistent(&state.kind) {
        return format!(
            "    {}: plt.persistent({:?}, {}), // {}\n",
            state.name,
            state.name,
            value,
            storage_scope(&state.kind)
        );
    }
    if is_reactive(state.kind) {
        return format!(
            "    {}: plt.signal({value}), // {}\n",
            state.name,
            storage_scope(&state.kind)
        );
    }
    format!("    {}: plt.ref({value}),\n", state.name)
}

fn event_channels(emits: &[String]) -> String {
    if emits.is_empty() {
        return "{}".to_string();
    }
    let entries = emits
        .iter()
        .map(|event| {
            let key = event.strip_prefix("plt:").unwrap_or(event.as_str());
            format!("{key:?}: plt.emitter({key:?})")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!("{{{entries}}}")
}

