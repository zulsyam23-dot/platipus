use crate::diagnostics::{Error, ErrorKind, Span};
use crate::ir::element::{ElementItem, IrElement};
use crate::ir::expression::IrStateKind;
use crate::ir::module::IrModule;
use crate::ir::statement::{IrStatement, StatementKind};
use crate::ir::{IrComponent, IrDerived, IrState};

pub use crate::ir::module::IrModule as IrProgram;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrError {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
}

impl IrError {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span: Some(span),
        }
    }

    pub fn into_error(self) -> Error {
        let mut error = Error::new(ErrorKind::Ir, self.code, self.message);
        if let Some(span) = self.span {
            error = error.with_span(span);
        }
        error
    }
}

pub fn verify(module: &IrModule) -> Result<(), Vec<IrError>> {
    let mut errors = Vec::new();
    if module.components.is_empty() {
        errors.push(IrError::new(
            "empty-module",
            "module contains no components to build",
            module.span,
        ));
        return Err(errors);
    }
    if module.components[0].name != module.name {
        errors.push(IrError::new(
            "app-name-mismatch",
            format!(
                "first component `{}` must be the app `{}`",
                module.components[0].name, module.name
            ),
            module.span,
        ));
    }
    for component in &module.components {
        verify_component(component, &mut errors);
    }
    for import in &module.imports {
        if import.path.trim().is_empty() {
            errors.push(IrError::new(
                "empty-import-path",
                format!("`{}` imports from an empty path", import.name),
                import.span,
            ));
        }
    }
    for theme in &module.themes {
        if theme.tokens.is_empty() {
            errors.push(IrError::new(
                "empty-theme",
                format!("theme `{}` declares no tokens", theme.name),
                theme.span,
            ));
        }
    }
    for test in &module.tests {
        if test.steps.is_empty() {
            errors.push(IrError::new(
                "empty-test",
                format!("test `{}` declares no steps", test.name),
                test.span,
            ));
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn verify_component(component: &IrComponent, errors: &mut Vec<IrError>) {
    let mut names: Vec<&str> = Vec::new();
    for state in &component.states {
        if names.contains(&state.name.as_str()) {
            errors.push(IrError::new(
                "duplicate-ir-symbol",
                format!("`{}` is lowered more than once", state.name),
                state.span,
            ));
        }
        names.push(&state.name);
    }
    for derived in &component.derived {
        if names.contains(&derived.name.as_str()) {
            errors.push(IrError::new(
                "duplicate-ir-symbol",
                format!("`{}` is lowered more than once", derived.name),
                derived.span,
            ));
        }
        names.push(&derived.name);
    }
    for input in &component.inputs {
        if input.required && input.default.is_some() {
            errors.push(IrError::new(
                "required-input-has-default",
                format!(
                    "input `{}` cannot be both required and defaulted",
                    input.name
                ),
                input.span,
            ));
        }
    }
    for derived in &component.derived {
        for dependency in &derived.dependencies {
            if !component
                .states
                .iter()
                .map(|state: &IrState| state.name.as_str())
                .any(|name| name == dependency)
                && !component
                    .derived
                    .iter()
                    .map(|item: &IrDerived| item.name.as_str())
                    .any(|name| name == dependency)
                && !component
                    .inputs
                    .iter()
                    .any(|input| input.name == *dependency)
            {
                errors.push(IrError::new(
                    "unknown-dependency",
                    format!(
                        "derived `{}` depends on unknown `{}`",
                        derived.name, dependency
                    ),
                    derived.span,
                ));
            }
        }
    }
    for function in &component.functions {
        for parameter in &function.parameters {
            if component
                .functions
                .iter()
                .any(|other| other.name == parameter.name && other.span != function.span)
            {
                errors.push(IrError::new(
                    "shadowed-parameter",
                    format!(
                        "parameter `{}` shadows another declaration of the same name",
                        parameter.name
                    ),
                    parameter.span,
                ));
            }
        }
        for statement in &function.body {
            verify_statement(statement, component, errors);
        }
    }
    for item in &component.body {
        verify_element_item(item, errors);
    }
    if component
        .handlers
        .iter()
        .any(|handler| handler.body.is_empty())
    {
        errors.push(IrError::new(
            "empty-handler",
            format!("component `{}` has an empty event handler", component.name),
            component.span,
        ));
    }

    if component
        .states
        .iter()
        .any(|state| state.kind == IrStateKind::Derived)
    {
        errors.push(IrError::new(
            "derived-in-states",
            "derived values must be lowered into the derived table, not the state table",
            component.span,
        ));
    }
}

fn verify_element_item(item: &ElementItem, errors: &mut Vec<IrError>) {
    match item {
        ElementItem::Child(element) => verify_element(element, errors),
        ElementItem::Statement(statement) => verify_bare_statement(statement, errors),
        ElementItem::Style(block) => {
            if block.entries.is_empty() && block.states.is_empty() {
                errors.push(IrError::new(
                    "empty-style",
                    "style block lowers to no declarations",
                    block.span,
                ));
            }
        }
        ElementItem::Responsive(block) => {
            if block.entries.is_empty() {
                errors.push(IrError::new(
                    "empty-responsive",
                    "`responsive { }` lowers to no breakpoint rules",
                    block.span,
                ));
            }
        }
    }
}

fn verify_element(element: &IrElement, errors: &mut Vec<IrError>) {
    if element.name.is_empty() {
        errors.push(IrError::new(
            "empty-element-name",
            "element lowered without a name",
            element.span,
        ));
    }
    for handler in &element.handlers {
        if handler.body.is_empty() {
            errors.push(IrError::new(
                "empty-handler",
                format!("`{}` has an empty event handler", element.name),
                handler.span,
            ));
        } else {
            for statement in &handler.body {
                verify_handler_statement(statement, &element.name, errors);
            }
        }
    }
    if let Some(body) = &element.body {
        for item in &body.items {
            verify_element_item(item, errors);
        }
    }
}

fn verify_handler_statement(statement: &IrStatement, element: &str, errors: &mut Vec<IrError>) {
    let mut stack: Vec<&IrStatement> = vec![statement];
    while let Some(current) = stack.pop() {
        if let StatementKind::MisplacedDeclaration { kind } = &current.kind {
            errors.push(IrError::new(
                "declaration-in-handler",
                format!("`{kind}` cannot be declared inside an event handler"),
                current.span,
            ));
            continue;
        }
        let mut bodies: Vec<&[IrStatement]> = Vec::new();
        match &current.kind {
            StatementKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                if let Some(branch) = else_branch.as_deref() {
                    bodies.push(branch);
                }
                bodies.push(then_branch);
            }
            StatementKind::For { body, .. } | StatementKind::Block(body) => bodies.push(body),
            StatementKind::Try { body, handler } => {
                bodies.push(handler);
                bodies.push(body);
            }
            _ => {}
        }
        for body in bodies {
            if body.is_empty() {
                errors.push(IrError::new(
                    "empty-branch",
                    format!("`{element}` has a control flow branch with no statements"),
                    current.span,
                ));
                continue;
            }
            stack.extend(body.iter().rev());
        }
    }
}

fn verify_statement(statement: &IrStatement, component: &IrComponent, errors: &mut Vec<IrError>) {
    match &statement.kind {
        StatementKind::Assign { target, .. } => {
            if !is_assignable(target, component) {
                errors.push(IrError::new(
                    "not-assignable",
                    format!("`{target}` cannot be assigned to"),
                    statement.span,
                ));
            }
        }
        StatementKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            for inner in then_branch {
                verify_statement(inner, component, errors);
            }
            if let Some(else_branch) = else_branch {
                for inner in else_branch {
                    verify_statement(inner, component, errors);
                }
            }
        }
        StatementKind::For { body, .. } => {
            for inner in body {
                verify_statement(inner, component, errors);
            }
        }
        StatementKind::Block(body) => {
            for inner in body {
                verify_statement(inner, component, errors);
            }
        }
        StatementKind::Try { body, handler } => {
            for inner in body.iter().chain(handler.iter()) {
                verify_statement(inner, component, errors);
            }
        }
        StatementKind::Emit { .. }
        | StatementKind::Expression(_)
        | StatementKind::Return(_)
        | StatementKind::Break
        | StatementKind::Continue
        | StatementKind::NoOp => {}
        StatementKind::Render(element) => verify_element(element, errors),
        StatementKind::MisplacedDeclaration { kind } => {
            errors.push(IrError::new(
                "declaration-in-function",
                format!("`{kind}` cannot be declared inside a function body"),
                statement.span,
            ));
        }
    }
}

fn verify_bare_statement(statement: &IrStatement, errors: &mut Vec<IrError>) {
    match &statement.kind {
        // An element body is a template: only control flow may appear there.
        StatementKind::If { .. } | StatementKind::For { .. } | StatementKind::NoOp => {}
        StatementKind::Block(_) | StatementKind::Render(_) => {}
        StatementKind::MisplacedDeclaration { kind } => errors.push(IrError::new(
            "declaration-in-body",
            format!("`{kind}` cannot be used as a runtime statement"),
            statement.span,
        )),
        StatementKind::Expression(value) if value.is_empty() => errors.push(IrError::new(
            "empty-expression",
            "expression statement lowers to nothing",
            statement.span,
        )),
        _ => errors.push(IrError::new(
            "not-a-template-statement",
            "only `if`, `for`, and element declarations are allowed inside an element body",
            statement.span,
        )),
    }
}

fn is_assignable(target: &str, component: &IrComponent) -> bool {
    let root = root_identifier(target);
    if component.derived.iter().any(|derived| derived.name == root) {
        return false;
    }
    if component.inputs.iter().any(|input| input.name == root) {
        return false;
    }
    true
}

fn root_identifier(target: &str) -> String {
    let mut root = String::new();
    for ch in target.chars() {
        if ch.is_alphanumeric() || ch == '_' {
            root.push(ch);
        } else {
            break;
        }
    }
    root
}
