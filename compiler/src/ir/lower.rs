use crate::ast::{
    self, AppDecl, AssignOp, BinaryOp, ComponentDecl, ComponentItem, Element, EventCategory,
    Expression, Identifier, Program, Statement, UnaryOp,
};
use crate::ir::element::ElementItem as IrElementItem;
use crate::ir::statement::StatementKind;
use crate::ir::{
    IrApi, IrBinding, IrComponent, IrDerived, IrElement, IrElementBody, IrExpression, IrFunction,
    IrHandler, IrImport, IrInput, IrModule, IrProperty, IrResponsiveBlock, IrResponsiveEntry,
    IrRoute, IrState, IrStateKind, IrStatement, IrStyleBlock, IrStyleEntry, IrStyleState, IrTest,
    IrTestStep, IrTheme, IrThemeToken,
};
use crate::semantic::events;

pub struct Lowering<'a> {
    components: Vec<&'a str>,
}

impl<'a> Lowering<'a> {
    pub fn new(components: Vec<&'a str>) -> Self {
        Self { components }
    }

    pub fn from_program(program: &'a Program) -> Self {
        Self::new(
            program
                .components
                .iter()
                .map(|component| component.name.as_str())
                .collect(),
        )
    }

    pub fn is_component(&self, name: &str) -> bool {
        self.components.contains(&name)
    }
}

pub fn lower_program(program: &Program) -> Option<IrModule> {
    let app = program.app.as_ref()?;
    let lowering = Lowering::from_program(program);
    let component = lowering.lower_component_from_app(app);
    let components = std::iter::once(component)
        .chain(
            program
                .components
                .iter()
                .map(|component| lowering.lower_component(component)),
        )
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
                    crate::ast::TestStep::Action {
                        name,
                        argument,
                        span,
                    } => IrTestStep::Action {
                        name: name.as_str().to_string(),
                        argument: argument.as_ref().map(lower_expression),
                        span: *span,
                    },
                    crate::ast::TestStep::Expect { expression, span } => IrTestStep::Expect {
                        expression: lower_expression(expression),
                        span: *span,
                    },
                })
                .collect(),
            span: test.span,
        })
        .collect();
    Some(IrModule {
        name: app.name.as_str().to_string(),
        components,
        styles,
        themes,
        apis,
        imports,
        tests,
        span: app.span,
    })
}

