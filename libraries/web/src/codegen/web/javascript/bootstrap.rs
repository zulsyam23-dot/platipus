use platipus_ir::IrModule;

pub fn render_bootstrap(module: &IrModule) -> String {
    let app = match module.components.first() {
        Some(app) => app,
        None => return String::new(),
    };
    let mut out = String::new();
    out.push_str("export function mount(target) {\n");
    out.push_str("  if (target.__plt) plt.dispose(target.__plt);\n");
    out.push_str(&format!("  const instance = {}();\n", app.name));
    out.push_str("  instance.root = plt.host(instance);\n");
    out.push_str("  target.replaceChildren(instance.root);\n");
    out.push_str("  plt.bind(target, instance);\n");
    out.push_str("  return instance;\n");
    out.push_str("}\n\n");
    out.push_str("if (typeof document !== \"undefined\") {\n");
    out.push_str("  const root = document.getElementById(\"root\");\n");
    out.push_str("  if (root) mount(root);\n");
    out.push_str("}\n");
    out
}

