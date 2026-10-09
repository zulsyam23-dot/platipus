use platipus_language::element::ElementRegistry;
use super::scope::{ScopeKind, ScopeStack, Symbol, SymbolKind};
use super::types::TypeRegistry;
use platipus_language::ast::{
    Block, ComponentDecl, ComponentItem, Element, ElementItem, EmitStatement, EventHandlerDecl,
    Expression, ForIterable, ForStatement, Identifier, IfStatement, LetStatement, Program,
    PropertyValue, Statement, TestStep, TypeExpr, WhileStatement,
    LambdaBody,
};
use platipus_diagnostics::{DiagnosticBag, Error, ErrorKind, Note, Span, Warning, WarningKind};

#[derive(Debug)]
pub struct SemanticChecker {
    diagnostics: DiagnosticBag,
    scopes: ScopeStack,
    elements: ElementRegistry,
    types: TypeRegistry,
    component_emits: Vec<(String, Vec<Identifier>)>,
    /// True while checking a `fn` body or an event handler: `let` and `while`
    /// are statements of executable code, never of a component template.
    in_function: bool,
    /// `fn` declarations by name, with their parameter signatures, so calls
    /// can be arity-checked and argument types checked against annotations.
    fn_signatures: std::collections::HashMap<String, Vec<FnSignature>>,
    /// True while checking a top-level `fn` body, which must be pure.
    in_pure_fn: bool,
}

/// A declared function's parameter signature: how many arguments it needs
/// (`min..=max`, minding parameters with defaults) and each parameter's
/// optional type annotation.
#[derive(Debug, Clone)]
struct FnSignature {
    min: usize,
    max: usize,
    params: Vec<Option<TypeExpr>>,
    span: Span,
}

impl Default for SemanticChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticChecker {
    pub fn new() -> Self {
        let mut scopes = ScopeStack::new();
        scopes.push(ScopeKind::Global);
        Self {
            diagnostics: DiagnosticBag::new(),
            scopes,
            elements: ElementRegistry::new(),
            types: TypeRegistry::new(),
            component_emits: Vec::new(),
            in_function: false,
            fn_signatures: std::collections::HashMap::new(),
            in_pure_fn: false,
        }
    }

    pub fn check(&mut self, program: &Program) -> DiagnosticBag {
        self.declare_components(program);
        self.declare_rust_exports(program);
        self.declare_top_level_functions(program);
        self.check_unique_declarations(program);
        self.check_components(program);
        self.check_app(program);
        self.check_apis(program);
        self.check_imports(program);
        self.check_tests(program);
        self.check_top_level_functions(program);
        self.diagnostics.clone()
    }

    /// Top-level `fn` declarations are hoisted: every name is visible in the
    /// global scope before any body is checked, so mutual recursion checks.
    fn declare_top_level_functions(&mut self, program: &Program) {
        for decl in &program.functions {
            self.declare(&decl.name, SymbolKind::Function);
            self.register_fn_signature(decl);
        }
    }

    /// Top-level `fn` bodies are checked under the purity contract: no
    /// component state, no emission, no elements, and only pure builtins.
    fn check_top_level_functions(&mut self, program: &Program) {
        for decl in &program.functions {
            let previous = std::mem::replace(&mut self.in_pure_fn, true);
            self.scopes.push(ScopeKind::Function);
            for parameter in &decl.parameters {
                self.declare(&parameter.name, SymbolKind::Parameter);
                self.check_type(&parameter.type_annotation);
                if let Some(default) = &parameter.default {
                    self.check_expression(default);
                    self.check_annotation_matches(&parameter.type_annotation, default);
                }
            }
            self.check_type(&decl.return_type);
            self.check_body(&decl.body);
            self.scopes.pop();
            self.in_pure_fn = previous;
        }
    }

    /// Rust functions exported with `#[export]` are ordinary global function
    /// symbols, so calls to them type-check like any other function call.
    fn declare_rust_exports(&mut self, program: &Program) {
        match platipus_language::rust_extract::extract_rust(program) {
            Ok(Some(extraction)) => {
                for export in &extraction.exports {
                    let _ = self.scopes.define(Symbol {
                        name: export.name.clone(),
                        kind: SymbolKind::Function,
                        span: Span::new(export.plt_offset as u32, export.plt_offset as u32),
                    });
                }
            }
            Ok(None) => {}
            Err(bag) => self.diagnostics.extend(bag),
        }
    }

    fn declare_components(&mut self, program: &Program) {
        for component in &program.components {
            self.component_emits.push((
                component.name.name.clone(),
                component.emits().into_iter().cloned().collect(),
            ));
            let _ = self.scopes.define(Symbol {
                name: component.name.name.clone(),
                kind: SymbolKind::Component,
                span: component.name.span,
            });
        }
    }

    fn check_unique_declarations(&mut self, program: &Program) {
        for component in &program.components {
            self.report_duplicates(
                component.name.as_str(),
                &declared_names(&component.inputs, &component.body),
            );
        }
        if let Some(app) = &program.app {
            self.report_duplicates(app.name.as_str(), &declared_names(&[], &app.body));
        }
    }

