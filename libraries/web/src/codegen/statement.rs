use crate::codegen::expression::{Scope, rewrite};
use crate::codegen::state::{function_access, local_access, param_access, state_access};
use platipus_ir::{IrFunction, IrStatement, StatementKind};

pub fn render_statement(statement: &IrStatement, scope: &Scope) -> String {
    let text = render_kind(&statement.kind, scope);
    if matches!(statement.kind, StatementKind::NoOp) {
        String::new()
    } else {
        text
    }
}

fn render_kind(kind: &StatementKind, scope: &Scope) -> String {
    match kind {
        StatementKind::NoOp => String::new(),
        StatementKind::Assign {
            target,
            operator,
            value,
        } => {
            let left = access_for(target, scope);
            format!("{} {operator} {}", left, rewrite(value, scope))
        }
        StatementKind::Expression(value) => rewrite(value, scope),
        StatementKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            let condition = rewrite(condition, scope);
            let mut out = format!("if ({condition}) {{ {} }}", block(then_branch, scope));
            if let Some(else_branch) = else_branch {
                if else_branch.is_empty() {
                    return out;
                }
                out.push_str(&format!(" else {{ {} }}", block(else_branch, scope)));
            }
            out
        }
        StatementKind::For {
            binding,
            iterable,
            body,
        } => {
            let mut inner = scope.clone();
            inner.bind(binding, format!("item_{}", binding));
            format!(
                "for (const item_{binding} of plt.iter({})) {{ {} }}",
                rewrite(iterable, scope),
                block(body, &inner)
            )
        }
        StatementKind::ForRange {
            binding,
            start,
            end,
            inclusive,
            body,
        } => {
            let mut inner = scope.clone();
            let counter = local_access(binding);
            let limit = format!("{counter}__end");
            inner.bind(binding, &counter);
            let test = if *inclusive { "<=" } else { "<" };
            format!(
                "for (let {counter} = {}, {limit} = {}; {counter} {test} {limit}; {counter}++) {{ {} }}",
                rewrite(start, scope),
                rewrite(end, scope),
                block(body, &inner)
            )
        }
        StatementKind::Let { name, value } => {
            format!("let {} = {}", local_access(name), rewrite(value, scope))
        }
        StatementKind::While { condition, body } => {
            format!(
                "while ({}) {{ {} }}",
                rewrite(condition, scope),
                block(body, scope)
            )
        }
        StatementKind::Return(value) => match value {
            Some(value) => format!("return {}", rewrite(value, scope)),
            None => "return".to_string(),
        },
        StatementKind::Break => "break".to_string(),
        StatementKind::Continue => "continue".to_string(),
        StatementKind::Try {
            binding,
            body,
            handler,
        } => {
            let catch_var = binding
                .as_deref()
                .map(local_access)
                .unwrap_or_else(|| "t".to_string());
            let mut inner = scope.clone();
            // The catch binding names the thrown value inside the handler; the
            // generated JavaScript has to bind it too.
            if let Some(name) = binding {
                inner.bind(name, &catch_var);
            }
            format!(
                "try {{ {} }} catch ({}) {{ {} }}",
                block(body, scope),
                catch_var,
                block(handler, &inner)
            )
        }
        StatementKind::Emit { event, payload } => {
            let target = match scope.access(event) {
                Some(access) => access.to_string(),
                None => format!("{:?}", dispatch_for(event)),
            };
            match payload {
                Some(payload) => format!("plt.emit({target}, {})", rewrite(payload, scope)),
                None => format!("plt.emit({target}, null)"),
            }
        }
        StatementKind::Block(body) => format!("{{ {} }}", block(body, scope)),
        StatementKind::Render(element) => {
            format!(
                "return {}",
                crate::codegen::components::render(element, scope)
            )
        }
        StatementKind::MisplacedDeclaration { .. } => String::new(),
    }
}

