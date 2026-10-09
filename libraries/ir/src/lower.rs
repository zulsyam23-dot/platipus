use platipus_language::ast::{
    self, AppDecl, AssignOp, BinaryOp, ComponentDecl, ComponentItem, Element, EventCategory,
    Expression, ForIterable, Identifier, Program, Statement, UnaryOp,
};
use crate::element::ElementItem as IrElementItem;
use crate::statement::StatementKind;
use crate::{
    IrApi, IrBinding, IrComponent, IrDerived, IrElement, IrElementBody, IrExpression, IrFunction,
    IrHandler, IrImport, IrInput, IrModule, IrProperty, IrResponsiveBlock, IrResponsiveEntry,
    IrRoute, IrState, IrStateKind, IrStatement, IrStyleBlock, IrStyleEntry, IrStyleState, IrTest,
    IrTestStep, IrTheme, IrThemeToken,
};
use platipus_language::events;

pub struct Lowering<'a> {
    components: Vec<&'a str>,
    /// Every `fn` name in the module (component-level and top-level), so
    /// a call to one is not mistaken for a reactive dependency.
    fn_names: Vec<&'a str>,
}

impl<'a> Lowering<'a> {
    pub fn new(components: Vec<&'a str>, fn_names: Vec<&'a str>) -> Self {
        Self { components, fn_names }
    }

    pub fn from_program(program: &'a Program) -> Self {
        let mut fn_names: Vec<&'a str> = program
            .functions
            .iter()
            .map(|decl| decl.name.as_str())
            .collect();
        let mut bodies: Vec<&[ComponentItem]> = Vec::new();
        if let Some(app) = &program.app {
            bodies.push(&app.body);
        }
        for component in &program.components {
            bodies.push(&component.body);
        }
        for body in bodies {
            for item in body {
                if let ComponentItem::Function(decl) = item {
                    fn_names.push(decl.name.as_str());
                }
            }
        }
        Self::new(
            program
                .components
                .iter()
                .map(|component| component.name.as_str())
                .collect(),
            fn_names,
        )
    }

    pub fn is_component(&self, name: &str) -> bool {
        self.components.contains(&name)
    }
}

pub fn lower_program(program: &Program) -> Option<IrModule> {
    let lowering = Lowering::from_program(program);
    let component = program
        .app
        .as_ref()
        .map(|app| lowering.lower_component_from_app(app));
    let mut components: Vec<IrComponent> = Vec::new();
    if let Some(component) = component {
        components.push(component);
    }
    components.extend(
        program
            .components
            .iter()
            .map(|component| lowering.lower_component(component)),
    );
    let functions: Vec<IrFunction> = program
        .functions
        .iter()
        .map(|decl| IrFunction {
            name: decl.name.as_str().to_string(),
            is_async: decl.is_async,
            parameters: decl.parameters.iter().map(lower_parameter).collect(),
            return_type: decl
                .return_type
                .as_ref()
                .map(|annotation| annotation.base_name().to_string()),
            body: decl
                .body
                .statements
                .iter()
                .map(|nested| lowering.lower_statement(nested))
                .collect(),
            span: decl.span,
        })
        .collect();
    let apis = program.apis.iter().map(lower_api).collect();
    let styles = program
        .styles
        .iter()
        .filter_map(|style| {
            let mut block = lower_style_block(&style.block)?;
            block.name = Some(style.name.as_str().to_string());
            Some(block)
        })
        .collect();
    let themes = program
        .themes
        .iter()
        .map(|theme| IrTheme {
            name: theme.name.as_str().to_string(),
            tokens: theme
                .tokens
                .iter()
                .map(|token| IrThemeToken {
                    path: token.qualified(),
                    value: lower_style_value(&token.value),
                    span: token.span,
                })
                .collect(),
            span: theme.span,
        })
        .collect();
    let imports = program
        .imports
        .iter()
        .map(|import| IrImport {
            name: import.name.as_str().to_string(),
            path: import.path.clone(),
            span: import.span,
        })
        .collect();
    let tests = program
        .tests
        .iter()
        .map(|test| IrTest {
            name: test.name.as_str().to_string(),
            steps: test
                .steps
                .iter()
                .map(|step| match step {
                    platipus_language::ast::TestStep::Action {
                        name,
                        argument,
                        span,
                    } => IrTestStep::Action {
                        name: name.as_str().to_string(),
                        argument: argument.as_ref().map(lower_expression),
                        span: *span,
                    },
                    platipus_language::ast::TestStep::Expect { expression, span } => IrTestStep::Expect {
                        expression: lower_expression(expression),
                        span: *span,
                    },
                })
                .collect(),
            span: test.span,
        })
        .collect();
    Some(IrModule {
        name: program
            .app
            .as_ref()
            .map(|app| app.name.as_str().to_string())
            .unwrap_or_else(|| {
                std::path::Path::new(&program.path)
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_string())
                    .unwrap_or_else(|| "module".to_string())
            }),
        components,
        styles,
        themes,
        apis,
        imports,
        tests,
        functions,
        span: program.app.as_ref().map(|app| app.span).unwrap_or(program.span),
    })
}