    fn report_duplicates(&mut self, owner: &str, names: &[(&str, Span)]) {
        let mut seen: Vec<(&str, Span)> = Vec::new();
        for (name, span) in names {
            match seen.iter().find(|(existing, _)| existing == name) {
                Some((_, first)) => {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "duplicate-declaration",
                            format!("`{name}` is already declared in `{owner}`"),
                        )
                        .with_span(*span)
                        .with_note(Note::at(
                            format!("first declared here at byte {}", first.start),
                            *first,
                        )),
                    );
                }
                None => seen.push((name, *span)),
            }
        }
    }

    fn check_components(&mut self, program: &Program) {
        let components = program.components.clone();
        for component in &components {
            self.check_component(component);
        }
    }

    fn check_component(&mut self, component: &ComponentDecl) {
        self.scopes.push(ScopeKind::Component);
        for input in &component.inputs {
            self.declare(&input.name, SymbolKind::Input);
            self.check_type(&input.type_annotation);
            if let Some(default) = &input.default {
                self.check_expression(default);
            }
        }
        let emits = component.emits();
        self.check_items(&component.body, &emits);
        self.scopes.pop();
    }

    fn check_app(&mut self, program: &Program) {
        let Some(app) = &program.app else {
            return;
        };
        self.scopes.push(ScopeKind::App);
        for input in &app.inputs {
            self.declare(&input.name, SymbolKind::Input);
            self.check_type(&input.type_annotation);
            if let Some(default) = &input.default {
                self.check_expression(default);
            }
        }
        self.check_items(&app.body, &app.emits());
        let roots = app
            .body
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    ComponentItem::Child(_) | ComponentItem::Stmt(Statement::Element(_))
                )
            })
            .count();
        if roots == 0 {
            self.diagnostics.error(
                Error::new(
                    ErrorKind::Semantic,
                    "empty-app",
                    format!("`{}` declares no root element", app.name.as_str()),
                )
                .with_span(app.name.span)
                .with_help("an app needs at least one root element to render"),
            );
        } else if roots > 1 {
            for item in &app.body {
                if matches!(
                    item,
                    ComponentItem::Child(_) | ComponentItem::Stmt(Statement::Element(_))
                ) {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "multiple-roots",
                            "an app must declare exactly one root element",
                        )
                        .with_span(item.span()),
                    );
                }
            }
        }
        self.scopes.pop();
    }

    fn check_apis(&mut self, program: &Program) {
        for api in &program.apis {
            let mut seen: Vec<(&str, Span)> = Vec::new();
            for route in &api.routes {
                if !route.path.starts_with('/') {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "invalid-route-path",
                            format!("route path `{}` must start with `/`", route.path),
                        )
                        .with_span(route.span),
                    );
                }
                let key = route.name.as_str();
                if seen.iter().any(|(name, _)| *name == key) {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "duplicate-route",
                            format!("route `{key}` is declared twice in `{}`", api.name.as_str()),
                        )
                        .with_span(route.name.span),
                    );
                } else {
                    seen.push((key, route.span));
                }
            }
        }
    }

    fn check_imports(&mut self, program: &Program) {
        for import in &program.imports {
            if !import.path.ends_with(".plt") {
                self.diagnostics.warning(
                    Warning::new(
                        WarningKind::Convention,
                        format!("import `{}` does not use the `.plt` extension", import.path),
                    )
                    .with_span(import.span),
                );
            }
        }
    }

    fn check_tests(&mut self, program: &Program) {
        for test in &program.tests {
            for step in &test.steps {
                let TestStep::Action { name, span, .. } = step else {
                    continue;
                };
                if platipus_testing::TEST_ACTIONS.contains(&name.as_str()) {
                    continue;
                }
                let known = platipus_testing::TEST_ACTIONS.join("`, `");
                self.diagnostics.error(
                    Error::new(
                        ErrorKind::Semantic,
                        "unknown-test-action",
                        format!("`{}` is not a test action", name.as_str()),
                    )
                    .with_help(format!("a test step can be `{known}`"))
                    .with_span(*span),
                );
            }
        }
    }

    fn check_items(&mut self, items: &[ComponentItem], emits: &[&Identifier]) {
        for item in items {
            self.check_item(item, emits);
        }
    }

    fn check_item(&mut self, item: &ComponentItem, emits: &[&Identifier]) {
        match item {
            ComponentItem::State(decl) => {
                self.declare(&decl.name, SymbolKind::State);
                if decl.initializer.is_none() && !decl.kind.allows_declaration_only() {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "missing-initializer",
                            format!(
                                "`{}` must be initialized unless declared as {}",
                                decl.name.as_str(),
                                decl.kind.as_str()
                            ),
                        )
                        .with_span(decl.name.span)
                        .with_help(format!(
                            "write `{} {} = ...`",
                            decl.kind.as_str(),
                            decl.name.as_str()
                        )),
                    );
                }
                self.check_type(&decl.type_annotation);
                if let Some(initializer) = &decl.initializer {
                    self.check_expression(initializer);
                    self.check_annotation_matches(&decl.type_annotation, initializer);
                }
            }
            ComponentItem::Derived(decl) => {
                self.declare(&decl.name, SymbolKind::Derived);
                self.check_type(&decl.type_annotation);
                self.check_expression(&decl.value);
                self.check_annotation_matches(&decl.type_annotation, &decl.value);
            }
            ComponentItem::Function(decl) => {
                self.declare(&decl.name, SymbolKind::Function);
                self.register_fn_signature(decl);
                self.scopes.push(ScopeKind::Function);
                for parameter in &decl.parameters {
                    self.declare(&parameter.name, SymbolKind::Parameter);
                    self.check_type(&parameter.type_annotation);
                    if let Some(default) = &parameter.default {
                        self.check_expression(default);
                        self.check_annotation_matches(&parameter.type_annotation, default);
                    }
                }
                self.check_type(&decl.return_type);
                self.check_body(&decl.body);
                self.scopes.pop();
            }
            ComponentItem::Handler(handler) => {
                self.validate_handler(handler, emits);
                self.scopes.push(ScopeKind::Function);
                self.check_body(&handler.body);
                self.scopes.pop();
            }
            ComponentItem::Child(element) => self.check_element(element),
            ComponentItem::Stmt(statement) => self.check_statement(statement),
            ComponentItem::Style(_) => {}
            ComponentItem::Import(_) | ComponentItem::Test(_) => {}
        }
    }

    fn validate_handler(&mut self, handler: &EventHandlerDecl, emits: &[&Identifier]) {
        let name = handler.name.as_str();
        if platipus_language::events::is_known(name) {
            return;
        }
        if emits.iter().any(|emit| emit.as_str() == name) {
            return;
        }
        let mut error = Error::new(
            ErrorKind::Semantic,
            "unknown-event",
            format!("`{name}` is not a known event"),
        )
        .with_span(handler.name.span);
        if let Some(symbol) = self
            .scopes
            .iter()
            .find(|scope| scope.kind == ScopeKind::Component)
            .and_then(|scope| scope.lookup_local(name))
        {
            error = error.with_note(Note::at(
                format!("`{name}` is a declared name, not an event"),
                symbol.span,
            ));
        }
        self.diagnostics.error(
            error.with_help("custom events must be emitted by the component that declares them"),
        );
    }

    fn check_block(&mut self, block: &Block) {
        self.scopes.push(ScopeKind::Block);
        for statement in &block.statements {
            self.check_statement(statement);
        }
        self.scopes.pop();
    }

    /// A block that is executable code — a `fn` body or an event handler —
    /// where `let` and `while` are legal.
    fn check_body(&mut self, body: &Block) {
        let previous = std::mem::replace(&mut self.in_function, true);
        self.check_block(body);
        self.in_function = previous;
    }

    fn check_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::State(decl) => {
                if self.in_pure_fn {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "pure-fn-uses-state",
                            format!("top-level `fn` cannot declare `state` (`{}`)", decl.name.as_str()),
                        )
                        .with_span(decl.span)
                        .with_help("state belongs to a component; top-level `fn` works on arguments only"),
                    );
                }
                self.declare(&decl.name, SymbolKind::State);
                if let Some(initializer) = &decl.initializer {
                    self.check_expression(initializer);
                }
            }
            Statement::Derived(decl) => {
                if self.in_pure_fn {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "pure-fn-uses-state",
                            format!("top-level `fn` cannot declare `derived` (`{}`)", decl.name.as_str()),
                        )
                        .with_span(decl.span)
                        .with_help("derived values belong to a component; top-level `fn` works on arguments only"),
                    );
                }
                self.declare(&decl.name, SymbolKind::Derived);
                self.check_expression(&decl.value);
            }
            Statement::Function(decl) => {
                self.declare(&decl.name, SymbolKind::Function);
                self.register_fn_signature(decl);
                self.scopes.push(ScopeKind::Function);
                for parameter in &decl.parameters {
                    self.declare(&parameter.name, SymbolKind::Parameter);
                }
                self.check_body(&decl.body);
                self.scopes.pop();
            }
            Statement::Element(element) => self.check_element(element),
            Statement::Handler(handler) => {
                self.validate_handler(handler, &[]);
                self.scopes.push(ScopeKind::Function);
                self.check_body(&handler.body);
                self.scopes.pop();
            }
            Statement::Emit(emit) => {
                if self.in_pure_fn {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "impure-call-in-pure-fn",
                            "`emit` cannot be used inside a top-level `fn`".to_string(),
                        )
                        .with_span(emit.name.span)
                        .with_help("top-level `fn` bodies must be pure; move side effects into a component handler"),
                    );
                }
                self.check_emit(emit);
            }
            Statement::Assignment(assignment) => self.check_assignment(assignment),
            Statement::Expression(expression) => self.check_expression(expression),
            Statement::If(statement) => self.check_if(statement),
            Statement::For(statement) => self.check_for(statement),
            Statement::Let(statement) => self.check_let(statement),
            Statement::While(statement) => self.check_while(statement),
            Statement::Return(statement) => {
                if let Some(value) = &statement.value {
                    self.check_expression(value);
                }
            }
            Statement::Break(span) | Statement::Continue(span) => {
                if !self
                    .scopes
                    .iter()
                    .any(|scope| scope.kind == ScopeKind::Loop)
                {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "break-outside-loop",
                            "`break` and `continue` are only valid inside a loop",
                        )
                        .with_span(*span),
                    );
                }
            }
            Statement::Try(statement) => {
                self.scopes.push(ScopeKind::Block);
                if let Some(binding) = &statement.binding {
                    self.declare(binding, SymbolKind::Variable);
                }
                self.check_block(&statement.body);
                self.scopes.pop();
                self.check_block(&statement.handler);
            }
            Statement::Block(block) => self.check_block(block),
            Statement::Import(_) | Statement::Test(_) | Statement::Empty(_) => {}
        }
    }

    fn check_emit(&mut self, emit: &EmitStatement) {
        let name = emit.name.as_str();
        if platipus_language::events::is_known(name) {
            self.diagnostics.error(
                Error::new(
                    ErrorKind::Semantic,
                    "reserved-event-name",
                    format!("`{name}` is a built-in event and cannot be emitted"),
                )
                .with_span(emit.name.span),
            );
        }
        for argument in &emit.arguments {
            self.check_expression(argument);
        }
    }

    fn check_assignment(&mut self, assignment: &platipus_language::ast::Assignment) {
        self.check_expression(&assignment.value);
        if let Expression::Identifier(identifier) = &assignment.target {
            match self
                .scopes
                .lookup(&identifier.name)
                .map(|symbol| (symbol.kind, symbol.span))
            {
                None => self.report_undefined(identifier),
                Some((SymbolKind::Derived, span)) => {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "assign-to-derived",
                            format!("`{}` is derived and cannot be assigned to", identifier.name),
                        )
                        .with_span(span)
                        .with_help("derived values are computed from other values"),
                    );
                }
                Some((SymbolKind::Input, span)) => {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "assign-to-input",
                            format!(
                                "`{}` is an input and cannot be assigned to",
                                identifier.name
                            ),
                        )
                        .with_span(span)
                        .with_help("inputs are provided by the parent element"),
                    );
                }
                Some((SymbolKind::Parameter, span)) => {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "assign-to-parameter",
                            format!(
                                "`{}` is a parameter and cannot be assigned to",
                                identifier.name
                            ),
                        )
                        .with_span(span)
                        .with_help(format!(
                            "copy it first with `let copy = {}`",
                            identifier.name
                        )),
                    );
                }
                Some((SymbolKind::Variable, _)) => {}
                Some((_, _)) => {}
            }
            return;
        }
        self.check_expression(&assignment.target);
    }

    /// Warns when the condition's static type is clearly not `Bool`;
    /// unknown types stay silent so truthiness-dependent code keeps working.
    fn check_condition(&mut self, condition: &platipus_language::ast::Expression) {
        let Some(ty) = self.infer_expr_type(condition) else {
            return;
        };
        if ty != "Bool" {
            self.diagnostics.warning(
                Warning::new(
                    WarningKind::NonBoolCondition,
                    format!("condition has type `{ty}`, not `Bool`"),
                )
                .with_span(condition.span())
                .with_help("compare explicitly, e.g. `x > 0`"),
            );
        }
    }

    fn check_if(&mut self, statement: &IfStatement) {
        self.check_expression(&statement.condition);
        self.check_condition(&statement.condition);
        self.check_block(&statement.then_branch);
        match statement.else_branch.as_deref() {
            Some(platipus_language::ast::ElseBranch::Block(block)) => self.check_block(block),
            Some(platipus_language::ast::ElseBranch::If(nested)) => self.check_if(nested),
            None => {}
        }
    }

    fn check_for(&mut self, statement: &ForStatement) {
        match &statement.iterable {
            ForIterable::Value(iterable) => {
                if matches!(self.infer_expr_type(iterable), Some("Int" | "Float")) {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "for-over-number",
                            "a `for` loop cannot iterate over a single number",
                        )
                        .with_span(iterable.span())
                        .with_help("use `for i in start..end { ... }` to count"),
                    );
                }
                self.check_expression(iterable);
            }
            ForIterable::Range { start, end, .. } => {
                self.check_expression(start);
                self.check_expression(end);
            }
        }
        self.scopes.push(ScopeKind::Loop);
        self.declare(&statement.binding, SymbolKind::Variable);
        self.check_block(&statement.body);
        self.scopes.pop();
    }

    fn check_let(&mut self, statement: &LetStatement) {
        if !self.in_function {
            self.diagnostics.error(
                Error::new(
                    ErrorKind::Semantic,
                    "statement-outside-function",
                    "`let` is only allowed inside a `fn` or an event handler",
                )
                .with_span(statement.span)
                .with_help("component bodies are templates; move the `let` into a handler"),
            );
        }
        self.check_expression(&statement.initializer);
        self.check_type(&statement.annotation);
        self.check_annotation_matches(&statement.annotation, &statement.initializer);
        self.report_shadow(&statement.name);
        self.declare(&statement.name, SymbolKind::Variable);
    }

    fn check_while(&mut self, statement: &WhileStatement) {
        if !self.in_function {
            self.diagnostics.error(
                Error::new(
                    ErrorKind::Semantic,
                    "statement-outside-function",
                    "`while` is only allowed inside a `fn` or an event handler",
                )
                .with_span(statement.span)
                .with_help("component bodies are templates; move the `while` into a handler"),
            );
        }
        self.check_expression(&statement.condition);
        self.check_condition(&statement.condition);
        self.scopes.push(ScopeKind::Loop);
        self.check_block(&statement.body);
        self.scopes.pop();
    }

    /// `let` may not hide a name that is already in scope; a duplicate in the
    /// very same scope is left to `declare`, which reports it precisely.
    fn report_shadow(&mut self, name: &Identifier) {
        let shadowed_here = self
            .scopes
            .current()
            .is_some_and(|scope| scope.lookup_local(&name.name).is_some());
        if shadowed_here {
            return;
        }
        let Some(symbol) = self.scopes.lookup(&name.name) else {
            return;
        };
        if !matches!(
            symbol.kind,
            SymbolKind::State
                | SymbolKind::Derived
                | SymbolKind::Input
                | SymbolKind::Parameter
                | SymbolKind::Variable
        ) {
            return;
        }
        let span = symbol.span;
        self.diagnostics.error(
            Error::new(
                ErrorKind::Semantic,
                "shadow-local",
                format!("`{}` shadows a name that is already in scope", name.name),
            )
            .with_span(name.span)
            .with_note(Note::at("the other declaration is here", span))
            .with_help("pick another name for the `let`"),
        );
    }

    fn emits_of(&self, component: &str) -> Vec<Identifier> {
        self.component_emits
            .iter()
            .find(|(name, _)| name == component)
            .map(|(_, emits)| emits.clone())
            .unwrap_or_default()
    }

    fn check_element(&mut self, element: &Element) {
        let name = element.name.as_str();
        self.validate_element_name(element);
        for argument in &element.arguments {
            self.check_expression(&argument.value);
        }
        if let Some(text) = &element.text {
            if self.elements.contains(name) && !self.elements.accepts_text(name) {
                self.diagnostics.error(
                    Error::new(
                        ErrorKind::Semantic,
                        "text-not-allowed",
                        format!("`{name}` cannot contain text content"),
                    )
                    .with_span(text.span()),
                );
            }
            self.check_expression(text);
        }
        if let Some(body) = &element.body {
            let child = body.items.iter().find_map(|item| match item {
                ElementItem::Child(child) => Some(child),
                _ => None,
            });
            if let Some(child) = child {
                if self.elements.contains(name) && !self.elements.accepts_children(name) {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "children-not-allowed",
                            format!("`{name}` cannot contain child elements"),
                        )
                        .with_span(child.span),
                    );
                }
            }
            self.check_properties(name, &body.properties());
            let owned = self.emits_of(name);
            let owned_refs: Vec<&Identifier> = owned.iter().collect();
            for item in &body.items {
                self.check_element_item(item, &owned_refs);
            }
        }
    }

    fn validate_element_name(&mut self, element: &Element) {
        let name = element.name.as_str();
        if self.elements.contains(name) {
            return;
        }
        if self
            .scopes
            .lookup(name)
            .is_some_and(|symbol| symbol.kind == SymbolKind::Component)
        {
            return;
        }
        let suggestions = self.elements.suggestions(name);
        let mut error = Error::new(
            ErrorKind::Semantic,
            "unknown-element",
            format!("`{name}` is neither a primitive element nor a declared component"),
        )
        .with_span(element.name.span)
        .with_help("element and component names start with an uppercase letter");
        if !suggestions.is_empty() {
            let names = suggestions
                .iter()
                .map(|candidate| format!("`{candidate}`"))
                .collect::<Vec<_>>()
                .join(", ");
            error = error.with_note(Note::new(format!("did you mean {names}?")));
        }
        self.diagnostics.error(error);
    }

    fn check_properties(&mut self, element_name: &str, properties: &[&PropertyValue]) {
        if !self.elements.contains(element_name) {
            return;
        }
        let provided: Vec<String> = properties
            .iter()
            .map(|property| property.name.as_str().to_string())
            .collect();
        for property in properties {
            if self
                .elements
                .unknown_property(element_name, property.name.as_str())
            {
                self.diagnostics.error(
                    Error::new(
                        ErrorKind::Semantic,
                        "unknown-property",
                        format!(
                            "`{element_name}` has no property `{}`",
                            property.name.as_str()
                        ),
                    )
                    .with_span(property.name.span),
                );
            }
            self.check_expression(&property.value);
        }
        for missing in self.elements.missing_required(element_name, &provided) {
            self.diagnostics.error(
                Error::new(
                    ErrorKind::Semantic,
                    "missing-property",
                    format!("`{element_name}` requires `{missing}`"),
                )
                .with_note(Note::new(format!(
                    "add `{missing}: ...` inside the element body"
                ))),
            );
        }
    }

    fn check_element_item(&mut self, item: &ElementItem, owned: &[&Identifier]) {
        match item {
            ElementItem::Child(element) => self.check_element(element),
            ElementItem::Property(property) => self.check_expression(&property.value),
            ElementItem::Handler(handler) => {
                self.validate_handler(handler, owned);
                self.scopes.push(ScopeKind::Function);
                self.check_body(&handler.body);
                self.scopes.pop();
            }
            ElementItem::Binding(binding) => self.check_expression(&binding.target),
            ElementItem::Style(_) | ElementItem::Responsive(_) => {}
            ElementItem::Stmt(statement) => self.check_statement(statement),
        }
    }

    /// Best-effort type of an expression, used to check explicit annotations.
    /// Returns `None` whenever the type is not statically known (identifiers,
    /// calls, holes) — an unknown type must never be rejected.
    fn infer_expr_type(&self, expression: &platipus_language::ast::Expression) -> Option<&'static str> {
        use platipus_language::ast::{BinaryOp, Expression};
        match expression {
            Expression::IntLiteral(..) => Some("Int"),
            Expression::FloatLiteral(..) => Some("Float"),
            Expression::StringLiteral(..) => Some("String"),
            Expression::BoolLiteral(..) => Some("Bool"),
            Expression::Unary { operand, op, .. } => match op {
                platipus_language::ast::UnaryOp::Not => Some("Bool"),
                platipus_language::ast::UnaryOp::BitNot => Some("Int"),
                platipus_language::ast::UnaryOp::Negate => self.infer_expr_type(operand),
            },
            Expression::Logical { .. } => Some("Bool"),
            Expression::Binary { op, left, right, .. } => {
                use BinaryOp::*;
                match op {
                    Eq | Ne | Lt | Le | Gt | Ge => Some("Bool"),
                    Add => {
                        let l = self.infer_expr_type(left);
                        let r = self.infer_expr_type(right);
                        match (l, r) {
                            (Some("String"), Some("String")) => Some("String"),
                            (Some("Int"), Some("Int")) => Some("Int"),
                            (Some("Float"), Some("Float"))
                            | (Some("Float"), Some("Int"))
                            | (Some("Int"), Some("Float")) => Some("Float"),
                            _ => None,
                        }
                    }
                    Sub | Mul | Div | Rem => {
                        let l = self.infer_expr_type(left);
                        let r = self.infer_expr_type(right);
                        match (l, r) {
                            (Some("Int"), Some("Int")) => Some("Int"),
                            (Some("Float"), Some("Float"))
                            | (Some("Float"), Some("Int"))
                            | (Some("Int"), Some("Float")) => Some("Float"),
                            _ => None,
                        }
                    }
                    And | Or | Xor | Shl | Shr => Some("Int"),
                }
            }
            _ => None,
        }
    }

    /// Reports a mismatch between an explicit annotation and a statically
    /// known expression type. Unknown expression types are accepted.
    fn check_annotation_matches(
        &mut self,
        annotation: &Option<TypeExpr>,
        expression: &platipus_language::ast::Expression,
    ) {
        let Some(annotation) = annotation else {
            return;
        };
        let expected = match annotation.base_name() {
            "Text" => "String",
            other => other,
        };
        if !matches!(expected, "Bool" | "Int" | "Float" | "String") {
            return;
        }
        let Some(actual) = self.infer_expr_type(expression) else {
            return;
        };
        if actual != expected {
            self.diagnostics.error(
                Error::new(
                    ErrorKind::Semantic,
                    "type-mismatch",
                    format!("expected `{expected}` but found `{actual}`"),
                )
                .with_span(expression.span()),
            );
        }
    }

    fn check_type(&mut self, annotation: &Option<TypeExpr>) {
        let Some(annotation) = annotation else {
            return;
        };
        let name = annotation.base_name();
        if self.types.by_name(name).is_some() {
            return;
        }
        if self.scopes.lookup(name).is_some() {
            return;
        }
        self.diagnostics.error(
            Error::new(
                ErrorKind::Semantic,
                "unknown-type",
                format!("`{name}` is not a known type"),
            )
            .with_span(annotation.span())
            .with_help("built-in types are Int, Float, String, Bool, Any, and Void"),
        );
    }

    fn check_expression(&mut self, expression: &Expression) {
        match expression {
            Expression::Identifier(identifier) => {
                if self.scopes.lookup(&identifier.name).is_none()
                    && !is_builtin_function(&identifier.name)
                {
                    self.report_undefined(identifier);
                }
            }
            Expression::Lambda {
                parameters, body, ..
            } => {
                self.scopes.push(ScopeKind::Function);
                for parameter in parameters {
                    self.report_shadow(parameter);
                    self.declare(parameter, SymbolKind::Parameter);
                }
                match body {
                    LambdaBody::Expression(expression) => self.check_expression(expression),
                    LambdaBody::Block(block) => self.check_body(block),
                }
                self.scopes.pop();
            }
            Expression::ArrayLiteral(items, _) => {
                for item in items {
                    self.check_expression(item);
                }
            }
            Expression::ObjectLiteral(entries, _) => {
                for entry in entries {
                    self.check_expression(&entry.value);
                }
            }
            Expression::Unary { operand, .. } => self.check_expression(operand),
            Expression::Binary {
                op, left, right, span,
            } => {
                self.check_expression(left);
                self.check_expression(right);
                self.check_binary_operands(op, left, right, *span);
            }
            Expression::Logical { left, right, .. } => {
                self.check_expression(left);
                self.check_expression(right);
            }
            Expression::Assign { target, value, .. } => {
                self.check_expression(target);
                self.check_expression(value);
            }
            Expression::Call {
                callee, arguments, ..
            } => {
                self.check_expression(callee);
                for argument in arguments {
                    self.check_expression(argument);
                }
                self.check_call(callee, arguments);
                if self.in_pure_fn {
                    if let Some(identifier) = callee.as_identifier() {
                        let is_declared_local = self
                            .scopes
                            .lookup(&identifier.name)
                            .is_some_and(|symbol| {
                                matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Parameter)
                            });
                        if !is_declared_local
                            && !platipus_standard::is_pure_call(&identifier.name, arguments.len())
                        {
                            self.diagnostics.error(
                                Error::new(
                                    ErrorKind::Semantic,
                                    "impure-call-in-pure-fn",
                                    format!(
                                        "`{}` is not a pure function and cannot be called from a top-level `fn`",
                                        identifier.name
                                    ),
                                )
                                .with_span(identifier.span)
                                .with_help("top-level `fn` bodies must be pure; move side effects into a component handler"),
                            );
                        }
                    }
                }
            }
            Expression::Member { object, .. } => self.check_expression(object),
            Expression::Index { object, index, .. } => {
                self.check_expression(object);
                self.check_expression(index);
            }
            Expression::Await { operand, .. } => self.check_expression(operand),
            Expression::Event(_) => {}
            Expression::IntLiteral(..)
            | Expression::FloatLiteral(..)
            | Expression::StringLiteral(..)
            | Expression::BoolLiteral(..)
            | Expression::NullLiteral(..) => {}
        }
    }

    fn check_binary_operands(
        &mut self,
        op: &platipus_language::ast::BinaryOp,
        left: &Expression,
        right: &Expression,
        span: Span,
    ) {
        use platipus_language::ast::BinaryOp;
        let (Some(l), Some(r)) = (self.infer_expr_type(left), self.infer_expr_type(right)) else {
            return;
        };
        if matches!(op, BinaryOp::And | BinaryOp::Or | BinaryOp::Xor | BinaryOp::Shl | BinaryOp::Shr) {
            // Bitwise operators act on the 32-bit integer view, so both
            // operands must be provably integers; a Float would be silently
            // truncated by JavaScript's ToInt32.
            if l != "Int" || r != "Int" {
                self.diagnostics.error(
                    Error::new(
                        ErrorKind::Semantic,
                        "type-mismatch",
                        format!("`{}` requires integer operands, got `{l}` and `{r}`", op.symbol()),
                    )
                    .with_span(span)
                    .with_help("round with `trunc`, `floor`, or `round` before applying the operator"),
                );
            }
            return;
        }
        if !op.is_arithmetic() {
            return;
        }
        let numeric = |t: &str| t == "Int" || t == "Float";
        let valid = match op {
            BinaryOp::Add => {
                (l == "String" && r == "String") || (numeric(l) && numeric(r))
            }
            _ => numeric(l) && numeric(r),
        };
        if !valid {
            self.diagnostics.error(
                Error::new(
                    ErrorKind::Semantic,
                    "type-mismatch",
                    format!("cannot apply `{}` to `{l}` and `{r}`", op.symbol()),
                )
                .with_span(span),
            );
        }
    }

    /// The signature to check a call against: prefer the symbol visible in
    /// the *current* scope (a component `fn` in its own component, a
    /// top-level `fn` globally), identified by its declaration span; fall
    /// back to any registered signature with the same name.
    fn resolve_fn_signature(&self, identifier: &Identifier) -> Option<FnSignature> {
        let signatures = self.fn_signatures.get(&identifier.name)?;
        if let Some(symbol) = self.scopes.lookup(&identifier.name) {
            if symbol.kind == SymbolKind::Function {
                if let Some(signature) = signatures.iter().find(|sig| sig.span == symbol.span) {
                    return Some(signature.clone());
                }
            }
        }
        signatures.first().cloned()
    }

    fn register_fn_signature(&mut self, decl: &platipus_language::ast::FunctionDecl) {
        let min = decl
            .parameters
            .iter()
            .filter(|parameter| parameter.default.is_none())
            .count();
        let max = decl.parameters.len();
        self.fn_signatures
            .entry(decl.name.name.clone())
            .or_default()
            .push(FnSignature {
                min,
                max,
                params: decl
                    .parameters
                    .iter()
                    .map(|parameter| parameter.type_annotation.clone())
                    .collect(),
                span: decl.span,
            });
    }

    /// Arity and annotation checks at a call site. A call through an
    /// identifier is checked against the matching `fn` signature, a standard
    /// builtin, or an impure builtin; a builtin shared by both registries
    /// (`drop`: list vs storage) accepts the union of arities.
    fn check_call(&mut self, callee: &Expression, arguments: &[Expression]) {
        let Some(identifier) = callee.as_identifier() else {
            return;
        };
        if let Some(signature) = self.resolve_fn_signature(identifier) {
            if arguments.len() < signature.min || arguments.len() > signature.max {
                self.diagnostics.error(
                    Error::new(
                        ErrorKind::Semantic,
                        "wrong-argument-count",
                        format!(
                            "`{}` expects {}, got {}",
                            identifier.name,
                            describe_arity(signature.min, signature.max),
                            arguments.len()
                        ),
                    )
                    .with_span(identifier.span),
                );
            } else {
                self.check_call_types(identifier, &signature, arguments);
            }
            return;
        }
        let standard = platipus_standard::lookup(&identifier.name);
        let impure = platipus_standard::IMPURE_BUILTINS
            .iter()
            .find(|builtin| builtin.name == identifier.name);
        match (standard, impure) {
            (Some(builtin), Some(impure_builtin)) => {
                // `drop`: accept either arity
                if !builtin.accepts(arguments.len()) && !impure_builtin.accepts(arguments.len()) {
                    self.wrong_arity(identifier, &builtin, arguments.len());
                }
            }
            (Some(builtin), None) => {
                if !builtin.accepts(arguments.len()) {
                    self.wrong_arity(identifier, &builtin, arguments.len());
                }
            }
            (None, Some(builtin)) => {
                if !builtin.accepts(arguments.len()) {
                    self.wrong_arity(identifier, builtin, arguments.len());
                }
            }
            (None, None) => {}
        }
    }

    fn check_call_types(
        &mut self,
        identifier: &Identifier,
        signature: &FnSignature,
        arguments: &[Expression],
    ) {
        for (annotation, argument) in signature.params.iter().zip(arguments) {
            let Some(annotation) = annotation else {
                continue;
            };
            let expected = match annotation.base_name() {
                "Text" => "String",
                other => other,
            };
            if !matches!(expected, "Bool" | "Int" | "Float" | "String") {
                continue;
            }
            if let Some(actual) = self.infer_expr_type(argument) {
                if actual != expected {
                    self.diagnostics.error(
                        Error::new(
                            ErrorKind::Semantic,
                            "type-mismatch",
                            format!(
                                "argument of `{}` should be `{expected}`, got `{actual}`",
                                identifier.name
                            ),
                        )
                        .with_span(argument.span()),
                    );
                }
            }
        }
    }

    fn wrong_arity(&mut self, identifier: &Identifier, builtin: &platipus_standard::Builtin, got: usize) {
        self.diagnostics.error(
            Error::new(
                ErrorKind::Semantic,
                "wrong-argument-count",
                format!(
                    "`{}` expects {}, got {}",
                    identifier.name,
                    describe_arity(builtin.min_args, builtin.max_args),
                    got
                ),
            )
            .with_span(identifier.span),
        );
    }

    fn report_undefined(&mut self, identifier: &Identifier) {
        let name = identifier.name.as_str();
        let suggestions = self.elements.suggestions(name);
        if self.starts_like_an_element(name) && !suggestions.is_empty() {
            let mut error = Error::new(
                ErrorKind::Semantic,
                "unknown-element",
                format!("`{name}` is neither a primitive element nor a declared component"),
            )
            .with_span(identifier.span)
            .with_help("element and component names start with an uppercase letter");
            let names = suggestions
                .iter()
                .map(|candidate| format!("`{candidate}`"))
                .collect::<Vec<_>>()
                .join(", ");
            error = error.with_note(Note::new(format!("did you mean {names}?")));
            self.diagnostics.error(error);
            return;
        }
        self.diagnostics.error(
            Error::new(
                ErrorKind::Semantic,
                "undefined-identifier",
                format!("`{name}` is not defined in this scope"),
            )
            .with_span(identifier.span)
            .with_note(Note::new(
                "declare it with `state`, `derived`, `input`, or `fn` before use",
            )),
        );
    }

    fn starts_like_an_element(&self, name: &str) -> bool {
        name.chars()
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
    }

    fn declare(&mut self, name: &Identifier, kind: SymbolKind) {
        let symbol = Symbol {
            name: name.name.clone(),
            kind,
            span: name.span,
        };
        let defined = self.scopes.define(symbol);
        if defined.is_ok() {
            return;
        }
        let first = self
            .scopes
            .current()
            .and_then(|scope| scope.lookup_local(&name.name))
            .map(|symbol| symbol.span);
        let mut error = Error::new(
            ErrorKind::Semantic,
            "duplicate-declaration",
            format!("`{}` is already declared in this scope", name.name),
        )
        .with_span(name.span);
        if let Some(span) = first {
            error = error.with_note(Note::at("first declared here", span));
        }
        self.diagnostics.error(error);
    }
}

fn describe_arity(min: usize, max: usize) -> String {
    if min == max {
        format!("{} argument{}", min, if min == 1 { "" } else { "s" })
    } else {
        format!("between {} and {} arguments", min, max)
    }
}

fn declared_names<'a>(
    inputs: &'a [platipus_language::ast::InputDecl],
    items: &'a [ComponentItem],
) -> Vec<(&'a str, Span)> {
    let mut names: Vec<(&'a str, Span)> = inputs
        .iter()
        .map(|input| (input.name.as_str(), input.name.span))
        .collect();
    for item in items {
        match item {
            ComponentItem::State(decl) => names.push((decl.name.as_str(), decl.name.span)),
            ComponentItem::Derived(decl) => names.push((decl.name.as_str(), decl.name.span)),
            ComponentItem::Function(decl) => names.push((decl.name.as_str(), decl.name.span)),
            ComponentItem::Handler(handler) => {
                names.push((handler.name.as_str(), handler.name.span))
            }
            _ => {}
        }
    }
    names
}

fn is_builtin_function(name: &str) -> bool {
    platipus_standard::is_builtin_name(name)
}
