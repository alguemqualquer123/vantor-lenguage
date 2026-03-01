use crate::ast::*;
use lexicon_core::error::{Error, Result};
use lexicon_core::span::Span;
use lexicon_lexer::tokens::SpannedToken as LexerSpannedToken;
use lexicon_lexer::tokens::Token;

pub struct Parser {
    tokens: Vec<LexerSpannedToken>,
    pos: usize,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: Vec<LexerSpannedToken>) -> Self {
        Parser {
            tokens,
            pos: 0,
            errors: Vec::new(),
        }
    }

    pub fn into_errors(self) -> Vec<Error> {
        self.errors
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
            let err = Error::Parse(format!(
                "Expected {:?} but found {:?}",
                expected, current.token
            ));
            self.errors.push(err.clone());
            Err(err)
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
            if let Ok(decl) = self.parse_declaration() {
                declarations.push(decl);
            } else {
                self.advance();
            }
        }

        Ok(Module {
            name,
            imports,
            declarations,
            span: start_span,
        })
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
        
        let mut i = 0;
        let mut cur = self.peek(i).token.clone();
        
        // Skip modifiers to see what kind of declaration it is
        while matches!(cur, Token::Pub | Token::Private | Token::Protected | Token::Async | Token::Static | Token::Const) {
            i += 1;
            cur = self.peek(i).token.clone();
        }

        match cur {
            Token::Fn => Ok(Decl::Function(self.parse_function(attrs)?)),
            Token::Class => Ok(Decl::Class(self.parse_class(attrs)?)),
            Token::Struct => Ok(Decl::Struct(self.parse_struct(attrs)?)),
            Token::Enum => Ok(Decl::Enum(self.parse_enum(attrs)?)),
            _ => Err(Error::Parse(format!(
                "Unexpected header for declaration: {:?}",
                self.current().token
            ))),
        }
    }

    fn parse_attribute(&mut self) -> Result<Attribute> {
        let start = self.current().span;
        self.expect(Token::At)?;
        let name = self.parse_ident()?;
        let mut args = Vec::new();
        if self.check(&Token::LParen) {
            self.advance();
            while !self.check(&Token::RParen) && !self.is_at_end() {
                args.push(self.parse_expression()?);
                if self.check(&Token::Comma) {
                    self.advance();
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
        self.expect(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(Token::RParen)?;

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
            type_params: Vec::new(),
            params,
            return_type,
            where_clause: Vec::new(),
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
            while matches!(cur, Token::Pub | Token::Private | Token::Protected | Token::Async | Token::Static | Token::Const | Token::Mut) {
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
            type_params: Vec::new(),
            extends: None,
            implements: Vec::new(),
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
            name: Ident { text: "constructor".to_string(), span: start },
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
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let mut f_attrs = Vec::new();
            while self.check(&Token::At) {
                f_attrs.push(self.parse_attribute()?);
            }
            if let Ok(f) = self.parse_field_internal(f_attrs) {
                fields.push(f);
            } else {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Struct {
            attrs,
            visibility,
            name,
            type_params: Vec::new(),
            fields,
            span: start,
        })
    }

    fn parse_enum(&mut self, attrs: Vec<Attribute>) -> Result<Enum> {
        let start = self.current().span;
        self.expect(Token::Enum)?;
        let name = self.parse_ident()?;
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
            type_params: Vec::new(),
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
        self.expect(Token::Semi)?;
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
            params.push(Param {
                attrs: Vec::new(),
                is_mutable: false,
                name,
                ty,
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
            statements.push(self.parse_statement()?);
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
            Token::Let | Token::Var => Ok(Stmt::Decl(self.parse_decl_stmt()?)),
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
        
        let name = self.parse_ident()?;
        
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
            pattern: Pattern::Ident(name.clone()),
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
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expr> {
        let mut expr = self.parse_pipe()?;

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
            let op = if self.check(&Token::EqEq) { BinOp::EqEq } else { BinOp::Neq };
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
        let mut expr = self.parse_term()?;
        while self.check(&Token::Lt) || self.check(&Token::LtEq) || self.check(&Token::Gt) || self.check(&Token::GtEq) {
            let op = match self.current().token {
                Token::Lt => BinOp::Lt,
                Token::LtEq => BinOp::LtEq,
                Token::Gt => BinOp::Gt,
                Token::GtEq => BinOp::GtEq,
                _ => unreachable!(),
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
            let op = if self.check(&Token::Plus) { BinOp::Add } else { BinOp::Sub };
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
            let op = match self.current().token {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => unreachable!(),
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
        if self.check(&Token::Bang) || self.check(&Token::Minus) || self.check(&Token::Await) || self.check(&Token::Star) {
            let op = match self.current().token {
                Token::Bang => UnOp::Not,
                Token::Minus => UnOp::Neg,
                Token::Star => UnOp::Deref,
                Token::Await => {
                    self.advance();
                    let expr = self.parse_unary()?;
                    return Ok(Expr::Await(Box::new(AwaitExpr { expr: Box::new(expr), span: Span::default() })));
                }
                _ => unreachable!(),
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
                    
                    let method_call = Expr::MethodCall(MethodCallExpr {
                        object: Box::new(expr.clone()),
                        method: method.clone(),
                        type_args,
                        args,
                        span: Span::default(),
                    });

                    if is_null_safe {
                        // Envolver em algo que represente null-safe no AST se necessário,
                        // ou tratar no typeck/codegen. Por agora mantemos como MethodCall normal
                        // mas poderíamos ter um enum variant específico.
                        expr = method_call;
                    } else {
                        expr = method_call;
                    }
                } else {
                    let field_access = Expr::FieldAccess(FieldAccessExpr {
                        object: Box::new(expr.clone()),
                        field: method,
                        span: Span::default(),
                    });
                    expr = field_access;
                }
            } else if self.check(&Token::QuestionQuestion) {
                self.advance();
                let rhs = self.parse_expression()?;
                expr = Expr::NullCoalesce(Box::new(expr), Box::new(rhs));
            } else if self.check(&Token::PathSep) {
                self.advance();
                let name = self.parse_ident()?;
                // Simplification for PathSep in expressions
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
                        ty: Type::Path(Path::from_ident(&Ident { text: "Any".to_string(), span: Span::default() })),
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
            Token::True => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
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
                Ok(Expr::Literal(Literal::Float(
                    n.parse().unwrap_or(0.0),
                    None,
                )))
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
                    let mut look = 1;
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
                let e = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(e)
            }
            _ => {
                let err = Error::Parse(format!("Expected expression, found {:?}", token));
                self.errors.push(err.clone());
                Err(err)
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

    fn parse_type(&mut self) -> Result<Type> {
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
            Ok(Type::Tuple(types))
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

    fn parse_ident(&mut self) -> Result<Ident> {
        if let Token::Ident(text) = &self.current().token.clone() {
            let span = self.current().span;
            self.advance();
            Ok(Ident {
                text: text.clone(),
                span,
            })
        } else {
            Err(Error::Parse("Expected identifier".to_string()))
        }
    }

    fn parse_path(&mut self) -> Result<Path> {
        let start = self.current().span;
        let mut segments = vec![self.parse_ident()?];
        while self.check(&Token::PathSep) || self.check(&Token::Dot) {
            self.advance();
            segments.push(self.parse_ident()?);
        }
        Ok(Path {
            segments,
            span: start,
        })
    }
}
