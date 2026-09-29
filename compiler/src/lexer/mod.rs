pub mod keyword;
pub mod literal;
pub mod operator;
pub mod scanner;
pub mod token;

pub use keyword::{ALL as KEYWORDS, CONTEXTUAL_ALL, ContextualKeyword, Keyword};
pub use literal::Literal;
pub use operator::{OPERATORS, OperatorInfo, Precedence};
pub use scanner::{ScanResult, Scanner};
pub use token::{Token, TokenKind};

use crate::diagnostics::Error;

pub fn tokenize(source: &str) -> Result<Vec<Token>, Vec<Error>> {
    let result = Scanner::new(source).scan();
    if result.errors.is_empty() {
        Ok(result.tokens)
    } else {
        Err(result.errors)
    }
}

pub fn tokenize_lossy(source: &str) -> (Vec<Token>, Vec<Error>) {
    let result = Scanner::new(source).scan();
    (result.tokens, result.errors)
}
