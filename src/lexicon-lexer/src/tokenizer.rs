use crate::tokens::{SpannedToken, Token};
use lexicon_core::span::Span;

/// A single lexing diagnostic collected without aborting tokenization.
///
/// `line` / `column` are 1-based source locations pointing at the start of
/// the offending construct (for a bad escape, at the escaped character).
/// `message` is a human-readable description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl LexError {
    pub fn new(line: usize, column: usize, message: impl Into<String>) -> Self {
        LexError {
            line,
            column,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for LexError {}

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    position: usize,
    line: usize,
    column: usize,
    start: usize,
    errors: Vec<LexError>,
    trivia_table: Vec<Vec<String>>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut chars = source.char_indices().peekable();
        // Skip a UTF-8 BOM so `lex run` accepts BOM-prefixed files.
        let mut position = 0;
        if let Some(&(0, '\u{FEFF}')) = chars.peek() {
            chars.next();
            position = 3;
        }
        Lexer {
            source,
            chars,
            position,
            line: 1,
            column: 1,
            start: 0,
            errors: Vec::new(),
            trivia_table: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        self.errors.clear();
        self.trivia_table.clear();
        let mut tokens = Vec::new();
        // Comments seen since the last non-comment token. They are kept as
        // regular Comment/BlockComment tokens in the stream AND recorded as
        // leading trivia for the next non-comment token.
        let mut pending_trivia: Vec<String> = Vec::new();

        while let Some((pos, ch)) = self.chars.next() {
            self.start = pos;
            self.position = pos + ch.len_utf8();
            let start_line = self.line;
            let start_column = self.column;

            // The outer iterator consumes the first character directly, so keep
            // the source position in sync before scanners consume additional chars.
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }

            let token = match ch {
                ' ' | '\t' | '\n' | '\r' => continue,
                '/' => self.scan_slash(start_line, start_column),
                '"' => self.scan_string(start_line, start_column),
                '\'' => self.scan_char(start_line, start_column),
                '0'..='9' => self.scan_number(pos, start_line, start_column),
                'a'..='z' | 'A'..='Z' | '_' => self.scan_ident(pos),
                '(' => Token::LParen,
                ')' => Token::RParen,
                '[' => Token::LBracket,
                ']' => Token::RBracket,
                '{' => Token::LBrace,
                '}' => Token::RBrace,
                '<' => self.scan_lt(),
                '>' => self.scan_gt(),
                '=' => self.scan_eq(),
                '!' => self.scan_bang(),
                '&' => self.scan_amp(),
                '|' => self.scan_pipe(),
                '+' => self.scan_plus(),
                '-' => self.scan_minus(),
                '*' => self.scan_star(),
                '%' => self.scan_percent(),
                '^' => Token::Caret,
                '~' => Token::Tilde,
                '?' => self.scan_question(),
                ':' => self.scan_colon(),
                ',' => Token::Comma,
                '.' => self.scan_dot(),
                ';' => Token::Semi,
                '#' => Token::Hash,
                '@' => Token::At,
                '▷' => Token::PipeRArrow,
                _ => {
                    let msg = format!("Unexpected character: {}", ch);
                    self.errors.push(LexError {
                        line: start_line,
                        column: start_column,
                        message: msg.clone(),
                    });
                    Token::Error(msg)
                }
            };

            // Trivia bookkeeping: must not change the Token stream itself.
            // Comment tokens get an empty leading-trivia entry and are
            // accumulated; any other token drains the accumulation.
            match &token {
                Token::Comment(c) | Token::BlockComment(c) => {
                    self.trivia_table.push(Vec::new());
                    pending_trivia.push(c.clone());
                }
                _ => {
                    let leading = std::mem::take(&mut pending_trivia);
                    self.trivia_table.push(leading);
                }
            }

            tokens.push(SpannedToken {
                token,
                span: Span::with_location(
                    self.start,
                    self.position,
                    start_line,
                    start_column,
                ),
            });
        }

        // Trailing comments attach to Eof so no trivia is lost.
        let trailing = std::mem::take(&mut pending_trivia);
        self.trivia_table.push(trailing);
        tokens.push(SpannedToken {
            token: Token::Eof,
            span: Span::new(self.source.len(), self.source.len()),
        });

        tokens
    }

    /// Additive multi-diagnostic entry point.
    ///
    /// Behaves exactly like [`Lexer::tokenize`] for the returned token stream
    /// (byte-identical for valid inputs) but additionally returns every
    /// [`LexError`] collected while scanning (unterminated strings, bad
    /// escapes, invalid numerics, unexpected characters, ...). Lexing never
    /// fails fast: on error it records the diagnostic, skips to a safe
    /// resumption point and continues tokenizing.
    pub fn tokenize_with_errors(&mut self) -> (Vec<SpannedToken>, Vec<LexError>) {
        let tokens = self.tokenize();
        let errors = self.errors.clone();
        (tokens, errors)
    }

    /// Collected diagnostics from the last [`Lexer::tokenize`] call.
    pub fn errors(&self) -> &[LexError] {
        &self.errors
    }

    /// Leading trivia (preceding `//...` / `/*...*/` comment bodies, in
    /// source order) for the token at `idx`. Returns an empty vec when out
    /// of bounds or when the token has no preceding comments.
    pub fn leading_trivia(&self, idx: usize) -> Vec<String> {
        self.trivia_table.get(idx).cloned().unwrap_or_default()
    }

    /// Full table parallel to the last token stream: `table[i]` holds the
    /// leading comments for token `i`. Built during [`Lexer::tokenize`].
    pub fn trivia_table(&self) -> Vec<Vec<String>> {
        self.trivia_table.clone()
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, ch)) = result {
            self.position = pos + ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        result
    }

    fn peek(&mut self) -> Option<&(usize, char)> {
        self.chars.peek()
    }

    fn scan_string(&mut self, start_line: usize, start_column: usize) -> Token {
        let mut s = String::new();
        let mut in_string = true;

        while let Some(&(_, ch)) = self.peek() {
            if ch == '"' {
                self.advance();
                in_string = false;
                break;
            }
            // Safe resumption: an unescaped line break terminates the scan
            // without consuming the newline, so the outer loop can continue
            // on the next line and remaining tokens stay intact.
            if ch == '\n' || ch == '\r' {
                break;
            }
            if ch == '\\' {
                self.advance(); // consume backslash
                if let Some(&(_, esc)) = self.peek() {
                    // Position of the escaped char (column right after `\`).
                    let esc_line = self.line;
                    let esc_col = self.column;
                    let escaped = match esc {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        _ => {
                            let msg = format!("bad escape sequence '\\{}'", esc);
                            self.errors.push(LexError {
                                line: esc_line,
                                column: esc_col,
                                message: msg,
                            });
                            esc
                        }
                    };
                    s.push(escaped);
                    self.advance();
                } else {
                    // Backslash at EOF: unterminated.
                    break;
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }

        if in_string {
            let msg = "Unterminated string".to_string();
            self.errors.push(LexError {
                line: start_line,
                column: start_column,
                message: msg.clone(),
            });
            Token::Error(msg)
        } else {
            Token::StringLit(s)
        }
    }

    fn scan_char(&mut self, start_line: usize, start_column: usize) -> Token {
        if let Some(&(_, ch)) = self.peek() {
            self.advance();
            if let Some(&(_, '\'')) = self.peek() {
                self.advance();
                return Token::CharLit(ch);
            }
        }
        let msg = "Unterminated char".to_string();
        self.errors.push(LexError {
            line: start_line,
            column: start_column,
            message: msg.clone(),
        });
        Token::Error(msg)
    }

    fn scan_number(&mut self, start: usize, start_line: usize, start_column: usize) -> Token {
        // Integer bases required by the language specification.
        if self.source[start..].starts_with("0x") || self.source[start..].starts_with("0X") {
            self.advance(); // x/X
            let mut digits = 0usize;
            while let Some(&(_, ch)) = self.peek() {
                if ch.is_ascii_hexdigit() {
                    digits += 1;
                    self.advance();
                } else if ch == '_' {
                    self.advance();
                } else {
                    break;
                }
            }
            // Trailing alphanumerics (e.g. `0x1G`, `0xZZ`) belong to the
            // invalid literal: consume them so we resume after the whole word.
            let mut has_suffix = false;
            while let Some(&(_, ch)) = self.peek() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    // `_` was already handled above, but an alnum here means
                    // the literal is malformed; consume the rest of the word.
                    // Note: `_` alone after valid digits was already consumed,
                    // so reaching here with `_` implies it follows an invalid char.
                    has_suffix = true;
                    self.advance();
                } else {
                    break;
                }
            }
            let text = &self.source[start..self.position];
            if digits == 0 || has_suffix {
                let msg = format!("invalid numeric literal '{}'", text);
                self.errors.push(LexError {
                    line: start_line,
                    column: start_column,
                    message: msg.clone(),
                });
                return Token::Error(msg);
            }
            return Token::IntLit(text.to_string());
        }

        if self.source[start..].starts_with("0b") || self.source[start..].starts_with("0B") {
            self.advance(); // b/B
            let mut digits = 0usize;
            while let Some(&(_, ch)) = self.peek() {
                if matches!(ch, '0' | '1') {
                    digits += 1;
                    self.advance();
                } else if ch == '_' {
                    self.advance();
                } else {
                    break;
                }
            }
            let mut has_suffix = false;
            while let Some(&(_, ch)) = self.peek() {
                if ch.is_ascii_alphanumeric() || ch == '_' {
                    has_suffix = true;
                    self.advance();
                } else {
                    break;
                }
            }
            let text = &self.source[start..self.position];
            if digits == 0 || has_suffix {
                let msg = format!("invalid numeric literal '{}'", text);
                self.errors.push(LexError {
                    line: start_line,
                    column: start_column,
                    message: msg.clone(),
                });
                return Token::Error(msg);
            }
            return Token::IntLit(text.to_string());
        }

        let mut has_dot = false;
        let mut has_exponent = false;
        while let Some(&(_, ch)) = self.peek() {
            match ch {
                '0'..='9' | '_' => { self.advance(); }
                '.' if !has_dot && !has_exponent => {
                    has_dot = true;
                    self.advance();
                }
                'e' | 'E' if !has_exponent => {
                    has_exponent = true;
                    self.advance();
                    if let Some(&(_, '+' | '-')) = self.peek() {
                        self.advance();
                    }
                }
                _ => break,
            }
        }

        let num_str = &self.source[start..self.position];
        // Missing exponent digits (`12e`, `1.5e+`, ...): invalid, but resume
        // right after the literal since it is already fully consumed.
        if has_exponent {
            if let Some(epos) = num_str.find(['e', 'E']) {
                let after = &num_str[epos + 1..];
                let after = after.strip_prefix(['+', '-']).unwrap_or(after);
                let digit_count = after.chars().filter(|c| c.is_ascii_digit()).count();
                if digit_count == 0 {
                    let msg = format!("invalid numeric literal '{}'", num_str);
                    self.errors.push(LexError {
                        line: start_line,
                        column: start_column,
                        message: msg.clone(),
                    });
                    return Token::Error(msg);
                }
            }
        }
        // A number directly glued to a letter (`123abc`, `1e2e3`) is one
        // invalid literal: consume the suffix word and report once.
        if let Some(&(_, ch)) = self.peek() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                while let Some(&(_, c)) = self.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        self.advance();
                    } else {
                        break;
                    }
                }
                let text = &self.source[start..self.position];
                let msg = format!("invalid numeric literal '{}'", text);
                self.errors.push(LexError {
                    line: start_line,
                    column: start_column,
                    message: msg.clone(),
                });
                return Token::Error(msg);
            }
        }

        if has_dot || has_exponent {
            Token::FloatLit(num_str.to_string())
        } else {
            Token::IntLit(num_str.to_string())
        }
    }

    fn scan_ident(&mut self, start: usize) -> Token {
        while let Some(&(_, ch)) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let ident = &self.source[start..self.position];

        if let Some(keyword) = Token::keyword(ident) {
            keyword
        } else {
            Token::Ident(ident.to_string())
        }
    }

    fn scan_slash(&mut self, start_line: usize, start_column: usize) -> Token {
        if let Some(&(_, '/')) = self.peek() {
            self.advance();
            let mut comment = String::new();
            while let Some(&(_, ch)) = self.peek() {
                if ch == '\n' {
                    break;
                }
                self.advance();
                comment.push(ch);
            }
            return Token::Comment(comment);
        }
        if let Some(&(_, '*')) = self.peek() {
            self.advance();
            let mut comment = String::new();
            while let Some(&(_, ch)) = self.peek() {
                self.advance();
                if ch == '*' {
                    if let Some(&(_, '/')) = self.peek() {
                        self.advance();
                        return Token::BlockComment(comment);
                    }
                }
                comment.push(ch);
            }
            let msg = "Unterminated block comment".to_string();
            self.errors.push(LexError {
                line: start_line,
                column: start_column,
                message: msg.clone(),
            });
            return Token::Error(msg);
        }
        Token::Slash
    }

    fn scan_lt(&mut self) -> Token {
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::LtEq;
        }
        if let Some(&(_, '<')) = self.peek() {
            self.advance();
            return Token::LtLt;
        }
        Token::Lt
    }

    fn scan_gt(&mut self) -> Token {
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::GtEq;
        }
        if let Some(&(_, '>')) = self.peek() {
            self.advance();
            return Token::GtGt;
        }
        Token::Gt
    }

    fn scan_eq(&mut self) -> Token {
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::EqEq;
        }
        if let Some(&(_, '>')) = self.peek() {
            self.advance();
            return Token::FatArrow;
        }
        Token::Eq
    }

    fn scan_bang(&mut self) -> Token {
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::Neq;
        }
        if let Some(&(_, '!')) = self.peek() {
            self.advance();
            return Token::BangBang;
        }
        Token::Bang
    }

    fn scan_amp(&mut self) -> Token {
        if let Some(&(_, '&')) = self.peek() {
            self.advance();
            return Token::AndAnd;
        }
        Token::Ampersand
    }

    fn scan_pipe(&mut self) -> Token {
        if let Some(&(_, '|')) = self.peek() {
            self.advance();
            return Token::OrOr;
        }
        if let Some(&(_, '>')) = self.peek() {
            self.advance();
            return Token::PipeRArrow;
        }
        Token::Pipe
    }

    fn scan_plus(&mut self) -> Token {
        if let Some(&(_, '+')) = self.peek() {
            self.advance();
            return Token::PlusPlus;
        }
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::PlusEq;
        }
        Token::Plus
    }

    fn scan_minus(&mut self) -> Token {
        if let Some(&(_, '-')) = self.peek() {
            self.advance();
            return Token::MinusMinus;
        }
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::MinusEq;
        }
        if let Some(&(_, '>')) = self.peek() {
            self.advance();
            return Token::RArrow;
        }
        Token::Minus
    }

    fn scan_star(&mut self) -> Token {
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::StarEq;
        }
        Token::Star
    }

    fn scan_percent(&mut self) -> Token {
        if let Some(&(_, '=')) = self.peek() {
            self.advance();
            return Token::PercentEq;
        }
        Token::Percent
    }

    fn scan_question(&mut self) -> Token {
        if let Some(&(_, '?')) = self.peek() {
            self.advance();
            return Token::QuestionQuestion;
        }
        if let Some(&(_, '.')) = self.peek() {
            self.advance();
            return Token::QuestionDot;
        }
        Token::Question
    }

    fn scan_dot(&mut self) -> Token {
        if let Some(&(_, '.')) = self.peek() {
            self.advance();
            if let Some(&(_, '.')) = self.peek() {
                self.advance();
                if let Some(&(_, '.')) = self.peek() {
                    self.advance();
                    return Token::DotDotDot;
                }
                if let Some(&(_, '=')) = self.peek() {
                    self.advance();
                    return Token::DotDotEq;
                }
                return Token::DotDot;
            }
        }
        Token::Dot
    }

    fn scan_colon(&mut self) -> Token {
        if let Some(&(_, ':')) = self.peek() {
            self.advance();
            return Token::PathSep;
        }
        Token::Colon
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_tokenization() {
        let source = "module test;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(matches!(&tokens[0].token, Token::Module));
        assert!(matches!(&tokens[1].token, Token::Ident(i) if i == "test"));
        assert!(matches!(&tokens[2].token, Token::Semi));
    }

    #[test]
    fn test_string_literal() {
        let source = r#"let x = "hello";"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(matches!(&tokens[3].token, Token::StringLit(s) if s == "hello"));
    }

    #[test]
    fn test_operators() {
        let source = "== != <= >= && || => -> :: ??";
        let mut lexer = Lexer::new(source);
        let all = lexer.tokenize();
        let tokens: Vec<_> = all
            .iter()
            .filter(|t| !matches!(t.token, Token::Eof))
            .collect();

        assert!(matches!(&tokens[0].token, Token::EqEq));
        assert!(matches!(&tokens[1].token, Token::Neq));
        assert!(matches!(&tokens[2].token, Token::LtEq));
    }

    #[test]
    fn test_numeric_literals() {
        let source = "123 0x1A 0b101 1.23e-4";
        let mut lexer = Lexer::new(source);
        let all = lexer.tokenize();
        let tokens: Vec<_> = all.into_iter().filter(|t| !matches!(t.token, Token::Eof)).collect();

        assert!(matches!(&tokens[0].token, Token::IntLit(s) if s == "123"));
        assert!(matches!(&tokens[1].token, Token::IntLit(s) if s == "0x1A"));
        assert!(matches!(&tokens[2].token, Token::IntLit(s) if s == "0b101"));
        assert!(matches!(&tokens[3].token, Token::FloatLit(s) if s == "1.23e-4"));
    }

    #[test]
    fn test_switch_tokens() {
        let source = "switch x { case 1: break; default: break; }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(matches!(&tokens[0].token, Token::Switch));
        assert!(matches!(&tokens[2].token, Token::LBrace));
        assert!(matches!(&tokens[3].token, Token::Case));
    }

    #[test]
    fn test_attribute_sigils() {
        // `@attr` and `#[cfg(...)]` directive forms (Spec §1 + §76).
        let source = "@inline #[cfg(feature = \"net\")]";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        assert!(matches!(&tokens[0].token, Token::At));
        assert!(matches!(&tokens[2].token, Token::Hash));
        assert!(matches!(&tokens[3].token, Token::LBracket));
    }

    #[test]
    fn test_multi_error_unterminated_and_bad_escape() {
        // One unterminated string (line 1) + one bad escape (line 2) in a
        // single file must yield exactly 2 errors with the right locations,
        // while the remaining tokens stay intact.
        let source = "let s = \"oops\nlet t = \"bad \\x here\";\nok";
        let mut lexer = Lexer::new(source);
        let (tokens, errors) = lexer.tokenize_with_errors();

        assert_eq!(errors.len(), 2, "expected 2 errors, got {:?}", errors);

        // Unterminated string starts at the opening quote: line 1, column 9.
        assert_eq!(errors[0].line, 1);
        assert_eq!(errors[0].column, 9);
        assert!(
            errors[0].message.to_lowercase().contains("unterminated"),
            "unexpected message: {}",
            errors[0].message
        );

        // Bad escape `\x`: the escaped char `x` is at line 2, column 15.
        assert_eq!(errors[1].line, 2);
        assert_eq!(errors[1].column, 15);
        assert!(
            errors[1].message.to_lowercase().contains("escape"),
            "unexpected message: {}",
            errors[1].message
        );

        // Remaining tokens intact: the string with the bad escape still
        // produces a StringLit, and the trailing `ok` ident survives.
        let has_string = tokens
            .iter()
            .any(|t| matches!(&t.token, Token::StringLit(s) if s.contains("bad ")));
        assert!(has_string, "expected StringLit for bad-escape line: {:?}", tokens.iter().map(|t| &t.token).collect::<Vec<_>>());
        let has_ok = tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Ident(i) if i == "ok"));
        assert!(has_ok, "expected trailing `ok` ident: {:?}", tokens.iter().map(|t| &t.token).collect::<Vec<_>>());
        // The unterminated line surfaces as an Error token but lexing continues.
        let has_unterminated = tokens
            .iter()
            .any(|t| matches!(&t.token, Token::Error(m) if m.to_lowercase().contains("unterminated")));
        assert!(has_unterminated);
    }

    #[test]
    fn test_invalid_numeric_collects_error_and_continues() {
        let source = "let a = 0x;\nlet b = 1;";
        let mut lexer = Lexer::new(source);
        let (tokens, errors) = lexer.tokenize_with_errors();

        assert_eq!(errors.len(), 1, "expected 1 error, got {:?}", errors);
        assert_eq!(errors[0].line, 1);
        assert_eq!(errors[0].column, 9);
        assert!(
            errors[0].message.to_lowercase().contains("numeric"),
            "unexpected message: {}",
            errors[0].message
        );
        // Resumption: the second line still lexes fully.
        let lets = tokens.iter().filter(|t| matches!(&t.token, Token::Let)).count();
        assert_eq!(lets, 2);
        let has_one = tokens
            .iter()
            .any(|t| matches!(&t.token, Token::IntLit(s) if s == "1"));
        assert!(has_one);
    }

    #[test]
    fn test_leading_trivia_in_order() {
        let source = "// first\n// second\n/* third */\nlet x = 1;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();

        // Token stream itself is unchanged: comments are still tokens.
        assert!(matches!(&tokens[0].token, Token::Comment(_)));
        assert!(matches!(&tokens[1].token, Token::Comment(_)));
        assert!(matches!(&tokens[2].token, Token::BlockComment(_)));
        assert!(matches!(&tokens[3].token, Token::Let));

        let let_idx = tokens
            .iter()
            .position(|t| matches!(&t.token, Token::Let))
            .expect("Let token");
        let trivia = lexer.leading_trivia(let_idx);
        assert_eq!(trivia.len(), 3, "expected 3 leading comments, got {:?}", trivia);
        assert!(trivia[0].contains("first"), "order broken: {:?}", trivia);
        assert!(trivia[1].contains("second"), "order broken: {:?}", trivia);
        assert!(trivia[2].contains("third"), "order broken: {:?}", trivia);

        // Full table agrees and stays parallel to the token stream.
        let table = lexer.trivia_table();
        assert_eq!(table.len(), tokens.len());
        assert_eq!(table[let_idx], trivia);
    }
}