fn lower_api(api: &platipus_language::ast::ApiDecl) -> IrApi {
    IrApi {
        name: api.name.as_str().to_string(),
        routes: api
            .routes
            .iter()
            .map(|route| IrRoute {
                method: route.method,
                name: route.name.as_str().to_string(),
                path: route.path.clone(),
                span: route.span,
            })
            .collect(),
        span: api.span,
    }
}

impl<'a> Lowering<'a> {
    fn lower_component(&self, component: &ComponentDecl) -> IrComponent {
        IrComponent {
            name: component.name.as_str().to_string(),
            is_app: false,
            inputs: component.inputs.iter().map(lower_input).collect(),
            states: Vec::new(),
            derived: Vec::new(),
            functions: Vec::new(),
            handlers: Vec::new(),
            body: Vec::new(),
            emits: component
                .emits()
                .into_iter()
                .map(|emit| emit.as_str().to_string())
                .collect(),
            span: component.span,
        }
        .tap_items(self, &component.body)
    }

    fn lower_component_from_app(&self, app: &AppDecl) -> IrComponent {
        IrComponent {
            name: app.name.as_str().to_string(),
            is_app: true,
            inputs: app.inputs.iter().map(lower_input).collect(),
            states: Vec::new(),
            derived: Vec::new(),
            functions: Vec::new(),
            handlers: Vec::new(),
            body: Vec::new(),
            emits: app
                .emits()
                .into_iter()
                .map(|emit| emit.as_str().to_string())
                .collect(),
            span: app.span,
        }
        .tap_items(self, &app.body)
    }
}

trait TapItems<'a> {
    fn tap_items(self, lowering: &Lowering<'a>, items: &[ComponentItem]) -> Self;
}

impl<'a> TapItems<'a> for IrComponent {
    fn tap_items(mut self, lowering: &Lowering<'a>, items: &[ComponentItem]) -> Self {
        for item in items {
            match item {
                ComponentItem::State(decl) => self.states.push(IrState {
                    name: decl.name.as_str().to_string(),
                    kind: IrStateKind::from(decl.kind),
                    type_name: decl
                        .type_annotation
                        .as_ref()
                        .map(|annotation| annotation.base_name().to_string()),
                    initializer: decl.initializer.as_ref().map(|e| lower_reactive(e, &lowering.fn_names)),
                    span: decl.span,
                }),
                ComponentItem::Derived(decl) => self.derived.push(IrDerived {
                    name: decl.name.as_str().to_string(),
                    type_name: decl
                        .type_annotation
                        .as_ref()
                        .map(|annotation| annotation.base_name().to_string()),
                    value: lower_expression(&decl.value),
                    dependencies: dependencies(&decl.value, &lowering.fn_names),
                    span: decl.span,
                }),
                ComponentItem::Function(decl) => self.functions.push(IrFunction {
                    name: decl.name.as_str().to_string(),
                    is_async: decl.is_async,
                    parameters: decl.parameters.iter().map(lower_parameter).collect(),
                    return_type: decl
                        .return_type
                        .as_ref()
                        .map(|annotation| annotation.base_name().to_string()),
                    body: decl
                        .body
                        .statements
                        .iter()
                        .map(|nested| lowering.lower_statement(nested))
                        .collect(),
                    span: decl.span,
                }),
                ComponentItem::Handler(handler) => {
                    self.handlers.push(lowering.lower_handler(handler));
                }
                ComponentItem::Child(element) => {
                    if let Some(ir_element) = lowering.lower_element(element) {
                        self.body.push(IrElementItem::Child(ir_element));
                    }
                }
                ComponentItem::Style(block) => {
                    if let Some(block) = lower_style_block(block) {
                        self.body.push(IrElementItem::Style(block));
                    }
                }
                ComponentItem::Stmt(statement) => {
                    self.body.push(IrElementItem::Statement(
                        lowering.lower_statement(statement),
                    ));
                }
                // The parser only produces `import` and `test` at the top level
                // of a file, so a component can never carry one.
                ComponentItem::Import(_) | ComponentItem::Test(_) => {}
            }
        }
        self
    }
}

