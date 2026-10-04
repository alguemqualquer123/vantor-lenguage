use thiserror::Error;

/// Severity of a diagnostic (Spec §69).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}

/// Machine-readable diagnostic record (Spec §8 + §69).
///
/// Every diagnostic carries a stable code (`E0101`, `E0201`, ...) so CI and
/// IDEs can match on it, plus human-readable message, source location,
/// snippet, suggestion, notes and related locations.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Stable machine-readable code, e.g. `E0201`.
    pub code: &'static str,
    pub severity: Severity,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub snippet: String,
    pub message: String,
    pub suggestion: Option<String>,
    pub notes: Vec<String>,
    pub related: Vec<String>,
}

impl Diagnostic {
    /// Append a related location (`related` in the §69 UX contract).
    pub fn with_related(mut self, location: impl Into<String>) -> Self {
        self.related.push(location.into());
        self
    }

    /// Append several notes at once (versioned findings, Spec §70).
    pub fn with_notes(mut self, notes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.notes.extend(notes.into_iter().map(|n| n.into()));
        self
    }

    pub fn error(
        code: &'static str,
        file: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Diagnostic {
            code: code,
            severity: Severity::Error,
            file: file.into(),
            line,
            column,
            snippet: String::new(),
            message: message.into(),
            suggestion: None,
            notes: Vec::new(),
            related: Vec::new(),
        }
    }

    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = snippet.into();
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Human-readable rendering with color when `color=true`.
    pub fn render(&self, color: bool) -> String {
        if color {
            format!(
                "\x1b[31m{}[{}]\x1b[0m {}:{}:{}: {}\n  snippet: {}\n  suggestion: {}",
                self.severity,
                self.code,
                self.file,
                self.line,
                self.column,
                self.message,
                if self.snippet.is_empty() { "-" } else { &self.snippet },
                self.suggestion.as_deref().unwrap_or("-"),
            )
        } else {
            format!(
                "{}[{}] {}:{}:{}: {} | snippet: {} | suggestion: {}",
                self.severity,
                self.code,
                self.file,
                self.line,
                self.column,
                self.message,
                if self.snippet.is_empty() { "-" } else { &self.snippet },
                self.suggestion.as_deref().unwrap_or("-"),
            )
        }
    }

    /// Machine-readable JSON rendering for IDEs/CI (Spec §69).
    pub fn to_json(&self) -> String {        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        format!(
            "{{\"code\":\"{}\",\"severity\":\"{}\",\"file\":\"{}\",\"line\":{},\"column\":{},\"message\":\"{}\",\"snippet\":\"{}\",\"suggestion\":\"{}\"}}",
            self.code,
            self.severity,
            esc(&self.file),
            self.line,
            self.column,
            esc(&self.message),
            esc(&self.snippet),
            esc(self.suggestion.as_deref().unwrap_or("")),
        )
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render(false))
    }
}

// ---- Diagnostic code catalogue (stable, versioned per Spec §70) ----
// E01xx: lexical | E02xx: parse/syntax | E03xx: types | E04xx: codegen
// E05xx: runtime | E06xx: std/io | E07xx: concurrency | E08xx: errors/unwrap
pub mod codes {
    pub const UNCLOSED_STRING: &str = "E0101";
    pub const INVALID_LITERAL: &str = "E0102";
    pub const UNEXPECTED_CHAR: &str = "E0103";
    pub const EXPECTED_TOKEN: &str = "E0201";
    pub const INVALID_ASSIGNMENT: &str = "E0202";
    pub const UNREACHABLE_CODE: &str = "E0203";
    pub const TYPE_MISMATCH: &str = "E0301";
    pub const UNKNOWN_TYPE: &str = "E0302";
    pub const GENERIC_CONSTRAINT: &str = "E0303";
    pub const INTERFACE_UNIMPLEMENTED: &str = "E0304";
    pub const NULL_UNSAFE_ACCESS: &str = "E0305";
    pub const NARROWING_REQUIRES_CAST: &str = "E0306";
    pub const CODEGEN_FAILED: &str = "E0401";
    pub const RUNTIME_PANIC: &str = "E0501";
    pub const CONCURRENCY_RACE: &str = "E0701";
    pub const UNWRAP_ON_NONE: &str = "E0801";
}

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

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("diagnostic {0}")]
    WithDiagnostic(Box<Diagnostic>),

    #[error("compilation failed")]
    CompilationFailed,
}

impl Error {
    pub fn diagnostic(d: Diagnostic) -> Self {
        Error::WithDiagnostic(Box::new(d))
    }

    // ---- Part X: wrapping / chaining + programmatic inspection ----
    //
    // `wrap`/`with_context` never lose the original message: the inner
    // error text is preserved after a stable prefix so `code()` and
    // machine-readable matching keep working. `chain` returns the
    // outermost-to-innermost message list (split on the `": "` context
    // separator); `root_cause` is the innermost message.

