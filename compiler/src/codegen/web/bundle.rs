use crate::ir::{IrModule, IrTestStep};

/// Serialisable description of the compiled program, used by tooling and
/// the runtime loader.
pub fn manifest(module: &IrModule) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"name\": {:?},\n", module.name));
    out.push_str(&format!(
        "  \"components\": [{}],\n",
        names(
            &module
                .components
                .iter()
                .map(|component| &component.name)
                .collect::<Vec<_>>()
        )
    ));
    out.push_str(&format!(
        "  \"apis\": [{}],\n",
        names(&module.apis.iter().map(|api| &api.name).collect::<Vec<_>>())
    ));
    out.push_str("  \"tests\": [");
    if module.tests.is_empty() {
        out.push_str("],\n");
    } else {
        out.push('\n');
        let tests: Vec<String> = module
            .tests
            .iter()
            .map(|test| {
                let steps: Vec<String> = test
                    .steps
                    .iter()
                    .map(|step| match step {
                        IrTestStep::Action { name, argument, .. } => match argument {
                            Some(argument) => {
                                format!("{{ \"action\": {name:?}, \"argument\": {argument:?} }}")
                            }
                            None => format!("{{ \"action\": {name:?} }}"),
                        },
                        IrTestStep::Expect { expression, .. } => {
                            format!("{{ \"expect\": {expression:?} }}")
                        }
                    })
                    .collect();
                format!(
                    "    {{ \"name\": {:?}, \"steps\": [{}] }}",
                    test.name,
                    steps.join(", ")
                )
            })
            .collect();
        out.push_str(&tests.join(",\n"));
        out.push_str("\n  ],\n");
    }
    out.push_str("  \"routes\": ");
    let routes: Vec<String> = module
        .apis
        .iter()
        .flat_map(|api| {
            api.routes.iter().map(move |route| {
                format!(
                    "{{ \"api\": {:?}, \"name\": {:?}, \"method\": {:?}, \"path\": {:?} }}",
                    api.name,
                    route.name,
                    route.method.as_str(),
                    route.path
                )
            })
        })
        .collect();
    if routes.is_empty() {
        out.push_str("[],\n");
    } else {
        out.push_str("[\n    ");
        out.push_str(&routes.join(",\n    "));
        out.push_str("\n  ],\n");
    }
    out.push_str("  \"entry\": \"index.html\"\n");
    out.push_str("}\n");
    out
}

fn names(items: &[&String]) -> String {
    items
        .iter()
        .map(|name| format!("{name:?}"))
        .collect::<Vec<_>>()
        .join(", ")
}