fn lower_input(input: &platipus_language::ast::InputDecl) -> IrInput {
    IrInput {
        name: input.name.as_str().to_string(),
        type_name: input
            .type_annotation
            .as_ref()
            .map(|annotation| annotation.base_name().to_string()),
        default: input.default.as_ref().map(lower_expression),
        required: input.required(),
        span: input.span,
    }
}

fn lower_parameter(parameter: &platipus_language::ast::Parameter) -> IrInput {
    IrInput {
        name: parameter.name.as_str().to_string(),
        type_name: parameter
            .type_annotation
            .as_ref()
            .map(|annotation| annotation.base_name().to_string()),
        default: parameter.default.as_ref().map(lower_expression),
        required: parameter.default.is_none(),
        span: parameter.span,
    }
}

fn lower_reactive(expression: &Expression, fn_names: &[&str]) -> IrExpression {
    IrExpression {
        value: lower_expression(expression),
        dependencies: dependencies(expression, fn_names),
        span: expression.span(),
    }
}

/// Collects the reactive dependencies of an expression: identifiers that
/// are *not* builtins and *not* declared functions (calls to those are
/// constants unless their arguments subscribe), and *not* lambda-renamed
/// parameter names already baked into IR text.
fn dependencies(expression: &Expression, fn_names: &[&str]) -> Vec<String> {
    let mut found: Vec<Identifier> = Vec::new();
    platipus_language::ast::collect_identifiers(expression, &mut found);
    let mut names: Vec<String> = found
        .into_iter()
        .map(|identifier| identifier.name)
        .filter(|name| !platipus_standard::is_builtin_name(name))
        .filter(|name| !fn_names.contains(&name.as_str()))
        .collect();
    names.sort();
    names.dedup();
    names
}

impl<'a> Lowering<'a> {
    fn lower_element(&self, element: &Element) -> Option<IrElement> {
        let name = element.name.as_str();
        let kind = if self.is_component(name) {
            crate::element::ElementKind::Component
        } else {
            crate::element::ElementKind::Primitive
        };
        let mut properties: Vec<IrProperty> = element
            .arguments
            .iter()
            .map(|property| IrProperty {
                name: property.name.as_str().to_string(),
                value: lower_reactive(&property.value, &self.fn_names),
                span: property.span,
            })
            .collect();
        let mut bindings: Vec<IrBinding> = Vec::new();
        let mut handlers: Vec<IrHandler> = Vec::new();
        let body = element.body.as_ref().map(|body| {
            let mut items: Vec<IrElementItem> = Vec::new();
            for item in &body.items {
                match item {
                    ast::ElementItem::Child(child) => {
                        if let Some(ir_child) = self.lower_element(child) {
                            items.push(IrElementItem::Child(ir_child));
                        }
                    }
                    ast::ElementItem::Property(property) => properties.push(IrProperty {
                        name: property.name.as_str().to_string(),
                        value: lower_reactive(&property.value, &self.fn_names),
                        span: property.span,
                    }),
                    ast::ElementItem::Handler(handler) => {
                        handlers.push(self.lower_handler(handler))
                    }
                    ast::ElementItem::Style(style) => {
                        if let Some(ir_style) = lower_style_block(style) {
                            items.push(IrElementItem::Style(ir_style));
                        }
                    }
                    ast::ElementItem::Responsive(responsive) => {
                        items.push(IrElementItem::Responsive(lower_responsive(responsive)))
                    }
                    ast::ElementItem::Binding(binding) => bindings.push(IrBinding {
                        name: binding.property.as_str().to_string(),
                        value: lower_reactive(&binding.target, &self.fn_names),
                        span: binding.span,
                    }),
                    ast::ElementItem::Stmt(statement) => {
                        items.push(IrElementItem::Statement(self.lower_statement(statement)))
                    }
                }
            }
            Box::new(IrElementBody {
                items,
                span: body.span,
            })
        });
        Some(IrElement {
            name: name.to_string(),
            kind,
            properties,
            bindings,
            text: element.text.as_ref().map(|e| lower_reactive(e, &self.fn_names)),
            handlers,
            body,
            span: element.span,
        })
    }
}