    /// Add an outer context prefix, preserving the inner message.
    pub fn with_context(self, ctx: impl Into<String>) -> Self {
        let ctx = ctx.into();
        match self {
            Error::Lexical(m) => Error::Lexical(format!("{}: {}", ctx, m)),
            Error::Parse(m) => Error::Parse(format!("{}: {}", ctx, m)),
            Error::Type(m) => Error::Type(format!("{}: {}", ctx, m)),
            Error::Codegen(m) => Error::Codegen(format!("{}: {}", ctx, m)),
            Error::Io(m) => Error::Io(format!("{}: {}", ctx, m)),
            Error::Runtime(m) => Error::Runtime(format!("{}: {}", ctx, m)),
            Error::WithDiagnostic(mut d) => {
                d.notes.push(format!("context: {}", ctx));
                Error::WithDiagnostic(d)
            }
            Error::CompilationFailed => Error::Codegen(ctx),
        }
    }

    /// Alias for [`Error::with_context`] (call-site readability).
    pub fn wrap(self, ctx: impl Into<String>) -> Self {
        self.with_context(ctx)
    }

    /// Outermost-to-innermost message chain for programmatic inspection.
    pub fn chain(&self) -> Vec<String> {
        match self {
            Error::Lexical(m) | Error::Parse(m) | Error::Type(m)
            | Error::Codegen(m) | Error::Io(m) | Error::Runtime(m) => {
                m.split(": ").map(|s| s.to_string()).collect()
            }
            Error::WithDiagnostic(d) => {
                let mut out = vec![d.message.clone()];
                out.extend(d.notes.clone());
                out
            }
            Error::CompilationFailed => vec!["compilation failed".to_string()],
        }
    }

    /// Innermost message in [`Error::chain`].
    pub fn root_cause(&self) -> String {
        self.chain().last().cloned().unwrap_or_default()
    }

    /// Whether this error carries the given stable diagnostic `code`.
    pub fn has_code(&self, code: &str) -> bool {
        self.code() == code
            || self.chain().iter().any(|m| m.contains(code))
    }

