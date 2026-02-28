use crate::ast::*;
use vantor_lexer::tokens::Token;
use vantor_lexer::tokens::SpannedToken as LexerSpannedToken;
use vantor_core::span::Span;
use vantor_core::error::{Error, Result};

pub struct Parser {
    tokens: Vec<LexerSpannedToken>,
    pos: usize,
    errors: Vec<Error>,
}

impl Parser {
    pub fn new(tokens: Vec<LexerSpannedToken>) -> Self {
        Parser { tokens, pos: 0, errors: Vec::new() }
    }

    pub fn into_errors(self) -> Vec<Error> { self.errors }

    fn current(&self) -> LexerSpannedToken {
        self.tokens.get(self.pos).cloned().unwrap_or(LexerSpannedToken::new(Token::Eof, Span::default()))
    }

    fn advance(&mut self) { if !self.is_at_end() { self.pos += 1; } }
    fn is_at_end(&self) -> bool {
        matches!(self.current().token, Token::Eof)
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
            let err = Error::Parse(format!("Expected {:?} but found {:?}", expected, current.token));
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
            Path { segments: vec![Ident { text: "main".to_string(), span: start_span }], span: start_span }
        };
        
        let mut imports = Vec::new();
        while self.check(&Token::Import) { imports.push(self.parse_import()?); }
        
        let mut declarations = Vec::new();
        while !self.is_at_end() {
            if let Ok(decl) = self.parse_declaration() { declarations.push(decl); }
            else { self.advance(); }
        }
        
