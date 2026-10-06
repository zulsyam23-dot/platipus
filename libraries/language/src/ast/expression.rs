use platipus_diagnostics::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

impl Identifier {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl std::fmt::Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Negate,
    Not,
}

impl UnaryOp {
    pub const fn symbol(self) -> &'static str {
        match self {
            UnaryOp::Negate => "-",
            UnaryOp::Not => "!",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl BinaryOp {
    pub const fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Rem => "%",
            BinaryOp::Eq => "==",
            BinaryOp::Ne => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Le => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::Ge => ">=",
        }
    }

    pub const fn is_comparison(self) -> bool {
        matches!(
            self,
            BinaryOp::Eq | BinaryOp::Ne | BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge
        )
    }

    pub const fn is_arithmetic(self) -> bool {
        matches!(
            self,
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Rem
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogicalOp {
    And,
    Or,
}

impl LogicalOp {
    pub const fn symbol(self) -> &'static str {
        match self {
            LogicalOp::And => "&&",
            LogicalOp::Or => "||",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

impl AssignOp {
    pub const fn symbol(self) -> &'static str {
        match self {
            AssignOp::Assign => "=",
            AssignOp::Add => "+=",
            AssignOp::Sub => "-=",
            AssignOp::Mul => "*=",
            AssignOp::Div => "/=",
            AssignOp::Rem => "%=",
        }
    }

    pub const fn to_binary(self) -> Option<BinaryOp> {
        match self {
            AssignOp::Assign => None,
            AssignOp::Add => Some(BinaryOp::Add),
            AssignOp::Sub => Some(BinaryOp::Sub),
            AssignOp::Mul => Some(BinaryOp::Mul),
            AssignOp::Div => Some(BinaryOp::Div),
            AssignOp::Rem => Some(BinaryOp::Rem),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyKey {
    Named(Identifier),
    String(String, Span),
    Index(u32, Span),
}

impl PropertyKey {
    pub fn span(&self) -> Span {
        match self {
            PropertyKey::Named(identifier) => identifier.span,
            PropertyKey::String(_, span) | PropertyKey::Index(_, span) => *span,
        }
    }

    pub fn name(&self) -> String {
        match self {
            PropertyKey::Named(identifier) => identifier.name.clone(),
            PropertyKey::String(name, _) => name.clone(),
            PropertyKey::Index(index, _) => index.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectEntry {
    pub key: PropertyKey,
    pub value: Expression,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    StringLiteral(String, Span),
    BoolLiteral(bool, Span),
    NullLiteral(Span),
    Identifier(Identifier),
    Event(Span),
    ArrayLiteral(Vec<Expression>, Span),
    ObjectLiteral(Vec<ObjectEntry>, Span),
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,
        span: Span,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    Logical {
        op: LogicalOp,
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    Assign {
        op: AssignOp,
        target: Box<Expression>,
        value: Box<Expression>,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    },
    Member {
        object: Box<Expression>,
        property: Identifier,
        span: Span,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    Await {
        operand: Box<Expression>,
        span: Span,
    },
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Expression::IntLiteral(_, span)
            | Expression::FloatLiteral(_, span)
            | Expression::StringLiteral(_, span)
            | Expression::BoolLiteral(_, span)
            | Expression::NullLiteral(span)
            | Expression::Event(span) => *span,
            Expression::Identifier(identifier) => identifier.span,
            Expression::ArrayLiteral(_, span) | Expression::ObjectLiteral(_, span) => *span,
            Expression::Unary { span, .. }
            | Expression::Binary { span, .. }
            | Expression::Logical { span, .. }
            | Expression::Assign { span, .. }
            | Expression::Call { span, .. }
            | Expression::Member { span, .. }
            | Expression::Index { span, .. }
            | Expression::Await { span, .. } => *span,
        }
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Expression::IntLiteral(_, _)
                | Expression::FloatLiteral(_, _)
                | Expression::StringLiteral(_, _)
                | Expression::BoolLiteral(_, _)
                | Expression::NullLiteral(_)
        )
    }

    pub fn as_identifier(&self) -> Option<&Identifier> {
        match self {
            Expression::Identifier(identifier) => Some(identifier),
            _ => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Expression::IntLiteral(_, _) => "Int",
            Expression::FloatLiteral(_, _) => "Float",
            Expression::StringLiteral(_, _) => "String",
            Expression::BoolLiteral(_, _) => "Bool",
            Expression::NullLiteral(_) => "Null",
            Expression::ArrayLiteral(_, _) => "Array",
            Expression::ObjectLiteral(_, _) => "Object",
            _ => "Unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    Named {
        name: Identifier,
        arguments: Vec<TypeExpr>,
        span: Span,
    },
    Nullable {
        inner: Box<TypeExpr>,
        span: Span,
    },
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named { span, .. } | TypeExpr::Nullable { span, .. } => *span,
        }
    }

    pub fn base_name(&self) -> &str {
        match self {
            TypeExpr::Named { name, .. } => name.as_str(),
            TypeExpr::Nullable { inner, .. } => inner.base_name(),
        }
    }
}
