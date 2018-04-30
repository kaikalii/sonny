use either::*;

use builder::*;
use lexer::{Token, TokenType};
use parser::*;

#[derive(Debug, Clone)]
pub enum ErrorSpec {
    FileNotFound(String),
    ExpectedFound(Either<TokenType, String>, Token),
    CloseDelimeter(String),
    InvalidDelimeter(String),
    InvalidPitch(Token),
    DurationQuantifier(Token),
    InvalidBackLink(Token),
    InvalidKeyword(String),
    ExpectedNotesProperty(Token),
    InvalidTerm(Token),
    PeriodCantFindChain(ChainName),
}

#[derive(Debug, Clone)]
pub struct Error {
    pub spec: ErrorSpec,
    pub line: Option<(usize, usize)>,
}

impl Error {
    pub fn new(spec: ErrorSpec) -> Error {
        Error { spec, line: None }
    }
    pub fn on_line(mut self, line: (usize, usize)) -> Error {
        self.line = Some(line);
        self
    }
}

pub type SonnyResult<T> = Result<T, Error>;