    pub fn code(&self) -> &'static str {
        match self {
            Error::Lexical(_) => codes::UNCLOSED_STRING,
            Error::Parse(_) => codes::EXPECTED_TOKEN,
            Error::Type(_) => codes::TYPE_MISMATCH,
            Error::Codegen(_) => codes::CODEGEN_FAILED,
            Error::Io(_) => "E0601",
            Error::Runtime(_) => codes::RUNTIME_PANIC,
            Error::WithDiagnostic(d) => d.code,
            Error::CompilationFailed => codes::CODEGEN_FAILED,
        }
    }

    /// Human-readable message for this error (borrowed, no allocation).
    ///
    /// Returns the inner message for string variants, the diagnostic
    /// message for `WithDiagnostic`, and `"compilation failed"` for
    /// `CompilationFailed`.
    /// Complexity: O(1).
    #[inline]
    pub fn message(&self) -> &str {
        match self {
            Error::Lexical(m)
            | Error::Parse(m)
            | Error::Type(m)
            | Error::Codegen(m)
            | Error::Io(m)
            | Error::Runtime(m) => m.as_str(),
            Error::WithDiagnostic(d) => d.message.as_str(),
            Error::CompilationFailed => "compilation failed",
        }
    }

    /// Whether this is a lexical error.
    /// Complexity: O(1).
    #[inline]
    pub fn is_lexical(&self) -> bool {
        matches!(self, Error::Lexical(_))
    }

    /// Whether this is a parse error.
    /// Complexity: O(1).
    #[inline]
    pub fn is_parse(&self) -> bool {
        matches!(self, Error::Parse(_))
    }

    /// Whether this is a type error.
    /// Complexity: O(1).
    #[inline]
    pub fn is_type(&self) -> bool {
        matches!(self, Error::Type(_))
    }

    /// Whether this is a codegen error.
    /// Complexity: O(1).
    #[inline]
    pub fn is_codegen(&self) -> bool {
        matches!(self, Error::Codegen(_))
    }

    /// Whether this is an I/O error.
    /// Complexity: O(1).
    #[inline]
    pub fn is_io(&self) -> bool {
        matches!(self, Error::Io(_))
    }

    /// Whether this is a runtime error.
    /// Complexity: O(1).
    #[inline]
    pub fn is_runtime(&self) -> bool {
        matches!(self, Error::Runtime(_))
    }

    /// Whether this carries a full diagnostic.
    /// Complexity: O(1).
    #[inline]
    pub fn is_diagnostic(&self) -> bool {
        matches!(self, Error::WithDiagnostic(_))
    }

    /// Whether this is a generic compilation failure.
    /// Complexity: O(1).
    #[inline]
    pub fn is_compilation_failed(&self) -> bool {
        matches!(self, Error::CompilationFailed)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod partx_tests {
    use super::*;

    #[test]
    fn context_preserves_inner_message() {
        let e = Error::Type("E0301: bad".to_string()).with_context("while checking fn f");
        assert!(e.to_string().contains("bad"));
        assert!(e.to_string().contains("while checking fn f"));
        assert_eq!(e.code(), codes::TYPE_MISMATCH);
        assert!(e.has_code("E0301"));
    }

    #[test]
    fn chain_and_root_cause() {
        let e = Error::Runtime("outer: inner".to_string());
        assert_eq!(e.chain(), vec!["outer".to_string(), "inner".to_string()]);
        assert_eq!(e.root_cause(), "inner");
        assert_eq!(Error::CompilationFailed.root_cause(), "compilation failed");
    }

    #[test]
    fn diagnostic_builders() {
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "f.lex", 1, 2, "m")
            .with_related("see also g.lex:3:4")
            .with_notes(["n1".to_string(), "n2".to_string()]);
        assert_eq!(d.related.len(), 1);
        assert_eq!(d.notes.len(), 2);
    }

    #[test]
    fn message_per_variant() {
        assert_eq!(Error::Lexical("lex boom".into()).message(), "lex boom");
        assert_eq!(Error::Parse("parse boom".into()).message(), "parse boom");
        assert_eq!(Error::Type("type boom".into()).message(), "type boom");
        assert_eq!(Error::Codegen("codegen boom".into()).message(), "codegen boom");
        assert_eq!(Error::Io("io boom".into()).message(), "io boom");
        assert_eq!(Error::Runtime("rt boom".into()).message(), "rt boom");
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "f.lex", 1, 1, "diag msg");
        assert_eq!(Error::diagnostic(d).message(), "diag msg");
        assert_eq!(Error::CompilationFailed.message(), "compilation failed");
    }

    #[test]
    fn is_predicates_cover_all_variants() {
        let lex = Error::Lexical("x".into());
        assert!(lex.is_lexical() && !lex.is_parse() && !lex.is_type() && !lex.is_codegen() && !lex.is_io() && !lex.is_runtime() && !lex.is_diagnostic() && !lex.is_compilation_failed());
        assert!(Error::Parse("x".into()).is_parse());
        assert!(Error::Type("x".into()).is_type());
        assert!(Error::Codegen("x".into()).is_codegen());
        assert!(Error::Io("x".into()).is_io());
        assert!(Error::Runtime("x".into()).is_runtime());
        let d = Diagnostic::error(codes::TYPE_MISMATCH, "f.lex", 1, 1, "m");
        assert!(Error::diagnostic(d).is_diagnostic());
        assert!(Error::CompilationFailed.is_compilation_failed());
        assert!(!Error::CompilationFailed.is_io());
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing file");
        let e = Error::from(io);
        assert!(e.is_io());
        assert!(e.message().contains("missing file"));
        assert_eq!(e.code(), "E0601");
    }

    #[test]
    fn wrap_chain_context_codes() {
        let base = Error::Type("E0301: bad".into());
        let wrapped = base.wrap("while checking");
        assert!(wrapped.message().contains("while checking"));
        assert!(wrapped.message().contains("bad"));
        assert_eq!(wrapped.chain(), vec!["while checking".to_string(), "E0301".to_string(), "bad".to_string()]);
        assert_eq!(wrapped.root_cause(), "bad");
        assert!(wrapped.has_code("E0301"));
        assert!(wrapped.has_code(codes::TYPE_MISMATCH));
        // Diagnostic wrap appends context note and preserves chain.
        let d = Diagnostic::error(codes::EXPECTED_TOKEN, "f.lex", 2, 3, "expected `;`");
        let w = Error::diagnostic(d).with_context("while parsing item");
        assert!(w.is_diagnostic());
        assert!(w.chain().iter().any(|m| m.contains("while parsing item")));
        assert_eq!(w.code(), codes::EXPECTED_TOKEN);
        // Codes catalogue sanity.
        assert_eq!(codes::UNCLOSED_STRING, "E0101");
        assert_eq!(codes::EXPECTED_TOKEN, "E0201");
        assert_eq!(codes::TYPE_MISMATCH, "E0301");
        assert_eq!(codes::CODEGEN_FAILED, "E0401");
        assert_eq!(codes::RUNTIME_PANIC, "E0501");
        assert_eq!(codes::CONCURRENCY_RACE, "E0701");
        assert_eq!(codes::UNWRAP_ON_NONE, "E0801");
    }
}