        Ok(Module { name, imports, declarations, span: start_span })
    }

    fn parse_import(&mut self) -> Result<Import> {
        let start = self.current().span;
        self.expect(Token::Import)?;
        let path = self.parse_path()?;
        let alias = if self.check(&Token::As) { self.advance(); Some(self.parse_ident()?) } else { None };
        self.expect(Token::Semi)?;
        Ok(Import { path, alias, span: start })
    }

    fn parse_declaration(&mut self) -> Result<Decl> {
        match &self.current().token {
            Token::Fn => Ok(Decl::Function(self.parse_function(Vec::new())?)),
            Token::Class => Ok(Decl::Class(self.parse_class(Vec::new())?)),
            Token::Struct => Ok(Decl::Struct(self.parse_struct(Vec::new())?)),
            Token::Enum => Ok(Decl::Enum(self.parse_enum(Vec::new())?)),
            _ => Err(Error::Parse(format!("Unexpected: {:?}", self.current().token))),
        }
    }

    fn parse_function(&mut self, attrs: Vec<Attribute>) -> Result<Function> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) { self.advance(); Visibility::Public } else { Visibility::Private };
        let is_async = self.check(&Token::Async);
        if is_async { self.advance(); }
        
        self.expect(Token::Fn)?;
        let name = self.parse_ident()?;
        self.expect(Token::LParen)?;
        let params = self.parse_param_list()?;
        self.expect(Token::RParen)?;
        
        let return_type = if self.check(&Token::RArrow) { self.advance(); Some(self.parse_type()?) } else { None };
        
        let body = if self.check(&Token::LBrace) { Some(self.parse_block()?) } else { self.expect(Token::Semi)?; None };
        
        Ok(Function { attrs, visibility, is_async, name, type_params: Vec::new(), params, return_type, where_clause: Vec::new(), preconditions: Vec::new(), postconditions: Vec::new(), body, span: start })
    }

    fn parse_class(&mut self, attrs: Vec<Attribute>) -> Result<Class> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) { self.advance(); Visibility::Public } else { Visibility::Private };
        self.expect(Token::Class)?;
        let name = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut members = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if self.check(&Token::Fn) { members.push(ClassMember::Method(self.parse_function(Vec::new())?)); }
            else if let Ok(f) = self.parse_field_internal() { members.push(ClassMember::Field(f)); }
            else { self.advance(); }
        }
        self.expect(Token::RBrace)?;
        Ok(Class { attrs, visibility, name, type_params: Vec::new(), extends: None, implements: Vec::new(), members, span: start })
    }

    fn parse_struct(&mut self, attrs: Vec<Attribute>) -> Result<Struct> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) { self.advance(); Visibility::Public } else { Visibility::Private };
        self.expect(Token::Struct)?;
        let name = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            if let Ok(f) = self.parse_field_internal() { fields.push(f); }
            else { self.advance(); }
        }
        self.expect(Token::RBrace)?;
        Ok(Struct { attrs, visibility, name, type_params: Vec::new(), fields, span: start })
    }

    fn parse_enum(&mut self, attrs: Vec<Attribute>) -> Result<Enum> {
        let start = self.current().span;
        self.expect(Token::Enum)?;
        let name = self.parse_ident()?;
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            let vname = self.parse_ident()?;
            variants.push(EnumVariant { name: vname, data: Vec::new(), span: start });
            if !self.check(&Token::RBrace) { self.expect(Token::Comma).ok(); }
        }
        self.expect(Token::RBrace)?;
        Ok(Enum { attrs, visibility: Visibility::Private, name, type_params: Vec::new(), variants, span: start })
    }

    fn parse_field_internal(&mut self) -> Result<Field> {
        let start = self.current().span;
        let visibility = if self.check(&Token::Pub) { self.advance(); Visibility::Public } else { Visibility::Private };
        let is_mutable = self.check(&Token::Mut);
        if is_mutable { self.advance(); }
        let name = self.parse_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Semi)?;
        Ok(Field { attrs: Vec::new(), visibility, is_mutable, name, ty, default: None, span: start })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        while !self.check(&Token::RParen) {
            let name = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { attrs: Vec::new(), is_mutable: false, name, ty, default: None, span: Span::default() });
            if !self.check(&Token::RParen) { self.expect(Token::Comma).ok(); }
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
        Ok(Block { statements, span: start })
    }

    fn parse_statement(&mut self) -> Result<Stmt> {
        match &self.current().token {
            Token::If => Ok(Stmt::If(self.parse_if()?)),
            Token::Return => { self.advance(); Ok(Stmt::Return(ReturnStmt { value: None, span: Span::default() })) }
            Token::Let | Token::Var => Ok(Stmt::Decl(self.parse_decl_stmt()?)),
            _ => Ok(Stmt::Expr(self.parse_expr_stmt()?)),
        }
    }

    fn parse_if(&mut self) -> Result<IfStmt> {
        let start = self.current().span;
        self.expect(Token::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        let else_body = if self.check(&Token::Else) { self.advance(); Some(self.parse_block()?) } else { None };
        Ok(IfStmt { condition, then_branch, else_branch: None, else_body, span: start })
    }

    fn parse_decl_stmt(&mut self) -> Result<DeclStmt> {
        let start = self.current().span;
        let is_mutable = self.check(&Token::Var);
        self.advance();
        let name = self.parse_ident()?;
        let init = if self.check(&Token::Eq) { self.advance(); Some(self.parse_expression()?) } else { None };
        self.expect(Token::Semi).ok();
        Ok(DeclStmt { pattern: Pattern::Ident(name.clone()), ty: None, init, span: start })
    }

    fn parse_expr_stmt(&mut self) -> Result<ExprStmt> {
        let expr = self.parse_expression()?;
        self.expect(Token::Semi).ok();
        Ok(ExprStmt { expr, span: Span::default() })
    }

    fn parse_expression(&mut self) -> Result<Expr> {
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        let token = self.current().token.clone();
        match &token {
            Token::True => { self.advance(); Ok(Expr::Literal(Literal::Bool(true))) }
            Token::False => { self.advance(); Ok(Expr::Literal(Literal::Bool(false))) }
            Token::IntLit(n) => { self.advance(); Ok(Expr::Literal(Literal::Int(n.parse().unwrap_or(0), None))) }
            Token::FloatLit(n) => { self.advance(); Ok(Expr::Literal(Literal::Float(n.parse().unwrap_or(0.0), None))) }
            Token::StringLit(s) => { self.advance(); Ok(Expr::Literal(Literal::String(s.clone()))) }
            Token::Ident(id) => { self.advance(); Ok(Expr::Ident(Ident { text: id.clone(), span: Span::default() })) }
            Token::LParen => { self.advance(); let e = self.parse_expression()?; self.expect(Token::RParen)?; Ok(e) }
            _ => { self.advance(); Err(Error::Parse(format!("Unexpected: {:?}", token))) }
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        if let Token::Ident(id) = &self.current().token.clone() {
            let span = self.current().span;
            self.advance();
            Ok(Type::Path(Path { segments: vec![Ident { text: id.clone(), span }], span }))
        } else if self.check(&Token::LParen) {
            self.advance();
            let mut types = Vec::new();
            while !self.check(&Token::RParen) {
                types.push(self.parse_type()?);
                if !self.check(&Token::RParen) { self.expect(Token::Comma).ok(); }
            }
            self.expect(Token::RParen)?;
            Ok(Type::Tuple(types))
        } else {
            Err(Error::Parse("Expected type".to_string()))
        }
    }

    fn parse_ident(&mut self) -> Result<Ident> {
        if let Token::Ident(text) = &self.current().token.clone() {
            let span = self.current().span;
            self.advance();
            Ok(Ident { text: text.clone(), span })
        } else {
            Err(Error::Parse("Expected identifier".to_string()))
        }
    }

    fn parse_path(&mut self) -> Result<Path> {
        let start = self.current().span;
        let mut segments = vec![self.parse_ident()?];
        while self.check(&Token::PathSep) { self.advance(); segments.push(self.parse_ident()?); }
        Ok(Path { segments, span: start })
    }
}
