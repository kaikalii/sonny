use colored::*;
use either::*;

use builder::*;
use lexer::{Token, TokenType};

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
    UnexpectedEndOfFile,
    ZeroBacklink,
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
    pub fn report(&self) {
        use self::ErrorSpec::*;
        let erl = if let Some(lineno) = self.line {
            format!(
                "{} on line {}",
                "Error".red().bold(),
                format!(
                    "{}:{}",
                    lineno.0.to_string().white(),
                    lineno.1.to_string().white()
                )
            )
        } else {
            format!("{}", "Error".red().bold())
        };
        println!("{}", erl);
        match self.spec {
            FileNotFound(ref filename) => println!("Unable to find file: '{}'.", filename),
            ExpectedFound(ref expected, ref found) => {
                if expected.is_left() {
                    println!("Expected {}, found {}.", expected, found)
                } else {
                    println!("Expected '{}', found {}.", expected, found)
                }
            }
            CloseDelimeter(ref found) => println!("Invalid close delimeter: '{}'.", found),
            InvalidDelimeter(ref found) => {
                println!("Delimeter is not valid in this context: '{}'.", found)
            }
            InvalidPitch(ref found) => println!("Expected pitch, found: '{}'.", found),
            DurationQuantifier(ref found) => {
                println!("Expected duration quantifier, found: '{}'.", found)
            }
            InvalidBackLink(ref found) => {
                println!("Expected number after '!', found: '{}'.", found)
            }
            InvalidKeyword(ref found) => {
                println!("Keyword is invalid in this context: '{}'.", found)
            }
            ExpectedNotesProperty(ref found) => {
                println!("Expected notes property, found: '{}'.", found)
            }
            InvalidTerm(ref found) => println!("Invalid term: '{}'.", found),
            PeriodCantFindChain(ref chain_name) => {
                println!("Unknown chain referenced in period: '{}'.", chain_name)
            }
            UnexpectedEndOfFile => println!("Unexpected end of file."),
            ZeroBacklink => println!("Backlinks must be greater than 0"),
        }
    }
}

pub type SonnyResult<T> = Result<T, Error>;