fn lower_responsive(block: &platipus_language::ast::ResponsiveBlock) -> IrResponsiveBlock {
    let entries = block
        .entries
        .iter()
        .map(|entry| {
            let breakpoint = entry.breakpoint.as_str().to_string();
            match &entry.kind {
                platipus_language::ast::ResponsiveEntryKind::Property { name, value } => {
                    IrResponsiveEntry::Property {
                        breakpoint,
                        name: name.as_str().to_string(),
                        value: lower_style_value(value),
                        span: entry.span,
                    }
                }
                platipus_language::ast::ResponsiveEntryKind::Named { name } => IrResponsiveEntry::Named {
                    breakpoint,
                    name: name.as_str().to_string(),
                    span: entry.span,
                },
            }
        })
        .collect();
    IrResponsiveBlock {
        entries,
        span: block.span,
    }
}

fn lower_style_block(block: &platipus_language::ast::StyleBlock) -> Option<IrStyleBlock> {
    let entries: Vec<IrStyleEntry> = block
        .entries
        .iter()
        .map(|entry| match entry {
            platipus_language::ast::StyleEntry::Property { name, value, span } => IrStyleEntry {
                name: name.as_str().to_string(),
                value: lower_style_value(value),
                span: *span,
            },
            platipus_language::ast::StyleEntry::Reference { name, span } => IrStyleEntry {
                name: name.as_str().to_string(),
                value: format!("var(--{})", name.as_str()),
                span: *span,
            },
        })
        .collect();
    let states: Vec<(IrStyleState, IrStyleBlock)> = block
        .groups
        .iter()
        .filter_map(|group| {
            let state = IrStyleState::from_name(group.state.as_str())?;
            let nested = lower_style_block(&group.block)?;
            Some((state, nested))
        })
        .collect();
    Some(IrStyleBlock {
        name: None,
        entries,
        states,
        span: block.span,
    })
}

impl<'a> Lowering<'a> {
    fn lower_handler(&self, handler: &platipus_language::ast::EventHandlerDecl) -> IrHandler {
        let name = handler.name.as_str();
        let category = events::category_of(name).unwrap_or(EventCategory::Custom);
        IrHandler {
            event: name.to_string(),
            category: crate::IrEventCategory(category),
            is_custom: !events::is_known(name),
            body: handler
                .body
                .statements
                .iter()
                .map(|nested| self.lower_statement(nested))
                .collect(),
            span: handler.span,
        }
    }
}

