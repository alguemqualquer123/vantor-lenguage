use crate::tokens::{Token, SpannedToken};
use vantor_core::span::Span;

pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    position: usize,
    line: usize,
    column: usize,
    start: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars = source.char_indices().peekable();
        Lexer {
            source,
            chars,
            position: 0,
            line: 1,
            column: 1,
            start: 0,
        }
    }
    
    pub fn tokenize(&mut self) -> Vec<SpannedToken> {
        let mut tokens = Vec::new();
        
        while let Some((pos, ch)) = self.chars.next() {
            self.start = pos;
            
            let token = match ch {
                ' ' | '\t' | '\n' | '\r' => continue,
                '/' => self.scan_slash(),
                '"' => self.scan_string(),
                '\'' => self.scan_char(),
                '0'..='9' => self.scan_number(pos),
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
                _ => Token::Error(format!("Unexpected character: {}", ch)),
            };
            
            tokens.push(SpannedToken {
                token,
                span: Span::new(self.start, self.position),
            });
        }
        
        tokens.push(SpannedToken {
            token: Token::Eof,
            span: Span::new(self.source.len(), self.source.len()),
        });
        
        tokens
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
    
    fn scan_string(&mut self) -> Token {
        let mut s = String::new();
        while let Some(&(_, ch)) = self.peek() {
            if ch == '"' {
                self.advance();
                return Token::StringLit(s);
            }
            if ch == '\\' {
                self.advance();
                if let Some(&(_, esc)) = self.peek() {
                    let escaped = match esc {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '\\' => '\\',
                        '"' => '"',
                        '\'' => '\'',
                        _ => esc,
                    };
                    s.push(escaped);
                    self.advance();
                }
            } else {
                s.push(ch);
                self.advance();
            }
        }
        Token::Error("Unterminated string".to_string())
    }
    
    fn scan_char(&mut self) -> Token {
        if let Some(&(_, ch)) = self.peek() {
            self.advance();
            if let Some(&(_, '\'')) = self.peek() {
                self.advance();
                return Token::CharLit(ch);
            }
        }
        Token::Error("Unterminated char".to_string())
    }
    
    fn scan_number(&mut self, start: usize) -> Token {
        let mut has_dot = false;
        while let Some(&(_, ch)) = self.peek() {
            match ch {
                '0'..='9' | '_' => { self.advance(); }
                '.' if !has_dot => {
                    has_dot = true;
                    self.advance();
                }
                'e' | 'E' => {
                    self.advance();
                    if let Some(&(_, '+' | '-')) = self.peek() {
                        self.advance();
                    }
                }
                _ => break,
            }
        }
        
        let num_str = &self.source[start..self.position];
        if has_dot {
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
    
    fn scan_slash(&mut self) -> Token {
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
            return Token::Error("Unterminated block comment".to_string());
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
        Token::Question
    }
    
    fn scan_dot(&mut self) -> Token {
        if let Some(&(_, '.')) = self.peek() {
            self.advance();
            if let Some(&(_, '=')) = self.peek() {
                self.advance();
                return Token::DotDotEq;
            }
            return Token::DotDot;
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
        
        assert!(matches!(tokens[0].token, Token::Module));
        assert!(matches!(tokens[1].token, Token::Ident(i) if i == "test"));
        assert!(matches!(tokens[2].token, Token::Semi));
    }
    
    #[test]
    fn test_string_literal() {
        let source = r#"let x = "hello";"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize();
        
        assert!(matches!(tokens[3].token, Token::StringLit(s) if s == "hello"));
    }
    
    #[test]
    fn test_operators() {
        let source = "== != <= >= && || => -> :: ??";
        let mut lexer = Lexer::new(source);
        let tokens: Vec<_> = lexer.tokenize().iter().filter(|t| !matches!(t.token, Token::Eof)).collect();
        
        assert!(matches!(tokens[0].token, Token::EqEq));
        assert!(matches!(tokens[1].token, Token::Neq));
        assert!(matches!(tokens[2].token, Token::LtEq));
    }
}
