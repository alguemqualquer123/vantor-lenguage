use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("lexical error: {0}")]
    Lexical(String),
    
    #[error("parse error: {0}")]
    Parse(String),
    
    #[error("type error: {0}")]
    Type(String),
    
    #[error("codegen error: {0}")]
    Codegen(String),
    
    #[error("io error: {0}")]
    Io(String),
    
    #[error("compilation failed")]
    CompilationFailed,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
