use platipus_ir::IrModule;

pub fn render_api(module: &IrModule) -> String {
    let mut out = String::new();
    if module.apis.is_empty() {
        return out;
    }
    out.push_str("export const $api = {\n");
    for api in &module.apis {
        out.push_str(&format!("  {:?}: {{\n", api.name));
        for route in &api.routes {
            out.push_str(&format!(
                "    {:?}: plt.route({:?}, {:?}, {:?}),\n",
                route.name,
                route.method.as_str(),
                route.path,
                api.name
            ));
        }
        out.push_str("  },\n");
    }
    out.push_str("};\n\n");
    out
}

