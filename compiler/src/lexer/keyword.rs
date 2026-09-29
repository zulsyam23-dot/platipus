use super::token::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    App,
    Component,
    Fn,
    Input,
    Import,
    From,
    Api,
    Theme,
    Style,
    Test,
    State,
    Derived,
    Shared,
    Global,
    Persistent,
    On,
    Emit,
    Bind,
    Responsive,
    If,
    Else,
    For,
    In,
    Return,
    Break,
    Continue,
    Async,
    Await,
    Try,
    Catch,
    True,
    False,
    Null,
}

pub const ALL: &[(Keyword, &str)] = &[
    (Keyword::App, "app"),
    (Keyword::Component, "component"),
    (Keyword::Fn, "fn"),
    (Keyword::Input, "input"),
    (Keyword::Import, "import"),
    (Keyword::From, "from"),
    (Keyword::Api, "api"),
    (Keyword::Theme, "theme"),
    (Keyword::Style, "style"),
    (Keyword::Test, "test"),
    (Keyword::State, "state"),
    (Keyword::Derived, "derived"),
    (Keyword::Shared, "shared"),
    (Keyword::Global, "global"),
    (Keyword::Persistent, "persistent"),
    (Keyword::On, "on"),
    (Keyword::Emit, "emit"),
    (Keyword::Bind, "bind"),
    (Keyword::Responsive, "responsive"),
    (Keyword::If, "if"),
    (Keyword::Else, "else"),
    (Keyword::For, "for"),
    (Keyword::In, "in"),
    (Keyword::Return, "return"),
    (Keyword::Break, "break"),
    (Keyword::Continue, "continue"),
    (Keyword::Async, "async"),
    (Keyword::Await, "await"),
    (Keyword::Try, "try"),
    (Keyword::Catch, "catch"),
    (Keyword::True, "true"),
    (Keyword::False, "false"),
    (Keyword::Null, "null"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextualKeyword {
    Event,
    Element,
    Expect,
    To,
    Normal,
    Hover,
    Pressed,
    Focused,
    Disabled,
    Selected,
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

pub const CONTEXTUAL_ALL: &[(ContextualKeyword, &str)] = &[
    (ContextualKeyword::Event, "event"),
    (ContextualKeyword::Element, "element"),
    (ContextualKeyword::Expect, "expect"),
    (ContextualKeyword::To, "to"),
    (ContextualKeyword::Normal, "normal"),
    (ContextualKeyword::Hover, "hover"),
    (ContextualKeyword::Pressed, "pressed"),
    (ContextualKeyword::Focused, "focused"),
    (ContextualKeyword::Disabled, "disabled"),
    (ContextualKeyword::Selected, "selected"),
    (ContextualKeyword::Get, "get"),
    (ContextualKeyword::Post, "post"),
    (ContextualKeyword::Put, "put"),
    (ContextualKeyword::Delete, "delete"),
    (ContextualKeyword::Patch, "patch"),
];

impl Keyword {
    pub const fn as_str(self) -> &'static str {
        match self {
            Keyword::App => "app",
            Keyword::Component => "component",
            Keyword::Fn => "fn",
            Keyword::Input => "input",
            Keyword::Import => "import",
            Keyword::From => "from",
            Keyword::Api => "api",
            Keyword::Theme => "theme",
            Keyword::Style => "style",
            Keyword::Test => "test",
            Keyword::State => "state",
            Keyword::Derived => "derived",
            Keyword::Shared => "shared",
            Keyword::Global => "global",
            Keyword::Persistent => "persistent",
            Keyword::On => "on",
            Keyword::Emit => "emit",
            Keyword::Bind => "bind",
            Keyword::Responsive => "responsive",
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::For => "for",
            Keyword::In => "in",
            Keyword::Return => "return",
            Keyword::Break => "break",
            Keyword::Continue => "continue",
            Keyword::Async => "async",
            Keyword::Await => "await",
            Keyword::Try => "try",
            Keyword::Catch => "catch",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::Null => "null",
        }
    }

    pub const fn token_kind(self) -> TokenKind {
        TokenKind::Keyword
    }

    pub fn from_ident(ident: &str) -> Option<Keyword> {
        ALL.iter()
            .find(|(_, text)| *text == ident)
            .map(|(keyword, _)| *keyword)
    }

    pub fn is_declaration_keyword(self) -> bool {
        matches!(
            self,
            Keyword::App
                | Keyword::Component
                | Keyword::Fn
                | Keyword::Input
                | Keyword::Import
                | Keyword::Api
                | Keyword::Theme
                | Keyword::Test
        )
    }

    pub fn is_state_modifier(self) -> bool {
        matches!(
            self,
            Keyword::Shared | Keyword::Global | Keyword::Persistent
        )
    }
}

impl ContextualKeyword {
    pub const fn as_str(self) -> &'static str {
        match self {
            ContextualKeyword::Event => "event",
            ContextualKeyword::Element => "element",
            ContextualKeyword::Expect => "expect",
            ContextualKeyword::To => "to",
            ContextualKeyword::Normal => "normal",
            ContextualKeyword::Hover => "hover",
            ContextualKeyword::Pressed => "pressed",
            ContextualKeyword::Focused => "focused",
            ContextualKeyword::Disabled => "disabled",
            ContextualKeyword::Selected => "selected",
            ContextualKeyword::Get => "get",
            ContextualKeyword::Post => "post",
            ContextualKeyword::Put => "put",
            ContextualKeyword::Delete => "delete",
            ContextualKeyword::Patch => "patch",
        }
    }

    pub fn from_ident(ident: &str) -> Option<ContextualKeyword> {
        CONTEXTUAL_ALL
            .iter()
            .find(|(_, text)| *text == ident)
            .map(|(keyword, _)| *keyword)
    }

    pub fn is_style_state(self) -> bool {
        matches!(
            self,
            ContextualKeyword::Normal
                | ContextualKeyword::Hover
                | ContextualKeyword::Pressed
                | ContextualKeyword::Focused
                | ContextualKeyword::Disabled
                | ContextualKeyword::Selected
        )
    }

    pub fn is_api_method(self) -> bool {
        matches!(
            self,
            ContextualKeyword::Get
                | ContextualKeyword::Post
                | ContextualKeyword::Put
                | ContextualKeyword::Delete
                | ContextualKeyword::Patch
        )
    }
}