pub fn block(statements: &[IrStatement], scope: &Scope) -> String {
    let mut scoped = scope.clone();
    // Every direct `let` of this block is a JavaScript local of the same
    // block, so bind it before any statement is rendered.
    for statement in statements {
        if let StatementKind::Let { name, .. } = &statement.kind {
            scoped.bind(name, local_access(name));
        }
    }
    statements
        .iter()
        .map(|statement| render_statement(statement, &scoped))
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn dispatch_for(event: &str) -> String {
    match crate::codegen::events::dom_event(event) {
        Some(dom) if !is_custom_name(event) => format!("plt:{}", dom),
        _ => format!("plt:{}", event),
    }
}

fn is_custom_name(event: &str) -> bool {
    crate::codegen::events::dom_event(event).is_none()
}

fn access_for(target: &str, scope: &Scope) -> String {
    rewrite(target, scope)
}

pub fn render_function(function: &IrFunction, scope: &Scope) -> String {
    let mut inner = scope.clone();
    for parameter in &function.parameters {
        inner.bind(&parameter.name, param_access(&parameter.name));
    }
    let parameters = function
        .parameters
        .iter()
        .map(|parameter| {
            let default = parameter
                .default
                .as_ref()
                .map(|value| format!(" = {}", value))
                .unwrap_or_default();
            format!("{}{}", parameter_access(parameter), default)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let keyword = if function.is_async {
        "async function"
    } else {
        "function"
    };
    format!(
        "{keyword} {}({parameters}) {{ {} }}",
        function_access(&function.name),
        block(&function.body, &inner)
    )
}

fn parameter_access(parameter: &platipus_ir::IrInput) -> String {
    let name = param_access(&parameter.name);
    match parameter.type_name.as_deref() {
        Some("Int") | Some("Number") => name,
        Some("Text") | Some("String") => name,
        _ => name,
    }
}

/// Builds the statements that fill a component's `inputs` with its declared
/// defaults, so `inputs.name` accessors stay valid after the merge.
///
/// The defaults are written into the object the parent passed rather than into a
/// fresh one, because the component's `render` closes over that same binding.
/// Replacing it would leave the closure reading the values the instance was
/// constructed with, so a parent that changes an input would never reach the
/// child. The applied defaults are also recorded on `inputDefaults` so the
/// runtime can re-apply them whenever a reuse brings a fresh props object.
pub fn declare_inputs(inputs: &[platipus_ir::IrInput]) -> String {
    if inputs.is_empty() {
        return "  const inputDefaults = {};\n".to_string();
    }
    let defaults = inputs
        .iter()
        .map(|input| match input.default.as_deref() {
            Some(default) => format!("    {}: {default},", input.name),
            None => format!("    {}: null,", input.name),
        })
        .collect::<Vec<_>>()
        .join("\n");
    let fills = inputs
        .iter()
        .map(|input| {
            format!(
                "  inputs.{name} = inputs.{name} ?? inputDefaults.{name};",
                name = input.name
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("  const inputDefaults = {{\n{defaults}\n  }};\n{fills}")
}

pub fn state_slot(name: &str) -> String {
    state_access(name)
}

#[cfg(test)]
mod tests {
    use super::{access_for, declare_inputs};
    use crate::codegen::expression::Scope;
    use platipus_ir::IrInput;

    fn input(name: &str, default: Option<&str>) -> IrInput {
        IrInput {
            name: name.to_string(),
            type_name: None,
            default: default.map(str::to_string),
            required: false,
            span: platipus_diagnostics::Span::new(0, 0),
        }
    }

    #[test]
    fn a_default_fills_in_only_where_the_parent_passed_nothing() {
        let emitted = declare_inputs(&[
            input("value", Some("\"0\"")),
            input("caption", Some("\"\"")),
        ]);
        // The fill has to be written into the object the parent passed. Building
        // a new object here would leave the component's closure reading the
        // values it was constructed with, so a changed input never arrives.
        assert_eq!(
            emitted,
            "  const inputDefaults = {\n    value: \"0\",\n    caption: \"\",\n  };\n  \
             inputs.value = inputs.value ?? inputDefaults.value;\n  \
             inputs.caption = inputs.caption ?? inputDefaults.caption;"
        );
    }

    #[test]
    fn an_input_without_a_declared_default_falls_back_to_null() {
        let emitted = declare_inputs(&[input("extra", None)]);
        assert_eq!(
            emitted,
            "  const inputDefaults = {\n    extra: null,\n  };\n  \
             inputs.extra = inputs.extra ?? inputDefaults.extra;"
        );
    }

    #[test]
    fn a_component_with_no_inputs_still_records_an_empty_default_set() {
        // The returned object always names `inputDefaults`, so it has to exist
        // even when the component declares no inputs.
        assert_eq!(declare_inputs(&[]), "  const inputDefaults = {};\n");
    }

    #[test]
    fn assignment_targets_rewrite_names_inside_indices() {
        let mut scope = Scope::new();
        scope.bind("items", "s.items.value");
        scope.bind("index", "s.index.value");

        assert_eq!(
            access_for("items[index]", &scope),
            "s.items.value[s.index.value]"
        );
    }
}