impl<'a> Lowering<'a> {
    fn lower_statement(&self, statement: &Statement) -> IrStatement {
        let span = statement.span();
        let kind = match statement {
            Statement::Assignment(assignment) => StatementKind::Assign {
                target: lower_expression(&assignment.target),
                operator: assign_op(assignment.op).to_string(),
                value: lower_expression(&assignment.value),
            },
            Statement::Expression(expression) => {
                StatementKind::Expression(lower_expression(expression))
            }
            Statement::If(statement) => StatementKind::If {
                condition: lower_expression(&statement.condition),
                then_branch: statement
                    .then_branch
                    .statements
                    .iter()
                    .map(|nested| self.lower_statement(nested))
                    .collect(),
                else_branch: statement
                    .else_branch
                    .as_ref()
                    .map(|branch| match branch.as_ref() {
                        platipus_language::ast::ElseBranch::Block(block) => block
                            .statements
                            .iter()
                            .map(|nested| self.lower_statement(nested))
                            .collect(),
                        platipus_language::ast::ElseBranch::If(nested) => {
                            vec![self.lower_statement(&Statement::If(nested.clone()))]
                        }
                    }),
            },
            Statement::For(statement) => {
                let body = statement
                    .body
                    .statements
                    .iter()
                    .map(|nested| self.lower_statement(nested))
                    .collect();
                let binding = statement.binding.as_str().to_string();
                match &statement.iterable {
                    ForIterable::Value(iterable) => StatementKind::For {
                        binding,
                        iterable: lower_expression(iterable),
                        body,
                    },
                    ForIterable::Range {
                        start, end, inclusive, ..
                    } => StatementKind::ForRange {
                        binding,
                        start: lower_expression(start),
                        end: lower_expression(end),
                        inclusive: *inclusive,
                        body,
                    },
                }
            }
            Statement::Let(statement) => StatementKind::Let {
                name: statement.name.as_str().to_string(),
                value: lower_expression(&statement.initializer),
            },
            Statement::While(statement) => StatementKind::While {
                condition: lower_expression(&statement.condition),
                body: statement
                    .body
                    .statements
                    .iter()
                    .map(|nested| self.lower_statement(nested))
                    .collect(),
            },
            Statement::Return(statement) => {
                StatementKind::Return(statement.value.as_ref().map(lower_expression))
            }
            Statement::Break(_) => StatementKind::Break,
            Statement::Continue(_) => StatementKind::Continue,
            Statement::Try(statement) => StatementKind::Try {
                binding: statement.binding.as_ref().map(|b| b.name.as_str().to_string()),
                body: statement
                    .body
                    .statements
                    .iter()
                    .map(|nested| self.lower_statement(nested))
                    .collect(),
                handler: statement
                    .handler
                    .statements
                    .iter()
                    .map(|nested| self.lower_statement(nested))
                    .collect(),
            },
            Statement::Emit(emit) => StatementKind::Emit {
                event: emit.name.as_str().to_string(),
                payload: emit.payload().map(lower_expression),
            },
            Statement::Block(block) => StatementKind::Block(
                block
                    .statements
                    .iter()
                    .map(|nested| self.lower_statement(nested))
                    .collect(),
            ),
            Statement::Empty(_) => StatementKind::NoOp,
            // A function may build markup, so an element statement is legal
            // here; every other declaration belongs to the component body.
            Statement::Element(element) => match self.lower_element(element) {
                Some(element) => StatementKind::Render(Box::new(element)),
                None => StatementKind::NoOp,
            },
            Statement::State(_) => misplaced("state"),
            Statement::Derived(_) => misplaced("derived"),
            Statement::Function(_) => misplaced("fn"),
            Statement::Handler(_) => misplaced("on"),
            Statement::Import(_) => misplaced("import"),
            Statement::Test(_) => misplaced("test"),
        };
        IrStatement::new(kind, span)
    }
}

fn misplaced(kind: &'static str) -> StatementKind {
    StatementKind::MisplacedDeclaration { kind }
}

fn assign_op(op: AssignOp) -> &'static str {
    op.symbol()
}

