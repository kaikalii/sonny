use colored::*;
use either::*;

use builder::{variable::*, *};
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
    DebugVar(Variable),
    DebugString(Variable),
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorSeverity {
    Fatal,
    Debug,
    Print,
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorTime {
    Compile,
    Run,
    RunClear,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub spec: ErrorSpec,
    pub runtime: ErrorTime,
    pub severity: ErrorSeverity,
    pub location: Option<CodeLocation>,
}

impl Error {
    pub fn new(spec: ErrorSpec) -> Error {
        use self::{ErrorSeverity::*, ErrorSpec::*, ErrorTime::*};
        let runtime = match spec {
            UnstatisfiedBacklink(..) => RunClear,
            DebugVar(..) | DebugString(..) => Run,
            _ => Compile,
        };
        let severity = match spec {
            DebugVar(..) => Debug,
            DebugString(..) => Print,
            _ => Fatal,
        };
        Error {
            spec,
            runtime,
            severity,
            location: None,
        }
    }
    pub fn on_line(mut self, line: CodeLocation) -> Error {
        self.location = Some(line);
        self
    }
    pub fn report(self) {
        use self::{ErrorSeverity::*, ErrorSpec::*, ErrorTime::*};
        // Print the generic error message
        let severity_str = match self.severity {
            Fatal => "Error".red().bold(),
            Debug => "Debug".yellow().bold(),
            Print => "Print".yellow().bold(),
        };
        if let Run = self.runtime {
            println!();
        }
        let erl = if let Some(ref loc) = self.location {
            format!(
                "{}{} in {} on line {}                                         ",
                match self.runtime {
                    Compile => "",
                    Run => "\n",
                    RunClear => "\r",
                },
                severity_str,
                loc.file.cyan().bold(),
                format!("{}:{}", loc.line, loc.column).cyan().bold()
            )
        } else {
            format!("{}", severity_str)
        };
        println!("{}", erl);

        // Print the error details
        match self.spec {
            FileNotFound(filename) => println!("Unable to find file: '{}'.", filename),
            ExpectedFound(expected, found) => println!("Expected {}, found {}.", expected, found),
            CloseDelimeter(found) => println!("Invalid close delimeter: {}.", found),
            InvalidDelimeter(found) => {
                println!("Delimeter is not valid in this context: {}.", found)
            }
            InvalidPitch(found) => println!("Expected pitch, found {}.", found),
            DurationQuantifier(found) => println!("Expected duration quantifier, found {}.", found),
            InvalidBackLink(found) => println!("Expected number after '!', found {}.", found),
            InvalidKeyword(found) => println!("Keyword is invalid in this context: '{}'.", found),
            InvalidTerm(found) => println!("Invalid term: {}.", found),
            CantFindChain(chain_name) => {
                println!("The {} could not be found in this scope.", chain_name)
            }
            UnexpectedEndOfFile => println!("Unexpected end of file."),
            ZeroBacklink => println!("Backlinks must be greater than 0."),
            PropertiesOfGenericChain(chain_name) => println!(
                "The {} contains expressions, so properties cannot be taken from it.",
                chain_name
            ),
            DurationOfGenericChain(chain_name) => println!(
                "The {} contains expressions, so it cannot be used to define a note duration",
                chain_name
            ),
            NamedChainInAnonChain(chain_name) => println!(
                "A named chain: '{}' cannot be declared inside an anonymous chain.",
                chain_name
            ),
            ChainRedeclaration(chain_name) => println!("Redeclaration of {}.", chain_name),
            CantOpenOutputFile => println!(
                "Unable to open output file.\nMake sure you have a default WAV player set."
            ),
            MultipleOutChains(loc) => println!(
                "Multiple output chains.\nFirst output declared on line {}.",
                loc
            ),
            UnstatisfiedBacklink(chain_name, expected, found) => println!(
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
            DebugVar(var) => println!("{:?}", var),
            DebugString(var) => println!("{}", var),
        }
    }
}

pub type SonnyResult<T> = Result<T, Error>;
