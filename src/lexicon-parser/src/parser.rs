use crate::ast::*;
use lexicon_core::error::{codes, Diagnostic, Error, Result};
use lexicon_core::span::Span;
use lexicon_lexer::tokens::SpannedToken as LexerSpannedToken;
use lexicon_lexer::tokens::Token;

/// A single syntax error with source location.
///
/// `file` names the compilation unit under parse (see
/// [`Parser::with_file`]; defaults to `"<input>"` when the caller only has
/// a token stream). `line`/`column` are 1-based positions taken from the
/// offending token's [`Span`]; `span` keeps the full byte range for
/// IDE/diagnostic use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub span: Span,
}

impl ParseError {
    pub fn new(file: impl Into<String>, span: Span, message: impl Into<String>) -> Self {
        ParseError {
            file: file.into(),
            line: span.line,
            column: span.column,
            message: message.into(),
            span,
        }
    }

    /// Structured diagnostic (`E0201`, Spec §69) preserving the location.
    pub fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic::error(
            codes::EXPECTED_TOKEN,
            self.file.clone(),
            self.line,
            self.column,
            self.message.clone(),
        )
        .with_snippet(format!("span {}..{}", self.span.start, self.span.end))
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}: {}", self.file, self.line, self.column, self.message)
    }
}

impl std::error::Error for ParseError {}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::diagnostic(e.to_diagnostic())
    }
}

pub struct Parser {
    tokens: Vec<LexerSpannedToken>,
    pos: usize,
    errors: Vec<ParseError>,
    file: String,
    /// Expression-nesting depth (see [`Parser::parse_expression`]).
    depth: usize,
}

/// Maximum expression-nesting depth (parens, calls-in-calls, blocks in
/// expression position…). Legitimate programs nest dozens deep; hostile
/// or runaway input gets a clean diagnostic instead of aborting the
/// process with a native stack overflow. The execution threads add a
/// large stack behind this, so the budget — not the OS — always wins.
pub const MAX_PARSE_DEPTH: usize = 2000;

/// Lookahead for an RPC service header: `service Name {`, where `i` is
/// the offset (past modifiers) of the `service` identifier.
/// Conservative on purpose: anything else falls through to the
/// global-variable path, so existing code is unaffected.
fn is_service_head(parser: &Parser, i: usize) -> bool {
    let ident_at = |n: usize| match parser.peek(n).token {
        Token::Ident(s) => Some(s),
        _ => None,
    };
    match (ident_at(i), ident_at(i + 1)) {
        (Some(kw), Some(_)) if kw == "service" => {
            matches!(parser.peek(i + 2).token, Token::LBrace)
        }
        _ => false,
    }
}

impl Parser {
    /// Comment tokens are trivia (Spec §1): the lexer keeps them for
    /// `leading_trivia`, but the grammar never references them — filtering
    /// here fixes every position at once (decls, blocks, exprs, match arms)
    /// while token spans keep true line/column numbers.
    fn without_trivia(tokens: Vec<LexerSpannedToken>) -> Vec<LexerSpannedToken> {
        tokens
            .into_iter()
            .filter(|t| !matches!(t.token, Token::Comment(_) | Token::BlockComment(_)))
            .collect()
    }

    pub fn new(tokens: Vec<LexerSpannedToken>) -> Self {
        Parser {
            tokens: Self::without_trivia(tokens),
            pos: 0,
            errors: Vec::new(),
            file: "<input>".to_string(),
            depth: 0,
        }
    }

    /// Create a parser that tags collected [`ParseError`]s with `file`.
    pub fn with_file(tokens: Vec<LexerSpannedToken>, file: impl Into<String>) -> Self {
        Parser {
            tokens: Self::without_trivia(tokens),
            pos: 0,
            errors: Vec::new(),
            file: file.into(),
            depth: 0,
        }
    }

    /// Retarget the file label used for subsequently recorded errors.
    pub fn set_file(&mut self, file: impl Into<String>) {
        self.file = file.into();
    }

    /// The file label attached to recorded errors.
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Legacy accessor (kept for backward compatibility): converts the
    /// collected [`ParseError`]s into structured [`Error::WithDiagnostic`]
    /// values (`E0201`, Spec §69) preserving file/line/column/message.
    pub fn into_errors(self) -> Vec<Error> {
        self.errors.into_iter().map(Error::from).collect()
    }

