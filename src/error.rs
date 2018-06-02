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
    InvalidTerm(Token),
    CantFindChain(ChainName),
    UnexpectedEndOfFile,
    ZeroBacklink,
    PropertiesOfGenericChain(ChainName),
    DurationOfGenericChain(ChainName),
    NamedChainInAnonChain(String),
    ChainRedeclaration(ChainName),
    CantOpenOutputFile,
    MultipleOutChains(CodeLocation),
    UnstatisfiedBacklink(ChainName, usize, usize),
    UnnamedTopChain,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub spec: ErrorSpec,
    pub runtime: bool,
    pub location: Option<CodeLocation>,
}

impl Error {
    pub fn new(spec: ErrorSpec) -> Error {
        use self::ErrorSpec::*;
        let runtime = match spec {
            UnstatisfiedBacklink(..) => true,
            _ => false,
        };
        Error {
            spec,
            runtime,
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
                "{}{} on in {} on line {}                                                ",
                if self.runtime { "\r" } else { "" },
                "Error".red().bold(),
                loc.file.cyan().bold(),
                format!("{}:{}", loc.line, loc.column).cyan().bold()
            )
        } else {
            format!("{}", "Error".red().bold())
        };
        println!("{}", erl);

        // Print the error details
        match self.spec {
            FileNotFound(ref filename) => println!("Unable to find file: '{}'.", filename),
            ExpectedFound(ref expected, ref found) => {
                println!("Expected {}, found {}.", expected, found)
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
            InvalidTerm(ref found) => println!("Invalid term: {}.", found),
            CantFindChain(ref chain_name) => {
                println!("The {} could not be found in this scope.", chain_name)
            }
            UnexpectedEndOfFile => println!("Unexpected end of file."),
            ZeroBacklink => println!("Backlinks must be greater than 0."),
            PropertiesOfGenericChain(ref chain_name) => println!(
                "The {} contains expressions, so properties cannot be taken from it.",
                chain_name
            ),
            DurationOfGenericChain(ref chain_name) => println!(
                "The {} contains expressions, so it cannot be used to define a note duration",
                chain_name
            ),
            NamedChainInAnonChain(ref chain_name) => println!(
                "A named chain: '{}' cannot be declared inside an anonymous chain.",
                chain_name
            ),
            ChainRedeclaration(ref chain_name) => println!("Redeclaration of {}.", chain_name),
            CantOpenOutputFile => println!(
                "Unable to open output file.\nMake sure you have a default WAV player set."
            ),
            MultipleOutChains(ref loc) => println!(
                "Multiple output chains.\nFirst output declared on line {}.",
                loc
            ),
            UnstatisfiedBacklink(ref chain_name, expected, found) => println!(
                "Backlink \"!{}\" in {} expects at least {} previous link{}, but {} found.",
                expected,
                chain_name,
                expected,
                if expected > 1 { "s" } else { "" },
                match found {
                    0 => "none were".to_string(),
                    1 => "only 1 was".to_string(),
                    _ => format!("only {} were", found),
                }
            ),
            UnnamedTopChain => println!("Chains within a file's top-level scope must be named."),
        }
    }
}

pub type SonnyResult<T> = Result<T, Error>;