thread_local! {
    /// Maps a lambda parameter name to its generated name (`l<n>_<name>`)
    /// while a lambda subtree is being lowered, so nested lambdas never
    /// collide in the plain-text output.
    static LAMBDA_SUBS: std::cell::RefCell<std::collections::HashMap<String, String>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    static LAMBDA_COUNTER: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub fn lower_expression(expression: &Expression) -> String {
    match expression {
        Expression::IntLiteral(value, _) => value.to_string(),
        Expression::FloatLiteral(value, _) => format_float(*value),
        Expression::StringLiteral(value, _) => js_string_literal(value),
        Expression::BoolLiteral(value, _) => value.to_string(),
        Expression::NullLiteral(_) => "null".into(),
        Expression::Identifier(identifier) => LAMBDA_SUBS.with(|subs| {
            subs.borrow()
                .get(&identifier.name)
                .cloned()
                .unwrap_or_else(|| identifier.name.clone())
        }),
        Expression::Event(_) => "event".into(),
        Expression::ArrayLiteral(items, _) => format!(
            "[{}]",
            items
                .iter()
                .map(lower_expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::ObjectLiteral(entries, _) => format!(
            "{{{}}}",
            entries
                .iter()
                .map(|entry| {
                    let key = match &entry.key {
                        platipus_language::ast::PropertyKey::Named(ident) => ident.name.clone(),
                        platipus_language::ast::PropertyKey::String(text, _) => text.clone(),
                        platipus_language::ast::PropertyKey::Index(index, _) => index.to_string(),
                    };
                    format!("{key}: {}", lower_expression(&entry.value))
                })
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::Unary { op, operand, .. } => {
            let operand = operand_at_least(operand, Prec::Postfix);
            format!("{}{}", unary_op(*op), operand)
        }
        Expression::Binary {
            op, left, right, ..
        } => {
            let precedence = binary_precedence(*op);
            let left = operand_at_least(left, precedence);
            let right = operand_at_least(right, precedence.next());
            format!("{} {} {}", left, binary_op(*op), right)
        }
        Expression::Logical {
            op, left, right, ..
        } => {
            let precedence = logical_precedence(*op);
            let left = operand_at_least(left, precedence);
            let right = operand_at_least(right, precedence.next());
            format!("{} {} {}", left, logical_op(*op), right)
        }
        Expression::Assign {
            op, target, value, ..
        } => format!(
            "{} {}= {}",
            operand_at_least(target, Prec::Assign),
            assign_op(*op).trim_end_matches('='),
            lower_expression(value)
        ),
        Expression::Call {
            callee, arguments, ..
        } => format!(
            "{}({})",
            operand_at_least(callee, Prec::Postfix),
            arguments
                .iter()
                .map(lower_expression)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        Expression::Member {
            object, property, ..
        } => {
            format!(
                "{}.{}",
                operand_at_least(object, Prec::Postfix),
                property.name
            )
        }
        Expression::Index { object, index, .. } => format!(
            "{}[{}]",
            operand_at_least(object, Prec::Postfix),
            lower_expression(index)
        ),
        Expression::Await { operand, .. } => {
            format!("await {}", operand_at_least(operand, Prec::Postfix))
        }
        Expression::Lambda {
            parameters, body, ..
        } => {
            let n = LAMBDA_COUNTER.with(|c| {
                let value = c.get();
                c.set(value + 1);
                value
            });
            let renames: Vec<(String, String)> = parameters
                .iter()
                .map(|parameter| (parameter.name.clone(), format!("l{}_{}", n, parameter.name)))
                .collect();
            let previous: Vec<(String, Option<String>)> = LAMBDA_SUBS.with(|subs| {
                let mut map = subs.borrow_mut();
                renames
                    .iter()
                    .map(|(name, renamed)| (name.clone(), map.insert(name.clone(), renamed.clone())))
                    .collect()
            });
            let params = renames
                .iter()
                .map(|(_, renamed)| renamed.clone())
                .collect::<Vec<_>>()
                .join(", ");
            let text = match body {
                platipus_language::ast::LambdaBody::Expression(expression) => lower_expression(expression),
                platipus_language::ast::LambdaBody::Block(block) => {
                    let statements = block
                        .statements
                        .iter()
                        .map(lambda_statement_text)
                        .collect::<Vec<_>>()
                        .join("; ");
                    format!("{{ {statements} }}")
                }
            };
            LAMBDA_SUBS.with(|subs| {
                let mut map = subs.borrow_mut();
                for (name, old) in previous {
                    match old {
                        Some(old) => {
                            map.insert(name, old);
                        }
                        None => {
                            map.remove(&name);
                        }
                    }
                }
            });
            format!("({}) => {}", params, text)
        }
    }
}

/// Renders a statement that may live inside a lambda block body as plain
/// JavaScript text. Local names stay raw here: the semantic checker bans
/// shadowing, so a raw name inside lambda text can never be rewritten into a
/// wrong access by the surrounding web-codegen scope pass.
fn lambda_statement_text(statement: &Statement) -> String {
    match statement {
        Statement::Assignment(assignment) => format!(
            "{} {} {}",
            lower_expression(&assignment.target),
            assign_op(assignment.op),
            lower_expression(&assignment.value)
        ),
        Statement::Expression(expression) => lower_expression(expression),
        Statement::If(if_statement) => lambda_if_text(if_statement),
        Statement::For(for_statement) => match &for_statement.iterable {
            ForIterable::Value(iterable) => format!(
                "for (const {} of plt.iter({})) {{ {} }}",
                for_statement.binding.name,
                lower_expression(iterable),
                lambda_block_text(&for_statement.body)
            ),
            ForIterable::Range {
                start,
                end,
                inclusive,
                ..
            } => format!(
                "for (let {} = {}; {} {} {}; {}++) {{ {} }}",
                for_statement.binding.name,
                lower_expression(start),
                for_statement.binding.name,
                if *inclusive { "<=" } else { "<" },
                lower_expression(end),
                for_statement.binding.name,
                lambda_block_text(&for_statement.body)
            ),
        },
        Statement::Let(let_statement) => format!(
            "let {} = {}",
            let_statement.name.name,
            lower_expression(&let_statement.initializer)
        ),
        Statement::While(while_statement) => format!(
            "while ({}) {{ {} }}",
            lower_expression(&while_statement.condition),
            lambda_block_text(&while_statement.body)
        ),
        Statement::Return(return_statement) => match &return_statement.value {
            Some(value) => format!("return {}", lower_expression(value)),
            None => "return".to_string(),
        },
        Statement::Break(_) => "break".to_string(),
        Statement::Continue(_) => "continue".to_string(),
        Statement::Try(try_statement) => format!(
            "try {{ {} }} catch ({}) {{ {} }}",
            lambda_block_text(&try_statement.body),
            try_statement
                .binding
                .as_ref()
                .map(|binding| binding.name.clone())
                .unwrap_or_else(|| "e".to_string()),
            lambda_block_text(&try_statement.handler)
        ),
        Statement::Block(block) => format!("{{ {} }}", lambda_block_text(block)),
        Statement::Empty(_) => String::new(),
        Statement::State(_) | Statement::Derived(_) | Statement::Function(_) | Statement::Handler(_) => {
            "/* declaration not allowed inside a lambda body */".to_string()
        }
        Statement::Element(_) => "/* element not allowed inside a lambda body */".to_string(),
        Statement::Emit(emit) => format!("/* emit {} */", emit.name.name),
        Statement::Import(_) | Statement::Test(_) => "/* not allowed inside a lambda body */".to_string(),
    }
}

fn lambda_block_text(block: &platipus_language::ast::Block) -> String {
    block
        .statements
        .iter()
        .map(lambda_statement_text)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

fn lambda_if_text(if_statement: &platipus_language::ast::IfStatement) -> String {
    match &if_statement.else_branch {
        None => format!(
            "if ({}) {{ {} }}",
            lower_expression(&if_statement.condition),
            lambda_block_text(&if_statement.then_branch)
        ),
        Some(branch) => match branch.as_ref() {
            platipus_language::ast::ElseBranch::Block(block) => format!(
                "if ({}) {{ {} }} else {{ {} }}",
                lower_expression(&if_statement.condition),
                lambda_block_text(&if_statement.then_branch),
                lambda_block_text(block)
            ),
            platipus_language::ast::ElseBranch::If(nested) => format!(
                "if ({}) {{ {} }} else {}",
                lower_expression(&if_statement.condition),
                lambda_block_text(&if_statement.then_branch),
                lambda_if_text(nested)
            ),
        },
    }
}

/// Binding strength of an expression, mirroring the precedence-climbing order
/// the parser uses so the target can re-parse the lowered text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Assign,
    Or,
    And,
    Equality,
    Comparison,
    BitOr,
    BitXor,
    BitAnd,
    Shift,
    Sum,
    Product,
    Unary,
    Postfix,
    Atomic,
}

impl Prec {
    /// The precedence one level tighter than `self`, used for the right operand
    /// of a left-associative operator.
    const fn next(self) -> Self {
        match self {
            Prec::Assign => Prec::Assign,
            Prec::Or => Prec::And,
            Prec::And => Prec::Equality,
            Prec::Equality => Prec::Comparison,
            Prec::Comparison => Prec::BitOr,
            Prec::BitOr => Prec::BitXor,
            Prec::BitXor => Prec::BitAnd,
            Prec::BitAnd => Prec::Shift,
            Prec::Shift => Prec::Sum,
            Prec::Sum => Prec::Product,
            Prec::Product => Prec::Unary,
            Prec::Unary => Prec::Postfix,
            Prec::Postfix => Prec::Atomic,
            Prec::Atomic => Prec::Atomic,
        }
    }
}

fn precedence_of(expression: &Expression) -> Prec {
    match expression {
        Expression::IntLiteral(..)
        | Expression::FloatLiteral(..)
        | Expression::StringLiteral(..)
        | Expression::BoolLiteral(..)
        | Expression::NullLiteral(_)
        | Expression::Identifier(_)
        | Expression::Event(_)
        | Expression::ArrayLiteral(..)
        | Expression::ObjectLiteral(..) => Prec::Atomic,
        Expression::Call { .. } | Expression::Member { .. } | Expression::Index { .. } => {
            Prec::Postfix
        }
        Expression::Await { .. } => Prec::Unary,
        Expression::Unary { .. } => Prec::Unary,
        Expression::Binary { op, .. } => binary_precedence(*op),
        Expression::Logical { op, .. } => logical_precedence(*op),
        Expression::Assign { .. } => Prec::Assign,
        Expression::Lambda { .. } => Prec::Atomic,
    }
}

/// Lowers `expression` and wraps it in parentheses when it binds more loosely
/// than the position it is being placed in.
///
/// A bitwise expression is wrapped even when it does not need to be, because
/// this text is re-parsed by JavaScript and JavaScript does not agree with
/// Platipus here: in Platipus `==` binds looser than `&`, while in JavaScript
/// `==` binds tighter. `n & k == 1` means `(n & k) == 1` in the source language
/// and would mean `n & (k == 1)` if the parentheses were left to the target's
/// precedence table. The emitted text is therefore unambiguous by construction
/// rather than by two tables happening to agree.
fn operand_at_least(expression: &Expression, minimum: Prec) -> String {
    let text = lower_expression(expression);
    if is_bitwise(expression) || precedence_of(expression) < minimum {
        format!("({text})")
    } else {
        text
    }
}

/// Whether `expression` is one of the five binary bitwise operators.
fn is_bitwise(expression: &Expression) -> bool {
    matches!(
        expression,
        Expression::Binary { op, .. }
            if matches!(
                op,
                BinaryOp::And | BinaryOp::Or | BinaryOp::Xor | BinaryOp::Shl | BinaryOp::Shr
            )
    )
}

fn binary_precedence(op: BinaryOp) -> Prec {
    match op {
        BinaryOp::Eq | BinaryOp::Ne => Prec::Equality,
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => Prec::Comparison,
        BinaryOp::Or => Prec::BitOr,
        BinaryOp::Xor => Prec::BitXor,
        BinaryOp::And => Prec::BitAnd,
        BinaryOp::Shl | BinaryOp::Shr => Prec::Shift,
        BinaryOp::Add | BinaryOp::Sub => Prec::Sum,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => Prec::Product,
    }
}

fn logical_precedence(op: platipus_language::ast::LogicalOp) -> Prec {
    match op {
        platipus_language::ast::LogicalOp::And => Prec::And,
        platipus_language::ast::LogicalOp::Or => Prec::Or,
    }
}

fn lower_style_value(expression: &Expression) -> String {
    match expression {
        Expression::StringLiteral(value, _) => value.clone(),
        Expression::ArrayLiteral(items, _) => items
            .iter()
            .map(lower_style_value)
            .collect::<Vec<_>>()
            .join(" "),
        other => lower_expression(other),
    }
}

fn format_float(value: f64) -> String {
    if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.1}")
    } else {
        value.to_string()
    }
}

/// JS/JSON-compatible string literal escaping: Rust's `{:?}` cannot be used
/// because it escapes astral characters as `\u{...}` rather than surrogate
/// pairs.
fn js_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c if (c as u32) < 0x7f => out.push(c),
            c => {
                let code = c as u32;
                if code <= 0xFFFF {
                    out.push_str(&format!("\\u{:04x}", code));
                } else {
                    let hi = 0xD800 + ((code - 0x10000) >> 10);
                    let lo = 0xDC00 + ((code - 0x10000) & 0x3FF);
                    out.push_str(&format!("\\u{:04x}\\u{:04x}", hi, lo));
                }
            }
        }
    }
    out.push('"');
    out
}

fn unary_op(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "-",
        UnaryOp::Not => "!",
        UnaryOp::BitNot => "~",
    }
}

fn binary_op(op: BinaryOp) -> &'static str {
    op.symbol()
}

fn logical_op(op: platipus_language::ast::LogicalOp) -> &'static str {
    match op {
        platipus_language::ast::LogicalOp::And => "&&",
        platipus_language::ast::LogicalOp::Or => "||",
    }
}