fn lower_api(api: &crate::ast::ApiDecl) -> IrApi {
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
                    initializer: decl.initializer.as_ref().map(lower_reactive),
                    span: decl.span,
                }),
                ComponentItem::Derived(decl) => self.derived.push(IrDerived {
                    name: decl.name.as_str().to_string(),
                    type_name: decl
                        .type_annotation
                        .as_ref()
                        .map(|annotation| annotation.base_name().to_string()),
                    value: lower_expression(&decl.value),
                    dependencies: dependencies(&decl.value),
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

fn lower_input(input: &crate::ast::InputDecl) -> IrInput {
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

fn lower_parameter(parameter: &crate::ast::Parameter) -> IrInput {
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

fn lower_reactive(expression: &Expression) -> IrExpression {
    IrExpression {
        value: lower_expression(expression),
        dependencies: dependencies(expression),
        span: expression.span(),
    }
}

fn dependencies(expression: &Expression) -> Vec<String> {
    let mut found: Vec<Identifier> = Vec::new();
    crate::ast::collect_identifiers(expression, &mut found);
    let mut names: Vec<String> = found
        .into_iter()
        .map(|identifier| identifier.name)
        .collect();
    names.sort();
    names.dedup();
    names
}

impl<'a> Lowering<'a> {
    fn lower_element(&self, element: &Element) -> Option<IrElement> {
        let name = element.name.as_str();
        let kind = if self.is_component(name) {
            crate::ir::element::ElementKind::Component
        } else {
            crate::ir::element::ElementKind::Primitive
        };
        let mut properties: Vec<IrProperty> = element
            .arguments
            .iter()
            .map(|property| IrProperty {
                name: property.name.as_str().to_string(),
                value: lower_reactive(&property.value),
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
                        value: lower_reactive(&property.value),
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
                        value: lower_reactive(&binding.target),
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
            text: element.text.as_ref().map(lower_reactive),
            handlers,
            body,
            span: element.span,
        })
    }
}

fn lower_responsive(block: &crate::ast::ResponsiveBlock) -> IrResponsiveBlock {
    let entries = block
        .entries
        .iter()
        .map(|entry| {
            let breakpoint = entry.breakpoint.as_str().to_string();
            match &entry.kind {
                crate::ast::ResponsiveEntryKind::Property { name, value } => {
                    IrResponsiveEntry::Property {
                        breakpoint,
                        name: name.as_str().to_string(),
                        value: lower_style_value(value),
                        span: entry.span,
                    }
                }
                crate::ast::ResponsiveEntryKind::Named { name } => IrResponsiveEntry::Named {
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

fn lower_style_block(block: &crate::ast::StyleBlock) -> Option<IrStyleBlock> {
    let entries: Vec<IrStyleEntry> = block
        .entries
        .iter()
        .map(|entry| match entry {
            crate::ast::StyleEntry::Property { name, value, span } => IrStyleEntry {
                name: name.as_str().to_string(),
                value: lower_style_value(value),
                span: *span,
            },
            crate::ast::StyleEntry::Reference { name, span } => IrStyleEntry {
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
    fn lower_handler(&self, handler: &crate::ast::EventHandlerDecl) -> IrHandler {
        let name = handler.name.as_str();
        let category = events::category_of(name).unwrap_or(EventCategory::Custom);
        IrHandler {
            event: name.to_string(),
            category: crate::ir::IrEventCategory(category),
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
                        crate::ast::ElseBranch::Block(block) => block
                            .statements
                            .iter()
                            .map(|nested| self.lower_statement(nested))
                            .collect(),
                        crate::ast::ElseBranch::If(nested) => {
                            vec![self.lower_statement(&Statement::If(nested.clone()))]
                        }
                    }),
            },
            Statement::For(statement) => StatementKind::For {
                binding: statement.binding.as_str().to_string(),
                iterable: lower_expression(&statement.iterable),
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

pub fn lower_expression(expression: &Expression) -> String {
    match expression {
        Expression::IntLiteral(value, _) => value.to_string(),
        Expression::FloatLiteral(value, _) => format_float(*value),
        Expression::StringLiteral(value, _) => format!("{value:?}"),
        Expression::BoolLiteral(value, _) => value.to_string(),
        Expression::NullLiteral(_) => "null".into(),
        Expression::Identifier(identifier) => identifier.name.clone(),
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
                        crate::ast::PropertyKey::Named(ident) => ident.name.clone(),
                        crate::ast::PropertyKey::String(text, _) => text.clone(),
                        crate::ast::PropertyKey::Index(index, _) => index.to_string(),
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
            Prec::Comparison => Prec::Sum,
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
    }
}

/// Lowers `expression` and wraps it in parentheses when it binds more loosely
/// than the position it is being placed in.
fn operand_at_least(expression: &Expression, minimum: Prec) -> String {
    let text = lower_expression(expression);
    if precedence_of(expression) < minimum {
        format!("({text})")
    } else {
        text
    }
}

fn binary_precedence(op: BinaryOp) -> Prec {
    match op {
        BinaryOp::Eq | BinaryOp::Ne => Prec::Equality,
        BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => Prec::Comparison,
        BinaryOp::Add | BinaryOp::Sub => Prec::Sum,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem => Prec::Product,
    }
}

fn logical_precedence(op: crate::ast::LogicalOp) -> Prec {
    match op {
        crate::ast::LogicalOp::And => Prec::And,
        crate::ast::LogicalOp::Or => Prec::Or,
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

fn unary_op(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Negate => "-",
        UnaryOp::Not => "!",
    }
}

fn binary_op(op: BinaryOp) -> &'static str {
    op.symbol()
}

fn logical_op(op: crate::ast::LogicalOp) -> &'static str {
    match op {
        crate::ast::LogicalOp::And => "&&",
        crate::ast::LogicalOp::Or => "||",
    }
}
