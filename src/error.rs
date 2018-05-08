use colored::*;
use either::*;

use builder::*;
use lexer::{CodeLocation, Token, TokenType};

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
    CantFindChain(ChainName),
    UnexpectedEndOfFile,
    ZeroBacklink,
    PropertyOfGenericChain(ChainName, String),
    NamedChainInAnonChain(String),
    ChainRedeclaration(ChainName),
    CantOpenOutputFile,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub spec: ErrorSpec,
    pub location: Option<CodeLocation>,
}

impl Error {
    pub fn new(spec: ErrorSpec) -> Error {
        Error {
            spec,
            location: None,
        }
    }
    pub fn on_line(mut self, line: CodeLocation) -> Error {
        self.location = Some(line);
        self
    }
    pub fn report(&self) {
        use self::ErrorSpec::*;
        // Print the generic error message
        let erl = if let Some(ref loc) = self.location {
            format!(
                "{} on line {}",
                "Error".red().bold(),
                format!("{}", loc).cyan().bold()
            )
        } else {
            format!("{}", "Error".red().bold())
        };
        println!("{}", erl);

        // Print the error details
        match self.spec {
            FileNotFound(ref filename) => println!("Unable to find file: '{}'.", filename),
            ExpectedFound(ref expected, ref found) => {
                if expected.is_left() {
                    println!("Expected {}, found {}.", expected, found)
                } else {
                    println!("Expected {}, found {}.", expected, found)
                }
            }
            CloseDelimeter(ref found) => println!("Invalid close delimeter: {}.", found),
            InvalidDelimeter(ref found) => {
                println!("Delimeter is not valid in this context: {}.", found)
            }
            InvalidPitch(ref found) => println!("Expected pitch, found {}.", found),
            DurationQuantifier(ref found) => {
                println!("Expected duration quantifier, found {}.", found)
            }
            InvalidBackLink(ref found) => println!("Expected number after '!', found {}.", found),
            InvalidKeyword(ref found) => {
                println!("Keyword is invalid in this context: '{}'.", found)
            }
            ExpectedNotesProperty(ref found) => {
                println!("Expected notes property, found {}.", found)
            }
            InvalidTerm(ref found) => println!("Invalid term: '{}'.", found),
            CantFindChain(ref chain_name) => {
                println!("The {} could not be found in this scope.", chain_name)
            }
            UnexpectedEndOfFile => println!("Unexpected end of file."),
            ZeroBacklink => println!("Backlinks must be greater than 0."),
            PropertyOfGenericChain(ref chain_name, ref property_name) => println!(
                "The {} contains expressions, so the property '{}' cannot be taken from it.",
                chain_name, property_name
            ),
            NamedChainInAnonChain(ref chain_name) => println!(
                "A named chain: '{}' cannot be declared inside an anonymous chain.",
                chain_name
            ),
            ChainRedeclaration(ref chain_name) => println!("Redeclaration of {}.", chain_name),
            CantOpenOutputFile => println!(
                "Unable to open output file.\nMake sure you have a default WAV player set."
            ),
        }
    }
}

pub type SonnyResult<T> = Result<T, Error>;