    /// Borrow the collected syntax errors without consuming the parser.
    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    /// Drain the collected syntax errors, leaving the buffer empty.
    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }

    /// Consume the parser and return the collected syntax errors.
    pub fn into_parse_errors(self) -> Vec<ParseError> {
        self.errors
    }

    /// Record an error at `span`, returning an [`Error::Parse`] carrying
    /// the same message so callers can propagate with `?`.
    fn push_error(&mut self, span: Span, message: String) -> Error {
        self.errors.push(ParseError::new(self.file.clone(), span, message.clone()));
        Error::Parse(message)
    }

    /// Record `err` at the current token when the failing `parse_*` call
    /// did not record anything itself (avoids double-reporting a single
    /// mistake: `expect` and friends already push on failure).
    fn record_if_new(&mut self, errors_before: usize, err: &Error) {
        if self.errors.len() == errors_before {
            let span = self.current().span;
            let message = match err {
                Error::Parse(m)
                | Error::Lexical(m)
                | Error::Type(m)
                | Error::Codegen(m)
                | Error::Io(m)
                | Error::Runtime(m) => m.clone(),
                Error::WithDiagnostic(d) => d.message.clone(),
                Error::CompilationFailed => "compilation failed".to_string(),
            };
            self.errors.push(ParseError::new(self.file.clone(), span, message));
        }
    }

    /// Panic-free recovery: skip tokens until a likely boundary at the SAME
    /// brace depth (`;` — consumed — or the closing `}` / a declaration
    /// keyword / EOF — left for the caller), so one bad statement never
    /// swallows the rest of its block (brace-aware: nested `{...}` is
    /// skipped whole instead of leaking inner tokens to outer levels).
    fn synchronize(&mut self) {
        let mut depth = 0usize;
        while !self.is_at_end() {
            match &self.current().token {
                Token::LBrace => {
                    depth += 1;
                    self.advance();
                }
                Token::RBrace => {
                    if depth == 0 {
                        return;
                    }
                    depth -= 1;
                    self.advance();
                }
                Token::Semi => {
                    // A `;` ends the broken statement — but only at the
                    // entry depth. Inside nested `{...}` it belongs to
                    // inner code; keep skipping so the block structure
                    // survives (the `}` arm rebalances on the way out).
                    if depth == 0 {
                        self.advance();
                        return;
                    }
                    self.advance();
                }
                Token::Fn
                | Token::Struct
                | Token::Enum
                | Token::Class
                | Token::Interface
                | Token::Trait
                | Token::Type
                | Token::Import
                | Token::Module
                | Token::Pub
                | Token::At
                | Token::Hash => return,
                // `impl` has no dedicated token; treat `impl` as a boundary.
                Token::Ident(s) if s == "impl" => return,
                _ => self.advance(),
            }
        }
    }

    fn current(&self) -> LexerSpannedToken {
        self.tokens
            .get(self.pos)
            .cloned()
            .unwrap_or(LexerSpannedToken::new(Token::Eof, Span::default()))
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.current().token, Token::Eof)
    }

    fn peek(&self, n: usize) -> LexerSpannedToken {
        self.tokens
            .get(self.pos + n)
            .cloned()
            .unwrap_or(LexerSpannedToken::new(Token::Eof, Span::default()))
    }

    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(&self.current().token) == std::mem::discriminant(expected)
    }

    fn expect(&mut self, expected: Token) -> Result<LexerSpannedToken> {
        let current = self.current().clone();
        if std::mem::discriminant(&current.token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(current)
        } else {
            let message = format!(
                "Expected {:?} but found {:?}",
                expected, current.token
            );
            Err(self.push_error(current.span, message))
        }
    }

    pub fn parse(&mut self) -> Result<Module> {
        let start_span = self.current().span;

        let name = if self.check(&Token::Module) {
            self.expect(Token::Module)?;
            let path = self.parse_path()?;
            self.expect(Token::Semi)?;
            path
        } else {
            Path {
                segments: vec![Ident {
                    text: "main".to_string(),
                    span: start_span,
                }],
                span: start_span,
            }
        };

        let mut imports = Vec::new();
        while self.check(&Token::Import) {
            imports.push(self.parse_import()?);
        }

        let mut declarations = Vec::new();
        while !self.is_at_end() {
            let errors_before = self.errors.len();
            let pos_before = self.pos;
            match self.parse_declaration() {
                Ok(decl) => declarations.push(decl),
                Err(e) => {
                    self.record_if_new(errors_before, &e);
                    self.synchronize();
                    // Guarantee progress even if nothing was consumed.
                    if self.pos == pos_before && !self.is_at_end() {
                        self.advance();
                    }
                }
            }
        }

        Ok(Module {
            name,
            imports,
            declarations,
            span: start_span,
        })
    }

    /// Error-recovering entry point (additive; [`Parser::parse`] is
    /// unchanged): parses the whole token stream, collecting every
    /// independent syntax error with its file/line/column instead of
    /// aborting at the first one. Returns the module built from all
    /// successfully parsed declarations plus the collected errors
    /// (empty when the file is clean).
    pub fn parse_with_errors(&mut self) -> (Module, Vec<ParseError>) {
        let start_span = self.current().span;

        let name = if self.check(&Token::Module) {
            let errors_before = self.errors.len();
            let pos_before = self.pos;
            let header: Result<Path> = (|| {
                self.expect(Token::Module)?;
                let path = self.parse_path()?;
                self.expect(Token::Semi)?;
                Ok(path)
            })();
            match header {
                Ok(path) => path,
                Err(e) => {
                    self.record_if_new(errors_before, &e);
                    self.synchronize();
                    if self.pos == pos_before && !self.is_at_end() {
                        self.advance();
                    }
                    Path {
                        segments: vec![Ident {
                            text: "main".to_string(),
                            span: start_span,
                        }],
                        span: start_span,
                    }
                }
            }
        } else {
            Path {
                segments: vec![Ident {
                    text: "main".to_string(),
                    span: start_span,
                }],
                span: start_span,
            }
        };

        let mut imports = Vec::new();
        while self.check(&Token::Import) {
            let errors_before = self.errors.len();
            let pos_before = self.pos;
            match self.parse_import() {
                Ok(import) => imports.push(import),
                Err(e) => {
                    self.record_if_new(errors_before, &e);
                    self.synchronize();
                    if self.pos == pos_before && !self.is_at_end() {
                        self.advance();
                    }
                }
            }
        }

        let mut declarations = Vec::new();
        while !self.is_at_end() {
            let errors_before = self.errors.len();
            let pos_before = self.pos;
            match self.parse_declaration() {
                Ok(decl) => declarations.push(decl),
                Err(e) => {
                    self.record_if_new(errors_before, &e);
                    self.synchronize();
                    if self.pos == pos_before && !self.is_at_end() {
                        self.advance();
                    }
                }
            }
        }

        let module = Module {
            name,
            imports,
            declarations,
            span: start_span,
        };
        let errors = self.take_errors();
        (module, errors)
    }

    fn parse_import(&mut self) -> Result<Import> {
        let start = self.current().span;
        self.expect(Token::Import)?;
        let path = self.parse_path()?;
        let alias = if self.check(&Token::As) {
            self.advance();
            Some(self.parse_ident()?)
        } else {
            None
        };
        self.expect(Token::Semi)?;
        Ok(Import {
            path,
            alias,
            span: start,
        })
    }

    fn parse_declaration(&mut self) -> Result<Decl> {
        let mut attrs = Vec::new();
        while self.check(&Token::At) {
            attrs.push(self.parse_attribute()?);
        }
        // `#[...]` directives: `#[cfg(...)]`, `#[allow(...)]`, `#[deprecated]`
        // (Spec §1 + §76). Stored as attributes; `cfg` is evaluated at compile.
        while self.check(&Token::Hash) {
            attrs.push(self.parse_hash_attribute()?);
        }

        let mut i = 0;
        let mut cur = self.peek(i).token.clone();

        // Skip modifiers to see what kind of declaration it is
        while matches!(
            cur,
            Token::Pub
                | Token::Private
                | Token::Protected
                | Token::Async
                | Token::Static
                | Token::Const
        ) {
            i += 1;
            cur = self.peek(i).token.clone();
        }

        match cur {
            Token::Fn => Ok(Decl::Function(self.parse_function(attrs)?)),
            Token::Class => Ok(Decl::Class(self.parse_class(attrs)?)),
            Token::Struct => Ok(Decl::Struct(self.parse_struct(attrs)?)),
            Token::Enum => Ok(Decl::Enum(self.parse_enum(attrs)?)),
            Token::Trait => Ok(Decl::Trait(self.parse_trait(attrs)?)),
            Token::Interface => Ok(Decl::Interface(self.parse_interface(attrs)?)),
            Token::Type => Ok(Decl::TypeAlias(self.parse_type_alias(attrs)?)),
            // Top-level bindings (`let x = …;`, `const X = …;`).
            Token::Let | Token::Var | Token::Const => {
                Ok(Decl::GlobalVar(self.parse_global_var(attrs)?))
            }
            Token::Ident(kw) if kw == "service" && is_service_head(self, i) => {
                Ok(Decl::Service(self.parse_service(attrs)?))
            }
            _ => {
                // Check if it's a global variable (Ident followed by : or =)
                if let Token::Ident(_) = cur {
                    Ok(Decl::GlobalVar(self.parse_global_var(attrs)?))
                } else {
                    let span = self.current().span;
                    let message = format!(
                        "Unexpected header for declaration: {:?}",
                        self.current().token
                    );
                    Err(self.push_error(span, message))
                }
            }
        }
    }

    /// Parse `#[name]` / `#[name(args)]`. `cfg` predicates are encoded as
    /// string-literal args (`feature="x"`, `target="..."`, `test`, ...) so
    /// evaluation needs no expression grammar.
    fn parse_hash_attribute(&mut self) -> Result<Attribute> {
        let start = self.current().span;
        self.expect(Token::Hash)?;
        self.expect(Token::LBracket)?;
        let name = self.parse_attr_name()?;
        let mut args = Vec::new();
        if self.check(&Token::LParen) {
            self.advance();
            if name.text == "cfg" {
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    let key = self.parse_ident()?;
                    let mut pred = key.text.clone();
                    if self.check(&Token::Eq) {
                        self.advance();
                        let token = self.current().token.clone();
                        match token {
                            Token::StringLit(s) => {
                                self.advance();
                                pred.push('=');
                                pred.push_str(&s);
                            }
                            Token::Ident(v) => {
                                self.advance();
                                pred.push('=');
                                pred.push_str(&v);
                            }
                            _ => {
                                let span = self.current().span;
                                let message = format!(
                                    "Expected cfg value, found {:?}",
                                    self.current().token
                                );
                                return Err(self.push_error(span, message));
                            }
                        }
                    }
                    args.push(Expr::Literal(Literal::String(pred)));
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
            } else {
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    args.push(self.parse_expression()?);
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
            }
            self.expect(Token::RParen)?;
        }
        self.expect(Token::RBracket)?;
        Ok(Attribute { name, args, span: start })
    }

    fn parse_attribute(&mut self) -> Result<Attribute> {
        let start = self.current().span;
        self.expect(Token::At)?;
        let name = self.parse_attr_name()?;
        let mut args = Vec::new();
        if self.check(&Token::LParen) {
            self.advance();
            // `@cfg(...)`: encode predicates exactly like `#[cfg(...)]`
            // (`feature="x"` / `test` / `target="..."` as string literals)
            // so `cfg_enabled` handles both attribute forms uniformly.
            if name.text == "cfg" {
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    let key = self.parse_ident()?;
                    let mut pred = key.text.clone();
                    if self.check(&Token::Eq) {
                        self.advance();
                        let token = self.current().token.clone();
                        match token {
                            Token::StringLit(s) => {
                                self.advance();
                                pred.push('=');
                                pred.push_str(&s);
                            }
                            Token::Ident(v) => {
                                self.advance();
                                pred.push('=');
                                pred.push_str(&v);
                            }
                            _ => {
                                let span = self.current().span;
                                let message = format!(
                                    "Expected cfg value, found {:?}",
                                    self.current().token
                                );
                                return Err(self.push_error(span, message));
                            }
                        }
                    }
                    args.push(Expr::Literal(Literal::String(pred)));
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
            } else {
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    args.push(self.parse_expression()?);
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
            }
            self.expect(Token::RParen)?;
        }
        Ok(Attribute {
            name,
            args,
            span: start,
        })
    }

    fn parse_function(&mut self, attrs: Vec<Attribute>) -> Result<Function> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        let is_async = self.check(&Token::Async);
        if is_async {
            self.advance();
        }

        self.expect(Token::Fn)?;
        let name = self.parse_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(Token::RParen)?;
        let where_clause = self.parse_where_clause()?;

        let return_type = if self.check(&Token::RArrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = if self.check(&Token::LBrace) {
            Some(self.parse_block()?)
        } else {
            self.expect(Token::Semi)?;
            None
        };

        Ok(Function {
            attrs,
            visibility,
            is_async,
            name,
            type_params,
            params,
            return_type,
            where_clause,
            preconditions: Vec::new(),
            postconditions: Vec::new(),
            body,
            span: start,
        })
    }

    fn parse_class(&mut self, attrs: Vec<Attribute>) -> Result<Class> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        self.expect(Token::Class)?;
        let name = self.parse_ident()?;
        let type_params = self.parse_type_params()?;
        // `extends Base` + `implements A, B` (Spec §5 composition model).
        let extends = if self.check(&Token::Extends) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let mut implements = Vec::new();
        if self.check(&Token::Implements) {
            self.advance();
            loop {
                implements.push(self.parse_type()?);
                if self.check(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        self.expect(Token::LBrace)?;
        let mut members = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let mut m_attrs = Vec::new();
            while self.check(&Token::At) {
                m_attrs.push(self.parse_attribute()?);
            }

            // Peek forward to see if this is a method or field
            let mut i = 0;
            let mut cur = self.peek(i).token.clone();
            while matches!(
                cur,
                Token::Pub
                    | Token::Private
                    | Token::Protected
                    | Token::Async
                    | Token::Static
                    | Token::Const
                    | Token::Mut
            ) {
                i += 1;
                cur = self.peek(i).token.clone();
            }

            if cur == Token::Fn {
                members.push(ClassMember::Method(self.parse_function(m_attrs)?));
            } else if cur == Token::Constructor {
                members.push(ClassMember::Constructor(self.parse_constructor(m_attrs)?));
            } else {
                members.push(ClassMember::Field(self.parse_field_internal(m_attrs)?));
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Class {
            attrs,
            visibility,
            name,
            type_params,
            extends,
            implements,
            members,
            span: start,
        })
    }

    fn parse_constructor(&mut self, attrs: Vec<Attribute>) -> Result<Constructor> {
        let start = self.current().span;
        self.expect(Token::Constructor)?;
        self.expect(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(Token::RParen)?;
        let body = self.parse_block()?;
        Ok(Constructor {
            attrs,
            visibility: Visibility::Public,
            name: Ident {
                text: "constructor".to_string(),
                span: start,
            },
            params,
            body,
            span: start,
        })
    }

    fn parse_struct(&mut self, attrs: Vec<Attribute>) -> Result<Struct> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        self.expect(Token::Struct)?;
        let name = self.parse_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let mut f_attrs = Vec::new();
            while self.check(&Token::At) {
                f_attrs.push(self.parse_attribute()?);
            }
            let errors_before = self.errors.len();
            match self.parse_field_internal(f_attrs) {
                Ok(f) => fields.push(f),
                Err(e) => {
                    self.record_if_new(errors_before, &e);
                    self.synchronize();
                    if self.check(&Token::Semi) {
                        self.advance();
                    }
                }
            }
            if self.is_at_end() {
                break;
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Struct {
            attrs,
            visibility,
            name,
            type_params,
            fields,
            span: start,
        })
    }

    fn parse_enum(&mut self, attrs: Vec<Attribute>) -> Result<Enum> {
        let start = self.current().span;
        self.expect(Token::Enum)?;
        let name = self.parse_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let vname = self.parse_ident()?;
            variants.push(EnumVariant {
                name: vname,
                data: Vec::new(),
                span: start,
            });
            if !self.check(&Token::RBrace) {
                self.expect(Token::Comma).ok();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Enum {
            attrs,
            visibility: Visibility::Private,
            name,
            type_params,
            variants,
            span: start,
        })
    }

    fn parse_field_internal(&mut self, attrs: Vec<Attribute>) -> Result<Field> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        let is_mutable = self.check(&Token::Mut);
        if is_mutable {
            self.advance();
        }
        let name = self.parse_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        // Field separator: `;` or `,` (both accepted); optional before `}`.
        if self.check(&Token::Semi) || self.check(&Token::Comma) {
            self.advance();
        }
        Ok(Field {
            attrs,
            visibility,
            is_mutable,
            name,
            ty,
            default: None,
            span: start,
        })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        while !self.check(&Token::RParen) {
            let name = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            // Variadic: `name: T...` (Spec §4). Must be last; enforced in typeck.
            let mut is_variadic = false;
            if self.check(&Token::DotDot) {
                self.advance();
                // `...` lexes as `..` + `.`
                if self.check(&Token::Dot) {
                    self.advance();
                }
                is_variadic = true;
            }
            params.push(Param {
                attrs: Vec::new(),
                is_mutable: false,
                name,
                ty,
                is_variadic,
                default: None,
                span: Span::default(),
            });
            if !self.check(&Token::RParen) {
                self.expect(Token::Comma).ok();
            }
        }
        Ok(params)
    }

    fn parse_block(&mut self) -> Result<Block> {
        let start = self.current().span;
        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let errors_before = self.errors.len();
            let pos_before = self.pos;
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    // Block-level recovery (same pattern as module level):
                    // one bad statement never aborts the whole block, so
                    // cascades stay inside the function instead of spraying
                    // "Unexpected header" errors at top level.
                    self.record_if_new(errors_before, &e);
                    self.synchronize();
                    if self.pos == pos_before && !self.is_at_end() {
                        self.advance();
                    }
                }
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Block {
            statements,
            span: start,
        })
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        match &self.current().token {
            Token::If => Ok(Stmt::If(self.parse_if()?)),
            Token::Switch => Ok(Stmt::Switch(self.parse_switch()?)),
            Token::Do => Ok(Stmt::DoWhile(self.parse_do_while()?)),
            Token::Break => {
                let start = self.current().span;
                self.advance();
                // Optional label: `break label;`
                let label = if let Token::Ident(_) = self.current().token {
                    Some(self.parse_ident()?)
                } else {
                    None
                };
                self.expect(Token::Semi).ok();
                Ok(Stmt::Break(BreakStmt { label, span: start }))
            }
            Token::Continue => {
                let start = self.current().span;
                self.advance();
                let label = if let Token::Ident(_) = self.current().token {
                    Some(self.parse_ident()?)
                } else {
                    None
                };
                self.expect(Token::Semi).ok();
                Ok(Stmt::Continue(ContinueStmt { label, span: start }))
            }
            Token::Return => {
                let start = self.current().span;
                self.advance();
                let value = if !self.check(&Token::Semi) && !self.check(&Token::RBrace) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Semi).ok();
                Ok(Stmt::Return(ReturnStmt {
                    value,
                    span: start,
                }))
            }
            Token::Defer => {
                let start = self.current().span;
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::Semi).ok();
                Ok(Stmt::Defer(DeferStmt { expr, span: start }))
            }
            Token::For => Ok(Stmt::For(self.parse_for()?)),
            Token::While => Ok(Stmt::While(self.parse_while()?)),
            Token::Loop => Ok(Stmt::Loop(self.parse_loop()?)),
            Token::Match => Ok(Stmt::Match(self.parse_match()?)),
            Token::Let | Token::Var => Ok(Stmt::Decl(self.parse_decl_stmt()?)),
            Token::Throw => {
                // `throw expr;` desugars to `panic(expr);` — same runtime
                // (`lex_panic`), JS/TS-familiar spelling. Zero downstream
                // changes: reuses Call + Ident machinery.
                let start = self.current().span;
                self.advance();
                let value = self.parse_expression()?;
                self.expect(Token::Semi).ok();
                let callee = Expr::Ident(Ident { text: "panic".to_string(), span: start });
                Ok(Stmt::Expr(ExprStmt {
                    expr: Expr::Call(CallExpr { callee: Box::new(callee), args: vec![value], span: start }),
                    span: start,
                }))
            }
            _ => Ok(Stmt::Expr(self.parse_expr_stmt()?)),
        }
    }

    fn parse_if(&mut self) -> Result<IfStmt> {
        let start = self.current().span;
        self.expect(Token::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        let else_body = if self.check(&Token::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        Ok(IfStmt {
            condition,
            then_branch,
            else_branch: None,
            else_body,
            span: start,
        })
    }

    fn parse_decl_stmt(&mut self) -> Result<DeclStmt> {
        let start = self.current().span;
        let _is_mutable = self.check(&Token::Var);
        self.advance(); // consume let/var

        // Handle 'mut' keyword after let
        if self.check(&Token::Mut) {
            self.advance();
        }

        // Full pattern so tuple destructuring works:
        // `let (a, b) = pair;` / `let (a, b): (int, string) = pair;`
        // (Spec §4 multiple return values).
        let pattern = self.parse_pattern()?;

        // Optional type annotation
        let ty = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        let init = if self.check(&Token::Eq) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.expect(Token::Semi).ok();
        Ok(DeclStmt {
            pattern,
            ty,
            init,
            span: start,
        })
    }

    fn parse_expr_stmt(&mut self) -> Result<ExprStmt> {
        let expr = self.parse_expression()?;
        self.expect(Token::Semi).ok();
        Ok(ExprStmt {
            expr,
            span: Span::default(),
        })
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        // Single funnel for all expression parsing: count nesting here so
        // hostile input (`((((…))))` ten thousand deep) fails cleanly with
        // `MAX_PARSE_DEPTH` instead of overflowing the native stack.
        // Zero-cost in practice (one counter around real work).
        self.depth += 1;
        if self.depth > MAX_PARSE_DEPTH {
            self.depth -= 1;
            let span = self.current().span;
            return Err(self.push_error(
                span,
                format!("expression too deeply nested (limit {})", MAX_PARSE_DEPTH),
            ));
        }
        let out = self.parse_assignment();
        self.depth -= 1;
        out
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let mut expr = self.parse_ternary()?;

        // Handle 'as' cast: expr as Type
        if self.check(&Token::As) {
            self.advance();
            let ty = self.parse_type()?;
            expr = Expr::Cast(CastExpr {
                expr: Box::new(expr),
                ty,
                span: Span::default(),
            });
        }

        if self.check(&Token::Eq) {
            self.advance();
            let value = self.parse_assignment()?;
            // Return as binary assignment
            return Ok(Expr::Binary(BinaryExpr {
                op: BinOp::AddEq, // Simplified - using AddEq as placeholder for '='
                lhs: Box::new(expr),
                rhs: Box::new(value),
                span: Span::default(),
            }));
        }

        Ok(expr)
    }

    fn parse_ternary(&mut self) -> Result<Expr> {
        let expr = self.parse_null_coalesce()?;
        if self.check(&Token::Question) {
            self.advance();
            let then_branch = self.parse_expression()?;
            self.expect(Token::Colon)?;
            let else_branch = self.parse_ternary()?;
            return Ok(Expr::If(Box::new(IfExpr {
                condition: Box::new(expr),
                then_expr: Box::new(then_branch),
                else_expr: Some(Box::new(else_branch)),
                span: Span::default(),
            })));
        }
        Ok(expr)
    }

    fn parse_null_coalesce(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_or()?;
        while self.check(&Token::QuestionQuestion) {
            self.advance();
            let rhs = self.parse_logical_or()?;
            expr = Expr::NullCoalesce(Box::new(expr), Box::new(rhs));
        }
        Ok(expr)
    }

    fn parse_pipe(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_or()?;
        while self.check(&Token::PipeRArrow) {
            let op = BinOp::Pipe;
            self.advance();
            let rhs = self.parse_logical_or()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.parse_logical_and()?;
        while self.check(&Token::OrOr) {
            let op = BinOp::OrOr;
            self.advance();
            let rhs = self.parse_logical_and()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.parse_equality()?;
        while self.check(&Token::AndAnd) {
            let op = BinOp::AndAnd;
            self.advance();
            let rhs = self.parse_equality()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expr> {
        let mut expr = self.parse_comparison()?;
        while self.check(&Token::EqEq) || self.check(&Token::Neq) {
            let op = if self.check(&Token::EqEq) {
                BinOp::EqEq
            } else {
                BinOp::Neq
            };
            self.advance();
            let rhs = self.parse_comparison()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bitor()?;
        while self.check(&Token::Lt)
            || self.check(&Token::LtEq)
            || self.check(&Token::Gt)
            || self.check(&Token::GtEq)
        {
            let op = match self.current().token.clone() {
                Token::Lt => BinOp::Lt,
                Token::LtEq => BinOp::LtEq,
                Token::Gt => BinOp::Gt,
                Token::GtEq => BinOp::GtEq,
                other => {
                    let span = self.current().span;
                    let message = format!("Expected comparison operator, found {:?}", other);
                    return Err(self.push_error(span, message));
                }
            };
            self.advance();
            let rhs = self.parse_bitor()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    // Bitwise precedence (Rust-style, loosest to tightest): `|`, `^`, `&`,
    // then shifts — all sitting between comparison and additive.
    fn parse_bitor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bitxor()?;
        while self.check(&Token::Pipe) {
            self.advance();
            let rhs = self.parse_bitxor()?;
            expr = Expr::Binary(BinaryExpr {
                op: BinOp::BitOr,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_bitxor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bitand()?;
        while self.check(&Token::Caret) {
            self.advance();
            let rhs = self.parse_bitand()?;
            expr = Expr::Binary(BinaryExpr {
                op: BinOp::BitXor,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_bitand(&mut self) -> Result<Expr> {
        let mut expr = self.parse_bitshift()?;
        while self.check(&Token::Ampersand) {
            self.advance();
            let rhs = self.parse_bitshift()?;
            expr = Expr::Binary(BinaryExpr {
                op: BinOp::BitAnd,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_bitshift(&mut self) -> Result<Expr> {
        let mut expr = self.parse_term()?;
        while self.check(&Token::LtLt) || self.check(&Token::GtGt) {
            let op = if self.check(&Token::LtLt) {
                BinOp::Shl
            } else {
                BinOp::Shr
            };
            self.advance();
            let rhs = self.parse_term()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr> {
        let mut expr = self.parse_factor()?;
        while self.check(&Token::Plus) || self.check(&Token::Minus) {
            let op = if self.check(&Token::Plus) {
                BinOp::Add
            } else {
                BinOp::Sub
            };
            self.advance();
            let rhs = self.parse_factor()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr> {
        let mut expr = self.parse_unary()?;
        while self.check(&Token::Star) || self.check(&Token::Slash) || self.check(&Token::Percent) {
            let op = match self.current().token.clone() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                other => {
                    let span = self.current().span;
                    let message = format!("Expected multiplicative operator, found {:?}", other);
                    return Err(self.push_error(span, message));
                }
            };
            self.advance();
            let rhs = self.parse_unary()?;
            expr = Expr::Binary(BinaryExpr {
                op,
                lhs: Box::new(expr),
                rhs: Box::new(rhs),
                span: Span::default(),
            });
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        if self.check(&Token::Bang)
            || self.check(&Token::Minus)
            || self.check(&Token::Await)
            || self.check(&Token::Star)
            || self.check(&Token::Tilde)
        {
            let op = match self.current().token.clone() {
                Token::Bang => UnOp::Not,
                Token::Minus => UnOp::Neg,
                Token::Star => UnOp::Deref,
                Token::Tilde => UnOp::BitNot,
                Token::Await => {
                    self.advance();
                    let expr = self.parse_unary()?;
                    return Ok(Expr::Await(Box::new(AwaitExpr {
                        expr: Box::new(expr),
                        span: Span::default(),
                    })));
                }
                other => {
                    let span = self.current().span;
                    let message = format!("Expected unary operator, found {:?}", other);
                    return Err(self.push_error(span, message));
                }
            };
            self.advance();
            let operand = self.parse_unary()?;
            return Ok(Expr::Unary(UnaryExpr {
                op,
                operand: Box::new(operand),
                span: Span::default(),
            }));
        }
        self.parse_call_member()
    }

    fn parse_call_member(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.check(&Token::LParen) {
                self.advance();
                let mut args = Vec::new();
                while !self.check(&Token::RParen) && !self.is_at_end() {
                    args.push(self.parse_expression()?);
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RParen)?;
                expr = Expr::Call(CallExpr {
                    callee: Box::new(expr),
                    args,
                    span: Span::default(),
                });
            } else if self.check(&Token::LBracket) {
                // Index: `a[i]`, `m["key"]` (Spec §10). Postfix on any
                // primary — arrays, lists, maps, strings.
                self.advance();
                let index = self.parse_expression()?;
                self.expect(Token::RBracket)?;
                expr = Expr::Index(IndexExpr {
                    object: Box::new(expr),
                    index: Box::new(index),
                    span: Span::default(),
                });
            } else if self.check(&Token::Dot) || self.check(&Token::Question) {
                let is_null_safe = self.check(&Token::Question);
                if is_null_safe {
                    self.advance();
                    self.expect(Token::Dot)?;
                } else {
                    self.advance();
                }

                let method = self.parse_ident()?;

                let mut type_args = Vec::new();
                if self.check(&Token::Lt) {
                    self.advance();
                    while !self.check(&Token::Gt) && !self.is_at_end() {
                        type_args.push(self.parse_type()?);
                        if self.check(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::Gt)?;
                }

                if self.check(&Token::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.check(&Token::RParen) && !self.is_at_end() {
                        args.push(self.parse_expression()?);
                        if self.check(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;

                    expr = Expr::MethodCall(MethodCallExpr {
                        object: Box::new(expr),
                        method,
                        type_args,
                        args,
                        span: Span::default(),
                    });
                } else {
                    expr = Expr::FieldAccess(FieldAccessExpr {
                        object: Box::new(expr),
                        field: method,
                        span: Span::default(),
                    });
                }
            } else if self.check(&Token::QuestionQuestion) {
                self.advance();
                let rhs = self.parse_expression()?;
                expr = Expr::NullCoalesce(Box::new(expr), Box::new(rhs));
            } else if self.check(&Token::PathSep) {
                self.advance();
                let name = self.parse_ident()?;
                expr = Expr::FieldAccess(FieldAccessExpr {
                    object: Box::new(expr),
                    field: name,
                    span: Span::default(),
                });
            } else if self.check(&Token::Pipe) {
                // Check for pipe closure: |v| "Aoba! {v}"
                self.advance();
                let mut params = Vec::new();
                while !self.check(&Token::Pipe) && !self.is_at_end() {
                    let name = self.parse_ident()?;
                    params.push(Param {
                        attrs: Vec::new(),
                        is_mutable: false,
                        name,
                        ty: Type::Path(Path::from_ident(&Ident {
                            text: "Any".to_string(),
                            span: Span::default(),
                        })),
                        is_variadic: false,
                        default: None,
                        span: Span::default(),
                    });
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::Pipe)?;
                let body = self.parse_expression()?;
                expr = Expr::Lambda(Lambda {
                    params,
                    return_type: None,
                    body: Box::new(body),
                    span: Span::default(),
                });
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let token = self.current().token.clone();
        match &token {
            Token::Pipe => {
                // Standalone lambda value: `|x| x + 1`, `|a, b| a < b`.
                // (The postfix `expr |x| ...` pipe-closure form below stays
                // for backward compatibility.)
                self.advance();
                let mut params = Vec::new();
                while !self.check(&Token::Pipe) && !self.is_at_end() {
                    let name = self.parse_ident()?;
                    params.push(Param {
                        attrs: Vec::new(),
                        is_mutable: false,
                        name,
                        ty: Type::Path(Path::from_ident(&Ident {
                            text: "Any".to_string(),
                            span: Span::default(),
                        })),
                        is_variadic: false,
                        default: None,
                        span: Span::default(),
                    });
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::Pipe)?;
                let body = self.parse_expression()?;
                Ok(Expr::Lambda(Lambda {
                    params,
                    return_type: None,
                    body: Box::new(body),
                    span: Span::default(),
                }))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Token::LBracket => {
                // Array literal: `[]`, `[a]`, `[a, b]` (Spec §10).
                self.advance();
                let mut elems = Vec::new();
                while !self.check(&Token::RBracket) && !self.is_at_end() {
                    elems.push(self.parse_expression()?);
                    if self.check(&Token::Comma) {
                        self.advance();
                    } else {
                        break;
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::Literal(Literal::Array(elems)))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Literal(Literal::Null))
            }
            Token::IntLit(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Int(n.parse().unwrap_or(0), None)))
            }
            Token::FloatLit(n) => {
                self.advance();
                Ok(Expr::Literal(Literal::Float(n.parse().unwrap_or(0.0), None)))
            }
            Token::StringLit(s) => {
                self.advance();
                if s.contains('{') && s.contains('}') {
                    // Simple heuristic for interpolation
                    let mut parts = Vec::new();
                    let mut last = 0;
                    while let Some(start) = s[last..].find('{') {
                        let actual_start = last + start;
                        if actual_start > last {
                            parts.push(InterpolatedPart::Text(s[last..actual_start].to_string()));
                        }
                        if let Some(end) = s[actual_start..].find('}') {
                            let actual_end = actual_start + end;
                            let expr_str = &s[actual_start + 1..actual_end];
                            // Re-parse the expression inside
                            let mut inner_lexer = lexicon_lexer::Lexer::new(expr_str);
                            let inner_tokens = inner_lexer.tokenize();
                            let mut inner_parser = Parser::new(inner_tokens);
                            if let Ok(inner_expr) = inner_parser.parse_expression() {
                                parts.push(InterpolatedPart::Expr(inner_expr));
                            }
                            last = actual_end + 1;
                        } else {
                            break;
                        }
                    }
                    if last < s.len() {
                        parts.push(InterpolatedPart::Text(s[last..].to_string()));
                    }
                    Ok(Expr::InterpolatedString(parts))
                } else {
                    Ok(Expr::Literal(Literal::String(s.clone())))
                }
            }
            Token::Panic => {
                // `panic` / `recover` lex as keywords but stay callable as
                // ordinary callee identifiers (`panic(e)`, `recover()`).
                let ident = Ident {
                    text: "panic".to_string(),
                    span: self.current().span,
                };
                self.advance();
                Ok(Expr::Ident(ident))
            }
            Token::Recover => {
                let ident = Ident {
                    text: "recover".to_string(),
                    span: self.current().span,
                };
                self.advance();
                Ok(Expr::Ident(ident))
            }
            Token::Ident(id) => {
                let ident = Ident {
                    text: id.clone(),
                    span: self.current().span,
                };
                self.advance();

                // Check for struct literal: Ident { field: value, ... }
                if self.check(&Token::LBrace) {
                    // Peek ahead to see if this looks like a struct literal
                    // (i.e., has "ident : expr" pattern inside)
                    let mut is_struct = false;
                    let look = 1;
                    let t1 = self.peek(look).token.clone();
                    if let Token::Ident(_) = t1 {
                        let t2 = self.peek(look + 1).token.clone();
                        if matches!(t2, Token::Colon) {
                            is_struct = true;
                        }
                    }
                    // Also handle empty struct literal: Ident { }
                    if let Token::RBrace = self.peek(1).token {
                        is_struct = true;
                    }

                    if is_struct {
                        return self.parse_struct_literal(ident);
                    }
                }

                Ok(Expr::Ident(ident))
            }
            Token::LParen => {
                self.advance();
                // Empty tuple / unit: `()`.
                if self.check(&Token::RParen) {
                    self.advance();
                    return Ok(Expr::Literal(Literal::Unit));
                }
                let first = self.parse_expression()?;
                // Tuple literal: `(a, b, ...)` — multiple return values and
                // destructuring sources (Spec §4). Single `(e)` stays grouped.
                if self.check(&Token::Comma) {
                    let mut elems = vec![first];
                    while self.check(&Token::Comma) {
                        self.advance();
                        if self.check(&Token::RParen) {
                            break;
                        }
                        elems.push(self.parse_expression()?);
                    }
                    self.expect(Token::RParen)?;
                    return Ok(Expr::Tuple(elems));
                }
                self.expect(Token::RParen)?;
                Ok(first)
            }
            Token::Unsafe => {
                // Module position wins over block syntax: `unsafe::Sizeof(x)`
                // (the `parse_ident` arm above covers paths/imports).
                if matches!(self.peek(1).token, Token::PathSep) {
                    let span = self.current().span;
                    self.advance();
                    return Ok(Expr::Ident(Ident { text: "unsafe".to_string(), span }));
                }
                let start = self.current().span;
                self.advance();
                let body = self.parse_block()?;
                Ok(Expr::UnsafeBlock(Box::new(body), start))
            }
            _ => {
                let span = self.current().span;
                // Targeted hint for the most common dead-end: JS-style
                // closures (`u => u.x`) have no AST node yet (Onda 2).
                let message = if matches!(token, Token::FatArrow) {
                    "closures (`x => ...`) are not supported yet — use a named `fn` instead".to_string()
                } else {
                    format!("Expected expression, found {:?}", token)
                };
                return Err(self.push_error(span, message));
            }
        }
    }

    fn parse_struct_literal(&mut self, name: Ident) -> Result<Expr> {
        self.expect(Token::LBrace)?;
        let mut args = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let field_name = self.parse_ident()?;

            let value = if self.check(&Token::Colon) {
                self.expect(Token::Colon)?;
                self.parse_expression()?
            } else {
                // Shorthand: User { id } becomes User { id: id }
                Expr::Ident(field_name.clone())
            };

            args.push(value);
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Expr::New(NewExpr {
            ty: Type::Path(Path::from_ident(&name)),
            args,
            span: name.span,
        }))
    }

    /// Full type with postfix slice sugar: `T[]`, `T[][]` (Spec §10).
    /// `?` nullability stays on the atom (existing branches).
    fn parse_type(&mut self) -> Result<Type> {
        // Prefix form `[]T` (same meaning as postfix `T[]`).
        let mut prefix_slices = 0usize;
        while self.check(&Token::LBracket) {
            // Lookahead: only consume if immediately closed (`[]`, not index).
            if matches!(self.peek(1).token, Token::RBracket) {
                self.advance();
                self.advance();
                prefix_slices += 1;
            } else {
                break;
            }
        }
        let mut ty = self.parse_type_atom()?;
        while self.check(&Token::LBracket) {
            self.advance();
            self.expect(Token::RBracket)?;
            ty = Type::Slice(Box::new(ty));
        }
        for _ in 0..prefix_slices {
            ty = Type::Slice(Box::new(ty));
        }
        Ok(ty)
    }

    fn parse_type_atom(&mut self) -> Result<Type> {
        // Function-pointer / callable-reference type (Spec §4):
        // `fn(T1, T2) -> Ret`. Stored as `Type::Function`, usable anywhere
        // a type is (params, locals, return types); values of this type are
        // called through a variable like any callee.
        if self.check(&Token::Fn) || self.check(&Token::Function) {
            self.advance();
            self.expect(Token::LParen)?;
            let mut params = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                params.push(self.parse_type()?);
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RParen)?;
            let ret = if self.check(&Token::RArrow) {
                self.advance();
                self.parse_type()?
            } else {
                Type::Primitive(PrimitiveType::Void)
            };
            let mut ty = Type::Function(params, Box::new(ret));
            if self.check(&Token::Question) {
                self.advance();
                ty = Type::Nullable(Box::new(ty));
            }
            return Ok(ty);
        }
        if self.check(&Token::LParen) {
            self.advance();
            let mut types = Vec::new();
            while !self.check(&Token::RParen) {
                types.push(self.parse_type()?);
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RParen)?;
            let mut ty = Type::Tuple(types);
            if self.check(&Token::Question) {
                self.advance();
                ty = Type::Nullable(Box::new(ty));
            }
            Ok(ty)
        } else {
            let path = self.parse_path()?;

            if self.check(&Token::Lt) {
                self.advance();
                let mut args = Vec::new();
                while !self.check(&Token::Gt) && !self.is_at_end() {
                    args.push(self.parse_type()?);
                    if self.check(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::Gt)?;
                let mut ty = Type::Generic(path, args);
                if self.check(&Token::Question) {
                    self.advance();
                    ty = Type::Nullable(Box::new(ty));
                }
                Ok(ty)
            } else {
                let mut ty = Type::Path(path);
                if self.check(&Token::Question) {
                    self.advance();
                    ty = Type::Nullable(Box::new(ty));
                }
                Ok(ty)
            }
        }
    }

    /// Parse a `a::b::C` path (Spec §12 imports / type paths).
    fn parse_path(&mut self) -> Result<Path> {
        let start = self.current().span;
        let first = self.parse_ident()?;
        let mut segments = vec![first];
        while self.check(&Token::PathSep) {
            self.advance();
            segments.push(self.parse_ident()?);
        }
        let span = segments
            .iter()
            .fold(start, |acc, ident| acc.merge(ident.span));
        Ok(Path { segments, span })
    }

    /// Attribute names may be keywords (`#[inline]` lexes as `Inline`, not
    /// `Ident`), so they need a wider acceptance set than plain identifiers.
    fn parse_attr_name(&mut self) -> Result<Ident> {
        let token = self.current().token.clone();
        let span = self.current().span;
        let text = match token {
            Token::Ident(t) => t,
            Token::Inline => "inline".to_string(),
            Token::Extern => "extern".to_string(),
            Token::Native => "native".to_string(),
            Token::Panic => "panic".to_string(),
            Token::Recover => "recover".to_string(),
            Token::Function => "function".to_string(),
            Token::Dynamic => "dynamic".to_string(),
            Token::Const => "const".to_string(),
            Token::Static => "static".to_string(),
            Token::Mut => "mut".to_string(),
            _ => {
                let span = self.current().span;
                return Err(self.push_error(span, "Expected attribute name".to_string()));
            }
        };
        self.advance();
        Ok(Ident { text, span })
    }

    fn parse_ident(&mut self) -> Result<Ident> {
        // `panic` / `recover` lex as keywords but must remain callable as
        // `panic(e)` / `recover()`; accept them wherever an identifier is
        // expected (callee position, paths).
        // Same for `new`: it lexes as a keyword but has no grammar use as a
        // standalone construct, so `Menu::new()`, `List::new()` and other
        // constructor-style calls accept it as a plain member name.
        let token = self.current().token.clone();
        let span = self.current().span;
        match token {
            Token::Ident(text) => {
                self.advance();
                Ok(Ident { text, span })
            }
            Token::Panic => {
                self.advance();
                Ok(Ident { text: "panic".to_string(), span })
            }
            Token::Recover => {
                self.advance();
                Ok(Ident { text: "recover".to_string(), span })
            }
            Token::New => {
                self.advance();
                Ok(Ident { text: "new".to_string(), span })
            }
            // `unsafe` lexes as a keyword (block starter) but must also work
            // as a module key: `import std::unsafe;` … `unsafe::Sizeof(x)`.
            Token::Unsafe => {
                self.advance();
                Ok(Ident { text: "unsafe".to_string(), span })
            }
            _ => {
                let message = format!("Expected identifier, found {:?}", token);
                Err(self.push_error(span, message))
            }
        }
    }

    fn parse_type_params(&mut self) -> Result<Vec<TypeParam>> {
        let mut params = Vec::new();
        if self.check(&Token::Lt) {
            self.advance();
            while !self.check(&Token::Gt) && !self.is_at_end() {
                let start = self.current().span;
                let name = self.parse_ident()?;
                let mut bounds = Vec::new();
                if self.check(&Token::Colon) {
                    self.advance();
                    while !self.check(&Token::Comma) && !self.check(&Token::Gt) && !self.is_at_end() {
                        bounds.push(self.parse_type()?);
                        if self.check(&Token::Comma) {
                            self.advance();
                        }
                    }
                }
                params.push(TypeParam {
                    name,
                    bounds,
                    span: start,
                });
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::Gt)?;
        }
        Ok(params)
    }

    fn parse_where_clause(&mut self) -> Result<Vec<TypeConstraint>> {
        let mut constraints = Vec::new();
        if self.check(&Token::Where) {
            self.advance();
            while !self.is_at_end() && (self.check(&Token::Lt) || matches!(self.current().token, Token::Ident(_))) {
                let type_param = self.parse_type()?;
                self.expect(Token::Colon)?;
                let mut bounds = Vec::new();
                while !self.check(&Token::Comma) && !self.check(&Token::Semi) && !self.check(&Token::RBrace) && !self.is_at_end() {
                    bounds.push(self.parse_type()?);
                    if self.check(&Token::Ampersand) {
                        self.advance();
                    }
                }
                constraints.push(TypeConstraint {
                    type_param,
                    bounds,
                });
                if self.check(&Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        Ok(constraints)
    }


    fn parse_for(&mut self) -> Result<ForStmt> {
        let start = self.current().span;
        self.expect(Token::For)?;
        let pattern = self.parse_pattern()?;
        self.expect(Token::In)?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(ForStmt {
            pattern,
            iterable,
            body,
            span: start,
        })
    }

    fn parse_while(&mut self) -> Result<WhileStmt> {
        let start = self.current().span;
        self.expect(Token::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(WhileStmt {
            condition,
            body,
            span: start,
        })
    }

    fn parse_loop(&mut self) -> Result<LoopStmt> {
        let start = self.current().span;
        self.expect(Token::Loop)?;
        let body = self.parse_block()?;
        Ok(LoopStmt { body, span: start })
    }

    fn parse_switch(&mut self) -> Result<SwitchStmt> {
        let start = self.current().span;
        self.expect(Token::Switch)?;
        let scrutinee = self.parse_expression()?;
        self.expect(Token::LBrace)?;
        let mut cases = Vec::new();
        let mut default = None;
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.check(&Token::Case) {
                let cstart = self.current().span;
                self.advance();
                let mut exprs = vec![self.parse_expression()?];
                while self.check(&Token::Comma) {
                    self.advance();
                    exprs.push(self.parse_expression()?);
                }
                // Optional guard shared by all case exprs:
                // `case a, b if cond:` (parity with match guards).
                let guard = if self.check(&Token::If) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.expect(Token::Colon)?;
                // Case body: statements until next case/default/RBrace.
                // Braced or bare form both accepted.
                let body = if self.check(&Token::LBrace) {
                    self.parse_block()?
                } else {
                    let bstart = self.current().span;
                    let mut statements = Vec::new();
                    while !self.check(&Token::Case)
                        && !self.check(&Token::Default)
                        && !self.check(&Token::RBrace)
                        && !self.is_at_end()
                    {
                        statements.push(self.parse_statement()?);
                    }
                    Block { statements, span: bstart }
                };
                cases.push(SwitchCase { exprs, guard, body, span: cstart });
            } else if self.check(&Token::Default) {
                self.advance();
                self.expect(Token::Colon)?;
                let body = if self.check(&Token::LBrace) {
                    self.parse_block()?
                } else {
                    let bstart = self.current().span;
                    let mut statements = Vec::new();
                    while !self.check(&Token::Case)
                        && !self.check(&Token::Default)
                        && !self.check(&Token::RBrace)
                        && !self.is_at_end()
                    {
                        statements.push(self.parse_statement()?);
                    }
                    Block { statements, span: bstart }
                };
                default = Some(body);
            } else {
                let span = self.current().span;
                let message = format!(
                    "Expected 'case' or 'default' in switch, found {:?}",
                    self.current().token
                );
                self.push_error(span, message);
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(SwitchStmt { scrutinee, cases, default, span: start })
    }

    fn parse_do_while(&mut self) -> Result<DoWhileStmt> {
        let start = self.current().span;
        self.expect(Token::Do)?;
        let body = self.parse_block()?;
        self.expect(Token::While)?;
        let condition = self.parse_expression()?;
        self.expect(Token::Semi).ok();
        Ok(DoWhileStmt { body, condition, span: start })
    }

    fn parse_match(&mut self) -> Result<MatchStmt> {        let start = self.current().span;
        self.expect(Token::Match)?;
        let expr = self.parse_expression()?;
        self.expect(Token::LBrace)?;
        let mut arms = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            // Optional guard: `pattern if cond => body` (Spec §3).
            let guard = if self.check(&Token::If) {
                self.advance();
                Some(self.parse_expression()?)
            } else {
                None
            };
            self.expect(Token::FatArrow)?;
            // Arm bodies accept tail expressions as well as `return <expr>`
            // (implicit tail return in statement position), `break` and
            // `continue` (desugared to unit).
            let body = if self.check(&Token::Return) {
                self.advance();
                if self.check(&Token::Comma) || self.check(&Token::RBrace) {
                    Expr::Literal(Literal::Unit)
                } else {
                    self.parse_expression()?
                }
            } else if self.check(&Token::Break) || self.check(&Token::Continue) {
                self.advance();
                Expr::Literal(Literal::Unit)
            } else {
                self.parse_expression()?
            };
            arms.push(MatchArm {
                pattern,
                guard,
                body,
                span: start,
            });
            if self.check(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(MatchStmt {
            expr,
            arms,
            span: start,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        let start = self.current().span;
        if self.check(&Token::Underscore) {
            self.advance();
            return Ok(Pattern::Wildcard(start));
        }

        if self.check(&Token::LParen) {
            self.advance();
            let mut patterns = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                patterns.push(self.parse_pattern()?);
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RParen)?;
            return Ok(Pattern::Tuple(patterns));
        }

        if self.check(&Token::LBracket) {
            self.advance();
            let mut patterns = Vec::new();
            while !self.check(&Token::RBracket) && !self.is_at_end() {
                patterns.push(self.parse_pattern()?);
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RBracket)?;
            return Ok(Pattern::Array(patterns));
        }

        if self.check(&Token::LBrace) {
            self.advance();
            let mut fields = Vec::new();
            while !self.check(&Token::RBrace) && !self.is_at_end() {
                let field = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let pattern = self.parse_pattern()?;
                fields.push(FieldPattern { field, pattern });
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RBrace)?;
            return Ok(Pattern::Object(fields));
        }

        // Check for Literal patterns (Int, Float, String, Bool)
        let token = self.current().token.clone();
        if matches!(token, Token::IntLit(_) | Token::FloatLit(_) | Token::StringLit(_) | Token::True | Token::False) {
            self.advance();
            let lit = match token {
                Token::IntLit(n) => Literal::Int(n.parse().unwrap_or(0), None),
                Token::FloatLit(n) => Literal::Float(n.parse().unwrap_or(0.0), None),
                Token::StringLit(s) => Literal::String(s),
                Token::True => Literal::Bool(true),
                Token::False => Literal::Bool(false),
                other => {
                    let span = self.current().span;
                    let message = format!("Expected literal pattern, found {:?}", other);
                    return Err(self.push_error(span, message));
                }
            };
            return Ok(Pattern::Literal(lit));
        }

        let ident = self.parse_ident()?;
        
        // Check for Enum Variant Pattern: Ident(pattern, ...)
        if self.check(&Token::LParen) {
            self.advance();
            let mut patterns = Vec::new();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                patterns.push(self.parse_pattern()?);
                if self.check(&Token::Comma) {
                    self.advance();
                }
            }
            self.expect(Token::RParen)?;
            return Ok(Pattern::EnumVariant(ident, patterns));
        }

        // Check for Named Pattern: ident @ pattern
        if self.check(&Token::At) {
            self.advance();
            let pattern = self.parse_pattern()?;
            return Ok(Pattern::Named(ident, Box::new(pattern)));
        }

        Ok(Pattern::Ident(ident))
    }

    fn parse_trait(&mut self, attrs: Vec<Attribute>) -> Result<Trait> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        self.expect(Token::Trait)?;
        let name = self.parse_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            methods.push(self.parse_method_sig()?);
        }
        self.expect(Token::RBrace)?;
        Ok(Trait {
            attrs,
            visibility,
            name,
            type_params,
            extends: Vec::new(),
            methods,
            span: start,
        })
    }

    fn parse_interface(&mut self, attrs: Vec<Attribute>) -> Result<Interface> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        self.expect(Token::Interface)?;
        let name = self.parse_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(Token::LBrace)?;
        let mut methods = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            methods.push(self.parse_method_sig()?);
        }
        self.expect(Token::RBrace)?;
        Ok(Interface {
            attrs,
            visibility,
            name,
            type_params,
            extends: Vec::new(),
            methods,
            span: start,
        })
    }

    fn parse_type_alias(&mut self, attrs: Vec<Attribute>) -> Result<TypeAlias> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        self.expect(Token::Type)?;
        let name = self.parse_ident()?;
        let type_params = self.parse_type_params()?;
        self.expect(Token::Eq)?;
        let ty = self.parse_type()?;
        self.expect(Token::Semi)?;
        Ok(TypeAlias {
            attrs,
            visibility,
            name,
            type_params,
            ty,
            span: start,
        })
    }

    /// Lookahead for an RPC service header: `service Name {`.
    /// `i` is the offset (past modifiers) of the `service` identifier.
    /// Conservative on purpose: anything else falls through to the
    /// global-variable path, so existing code is unaffected.
    fn parse_service(&mut self, attrs: Vec<Attribute>) -> Result<ServiceDecl> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        // consume the `service` keyword-identifier itself
        let kw = self.parse_ident()?;
        debug_assert_eq!(kw.text, "service");
        let name = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut rpcs = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let rstart = self.current().span;
            match self.current().token.clone() {
                Token::Ident(kw) if kw == "rpc" => {
                    self.advance();
                }
                other => {
                    let span = self.current().span;
                    let message = format!(
                        "Expected `rpc` declaration in service body, found {:?}",
                        other
                    );
                    return Err(self.push_error(span, message));
                }
            }
            let rname = self.parse_ident()?;
            self.expect(Token::LParen)?;
            let params = self.parse_param_list()?;
            self.expect(Token::RParen)?;
            let return_type = if self.check(&Token::RArrow) {
                self.advance();
                Some(self.parse_type()?)
            } else {
                None
            };
            // `;` or `,` separator, optional before `}`.
            if self.check(&Token::Semi) || self.check(&Token::Comma) {
                self.advance();
            }
            rpcs.push(ServiceRpc {
                name: rname,
                params,
                return_type,
                span: rstart,
            });
        }
        self.expect(Token::RBrace)?;
        Ok(ServiceDecl {
            attrs,
            visibility,
            name,
            rpcs,
            span: start,
        })
    }

    fn parse_global_var(&mut self, attrs: Vec<Attribute>) -> Result<GlobalVar> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        let is_static = self.check(&Token::Static);
        if is_static {
            self.advance();
        }
        // Top-level `let`/`var`/`const` bindings are global vars
        // (`let x = …;` at file scope, not just bare `x: T = …;`).
        // `const` implies immutable; `let`/`var` imply mutable.
        let mut is_mutable = self.check(&Token::Mut);
        if is_mutable {
            self.advance();
        }
        if self.check(&Token::Const) {
            self.advance();
        } else if self.check(&Token::Let) || self.check(&Token::Var) {
            self.advance();
            is_mutable = true;
        }
        let name = self.parse_ident()?;
        let ty = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.check(&Token::Eq) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        if !self.check(&Token::Semi) {
            let span = self.current().span;
            return Err(self.push_error(
                span,
                "expected `;` after global variable declaration (is the previous line missing `;`?)".to_string(),
            ));
        }
        self.expect(Token::Semi)?;
        Ok(GlobalVar {
            attrs,
            visibility,
            is_mutable,
            is_static,
            name,
            ty: ty.unwrap_or(Type::Primitive(PrimitiveType::Dynamic)),
            init,
            span: start,
        })
    }

    fn parse_method_sig(&mut self) -> Result<MethodSig> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) {
            self.advance();
            Visibility::Public
        } else {
            Visibility::Private
        };
        let is_async = self.check(&Token::Async);
        if is_async {
            self.advance();
        }
        self.expect(Token::Fn)?;
        let name = self.parse_ident()?;
        self.expect(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(Token::RParen)?;
        let return_type = if self.check(&Token::RArrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(Token::Semi)?;
        Ok(MethodSig {
            attrs: Vec::new(),
            visibility,
            is_async,
            name,
            type_params: Vec::new(),
            params,
            return_type,
            default_body: None,
            span: start,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lexicon_lexer::Lexer;

    #[test]
    fn test_parse_module() {
        let source = "module test; import std::io; fn main() {}";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let module = parser.parse().unwrap();

        assert_eq!(module.name.to_string(), "test");
        assert_eq!(module.imports.len(), 1);
        assert_eq!(module.declarations.len(), 1);
    }

    #[test]
    fn test_parse_struct() -> std::result::Result<(), String> {
        let source = "pub struct Point { x: int, y: int }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Struct(s) = decl {
            assert_eq!(s.name.text, "Point");
            assert_eq!(s.fields.len(), 2);
            assert_eq!(s.fields[0].name.text, "x");
            assert_eq!(s.fields[1].name.text, "y");
        } else {
            return Err("Expected struct declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_trait() -> std::result::Result<(), String> {
        let source = "trait Printable { fn print(); }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Trait(t) = decl {
            assert_eq!(t.name.text, "Printable");
            assert_eq!(t.methods.len(), 1);
            assert_eq!(t.methods[0].name.text, "print");
        } else {
            return Err("Expected trait declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_match() -> std::result::Result<(), String> {
        let source = "fn test(x: int) { match x { 1 => return true, _ => return false } }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            assert_eq!(body.statements.len(), 1);
            if let Stmt::Match(m) = &body.statements[0] {
                assert_eq!(m.arms.len(), 2);
            } else {
                return Err("Expected match statement".to_string());
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_switch() -> std::result::Result<(), String> {
        let source = "fn test(x: int) { switch x { case 1: break; case 2, 3: break; default: break; } }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            assert_eq!(body.statements.len(), 1);
            if let Stmt::Switch(s) = &body.statements[0] {
                assert_eq!(s.cases.len(), 2);
                assert_eq!(s.cases[1].exprs.len(), 2);
                assert!(s.default.is_some());
            } else {
                return Err("Expected switch statement".to_string());
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_break_continue_do_while() -> std::result::Result<(), String> {        let source = "fn test() { loop { break; continue; } do { break; } while true; }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            assert_eq!(body.statements.len(), 2);
            assert!(matches!(body.statements[0], Stmt::Loop(_)));
            assert!(matches!(body.statements[1], Stmt::DoWhile(_)));
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_variadic() -> std::result::Result<(), String> {
        let source = "fn sum(first: int, rest: int...) -> int { return first; }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            assert_eq!(f.params.len(), 2);
            assert!(!f.params[0].is_variadic);
            assert!(f.params[1].is_variadic);
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_match_guard() -> std::result::Result<(), String> {
        let source = "fn test(x: int) { match x { n if n > 0 => true, _ => false } }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            if let Stmt::Match(m) = &body.statements[0] {
                assert_eq!(m.arms.len(), 2);
                assert!(m.arms[0].guard.is_some());
                assert!(m.arms[1].guard.is_none());
            } else {
                return Err("Expected match statement".to_string());
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_unsafe_block() -> std::result::Result<(), String> {
        let source = "fn test() { unsafe { return; } }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            if let Stmt::Expr(e) = &body.statements[0] {
                assert!(matches!(e.expr, Expr::UnsafeBlock(_, _)));
            } else {
                return Err("Expected unsafe block expression".to_string());
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_cfg_attribute() -> std::result::Result<(), String> {
        let source = "#[cfg(feature = \"net\")] fn serve() {}";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            assert_eq!(f.attrs.len(), 1);
            assert_eq!(f.attrs[0].name.text, "cfg");
            assert_eq!(f.attrs[0].args.len(), 1);
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_class_extends_implements() -> std::result::Result<(), String> {
        let source = "class Dog extends Animal implements Loud, Friendly { }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Class(c) = decl {
            assert!(c.extends.is_some());
            assert_eq!(c.implements.len(), 2);
        } else {
            return Err("Expected class declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_at_cfg_form() -> std::result::Result<(), String> {
        // `@cfg(...)` must encode the same string predicates as `#[cfg(...)]`.
        // NOTE: tokens are built by hand because the lexer currently has no
        // `@` arm (`@` scans as `Token::Error`), so `@`-forms are only
        // reachable with a `Token::At` in the stream. Parser support is kept
        // so the form works once the lexer emits `At`.
        use lexicon_lexer::tokens::SpannedToken;
        let tok = |t: Token| SpannedToken::new(t, Span::default());
        let tokens = vec![
            tok(Token::At),
            tok(Token::Ident("cfg".to_string())),
            tok(Token::LParen),
            tok(Token::Ident("feature".to_string())),
            tok(Token::Eq),
            tok(Token::StringLit("net".to_string())),
            tok(Token::RParen),
            tok(Token::Fn),
            tok(Token::Ident("serve".to_string())),
            tok(Token::LParen),
            tok(Token::RParen),
            tok(Token::LBrace),
            tok(Token::RBrace),
            tok(Token::Eof),
        ];
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            assert_eq!(f.attrs.len(), 1);
            assert_eq!(f.attrs[0].name.text, "cfg");
            match &f.attrs[0].args[0] {
                Expr::Literal(Literal::String(p)) => assert_eq!(p, "feature=net"),
                other => return Err(format!("Expected cfg string pred, got {:?}", other)),
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_cfg_test_and_target() -> std::result::Result<(), String> {
        for source in [
            "#[cfg(test)] fn a() {}",
            "#[cfg(target = \"wasm\")] fn b() {}",
        ] {
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let decl = parser.parse_declaration().unwrap();
            if let Decl::Function(f) = decl {
                assert_eq!(f.attrs[0].name.text, "cfg");
                assert_eq!(f.attrs[0].args.len(), 1);
            } else {
                return Err("Expected function declaration".to_string());
            }
        }
        Ok(())
    }

    #[test]
    fn test_parse_tuple_let_destructuring() -> std::result::Result<(), String> {
        let source = "fn f() { let (a, b): (int, string) = pair; }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            if let Stmt::Decl(d) = &body.statements[0] {
                assert!(matches!(d.pattern, Pattern::Tuple(_)));
                assert!(d.ty.is_some());
                assert!(d.init.is_some());
            } else {
                return Err("Expected decl statement".to_string());
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_fn_pointer_type() -> std::result::Result<(), String> {
        let source = "fn f(cb: fn(int) -> int) -> int { return 1; }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            assert!(matches!(f.params[0].ty, Type::Function(_, _)));
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_tuple_literal() -> std::result::Result<(), String> {
        let source = "fn f() { let t = (1, 2); }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            if let Stmt::Decl(d) = &body.statements[0] {
                assert!(matches!(d.init, Some(Expr::Tuple(_))));
            } else {
                return Err("Expected decl statement".to_string());
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_switch_guard() -> std::result::Result<(), String> {
        let source = "fn test(x: int) { switch x { case 1 if x > 0: break; default: break; } }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            if let Stmt::Switch(s) = &body.statements[0] {
                assert_eq!(s.cases.len(), 1);
                assert!(s.cases[0].guard.is_some());
            } else {
                return Err("Expected switch statement".to_string());
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_panic_recover_calls() -> std::result::Result<(), String> {
        let source = "fn f() { panic(\"boom\"); recover(); }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            let body = f.body.unwrap();
            assert_eq!(body.statements.len(), 2);
            for stmt in &body.statements {
                if let Stmt::Expr(e) = stmt {
                    assert!(matches!(e.expr, Expr::Call(_)));
                } else {
                    return Err("Expected expr statement".to_string());
                }
            }
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_abi_inline_attrs() -> std::result::Result<(), String> {
        let source = "#[abi(\"C\")] #[inline] fn f() {}";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Function(f) = decl {
            assert_eq!(f.abi().as_deref(), Some("C"));
            assert!(f.is_inline());
        } else {
            return Err("Expected function declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_with_errors_collects_three_errors_and_keeps_valid_decls(
    ) -> std::result::Result<(), String> {
        // Three independent single-line syntax errors on lines 2, 4 and 6.
        // Each is crafted so panic-free synchronization lands exactly on
        // the next declaration (no stray-brace residue -> exactly 1 error
        // per broken declaration).
        let source = [
            "fn good_one() {}",
            "fn broken_one( ;",
            "fn good_two() {}",
            "type BrokenTwo = ;",
            "fn good_three() {}",
            "fn broken_three : int ;",
        ]
        .join("\n");
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::with_file(tokens, "three_errors.lex");
        let (module, errors) = parser.parse_with_errors();

        if errors.len() != 3 {
            return Err(format!("expected 3 errors, got {}: {:?}", errors.len(), errors));
        }
        let lines: Vec<usize> = errors.iter().map(|e| e.line).collect();
        if lines != vec![2, 4, 6] {
            return Err(format!("expected error lines [2, 4, 6], got {:?}", lines));
        }
        for e in &errors {
            if e.file != "three_errors.lex" {
                return Err(format!("expected file three_errors.lex, got {:?}", e.file));
            }
            if e.column == 0 || e.message.is_empty() {
                return Err(format!("error missing column/message: {:?}", e));
            }
        }
        if module.declarations.len() != 3 {
            return Err(format!(
                "expected 3 valid declarations, got {}",                module.declarations.len()
            ));
        }
        let names: Vec<String> = module
            .declarations
            .iter()
            .map(|d| match d {
                Decl::Function(f) => f.name.text.clone(),
                other => format!("unexpected {:?}", std::mem::discriminant(other)),
            })
            .collect();
        if names != vec!["good_one".to_string(), "good_two".to_string(), "good_three".to_string()] {
            return Err(format!("valid declarations lost or reordered: {:?}", names));
        }
        // `parse_with_errors` drains the buffer into the return value.
        if !parser.errors().is_empty() {
            return Err("expected drained error buffer after parse_with_errors".to_string());
        }

        // Legacy entry point keeps its contract: Ok(module) + errors via
        // `into_errors` (backward compatible `Vec<Error>`).
        let mut lexer2 = Lexer::new(&source);
        let tokens2 = lexer2.tokenize();
        let mut parser2 = Parser::new(tokens2);
        let module2 = parser2
            .parse()
            .map_err(|e| format!("legacy parse failed: {}", e))?;
        if module2.declarations.len() != 3 {
            return Err("legacy parse lost valid declarations".to_string());
        }
        if parser2.into_errors().len() != 3 {
            return Err("legacy into_errors did not report 3 errors".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_garbage_never_panics_and_yields_errors() -> std::result::Result<(), String> {        for source in [
            "@@@ ::: ;;; {{{",
            "$$$ %%% }{{{{ \"unterminated",
            ";;; ;;; ;;;",
            "{{{{{{{{",
        ] {
            let mut lexer = Lexer::new(source);
            let tokens = lexer.tokenize();
            // Recovering entry point: must terminate with >= 1 error.
            let mut parser = Parser::new(tokens.clone());
            let (_module, errors) = parser.parse_with_errors();
            if errors.is_empty() {
                return Err(format!("expected errors for garbage input {:?}", source));
            }
            // Legacy entry point: may return Ok or Err, but must not panic.
            let mut parser2 = Parser::new(tokens);
            let _ = parser2.parse();
        }
        Ok(())
    }

    #[test]
    fn test_comments_are_trivia_everywhere() -> std::result::Result<(), String> {        let source = "// leading\n/* block\nspanning */\npub fn f() -> int {\n    // inside\n    return 1; // trailing\n}\n// trailing";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (_module, errors) = parser.parse_with_errors();
        if !errors.is_empty() {
            return Err(format!("comments must not error: {:?}", errors));
        }
        Ok(())
    }

    #[test]
    fn test_top_level_let_const_var_are_global_vars() -> std::result::Result<(), String> {
        let source = "let a = 1;\nvar b = 2;\nconst C = 3;\nplain: int = 4;\n";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (module, errors) = parser.parse_with_errors();
        if !errors.is_empty() {
            return Err(format!("top-level bindings must parse: {:?}", errors));
        }
        if module.declarations.len() != 4 {
            return Err(format!("expected 4 decls, got {}", module.declarations.len()));
        }
        Ok(())
    }

    #[test]
    fn test_throw_desugars_to_panic_call() -> std::result::Result<(), String> {        let source = "pub fn f() -> void {\n    throw \"boom\";\n}\n";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (_module, errors) = parser.parse_with_errors();
        if !errors.is_empty() {
            return Err(format!("throw must parse: {:?}", errors));
        }
        Ok(())
    }

    #[test]
    fn test_slice_type_and_array_literal() -> std::result::Result<(), String> {
        let source = "const db: User[] = [];\nlet xs: int[][] = [];\nlet ys = [1, 2];\n";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (_module, errors) = parser.parse_with_errors();
        if !errors.is_empty() {
            return Err(format!("slice/array must parse: {:?}", errors));
        }
        Ok(())
    }

    #[test]
    fn test_debug_cascade_probe_prints_all_errors() {
        // Recovery quality gate: ONE root error, zero cascade noise.
        let prelude = "import core::io::Console;\nimport core::net::Http;\nimport core::collections::List;\nimport core::json::Json;\nimport core::env::Env;\n";
        let body = "pub fn f() -> void {\n    if (db.find(u => u.x)) {\n        throw \"a\";\n    }\n    let y = 1;\n}\npub fn g() -> void {\n}\n";
        let source = format!("{}{}", prelude, body);
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let (module, errors) = parser.parse_with_errors();
        assert_eq!(errors.len(), 1, "expected exactly 1 error, got {:?}", errors);
        assert_eq!(errors[0].line, 7, "wrong line: {:?}", errors[0]);
        assert_eq!(module.declarations.len(), 2);
    }

    #[test]
    fn test_parse_service() -> std::result::Result<(), String> {
        let source = "pub service UserService { rpc GetUser(id: int) -> User; rpc ListUsers() -> User; }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Service(s) = decl {
            assert_eq!(s.name.text, "UserService");
            assert_eq!(s.rpcs.len(), 2);
            assert_eq!(s.rpcs[0].name.text, "GetUser");
            assert_eq!(s.rpcs[0].params.len(), 1);
            assert_eq!(s.rpcs[0].params[0].name.text, "id");
            assert!(s.rpcs[0].return_type.is_some());
            assert_eq!(s.rpcs[1].name.text, "ListUsers");
            assert!(s.rpcs[1].return_type.is_some());
        } else {
            return Err("Expected service declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_parse_service_no_return() -> std::result::Result<(), String> {
        let source = "service Events { rpc Publish(topic: String); }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::Service(s) = decl {
            assert_eq!(s.name.text, "Events");
            assert_eq!(s.rpcs.len(), 1);
            assert!(s.rpcs[0].return_type.is_none());
        } else {
            return Err("Expected service declaration".to_string());
        }
        Ok(())
    }

    #[test]
    fn test_service_named_global_still_parses() -> std::result::Result<(), String> {
        // `service` as an ordinary global name must keep working.
        let source = "let service = 1;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let decl = parser.parse_declaration().unwrap();

        if let Decl::GlobalVar(g) = decl {
            assert_eq!(g.name.text, "service");
        } else {
            return Err("Expected global variable".to_string());
        }
        Ok(())
    }
}

