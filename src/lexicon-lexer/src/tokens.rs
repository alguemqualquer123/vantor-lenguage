use lexicon_core::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Eof,
    Comment(String),
    BlockComment(String),
    Module,
    Import,
    Pub,
    Private,
    Protected,
    Class,
    Struct,
    Enum,
    Trait,
    Interface,
    Type,
    Fn,
    Async,
    Return,
    Defer,
    Let,
    Var,
    Mut,
    Const,
    Static,
    If,
    Else,
    Match,
    Switch,
    Case,
    Default,
    Do,
    For,
    While,
    Loop,
    Break,
    Continue,
    Try,
    Catch,
    Throw,
    Finally,
    Await,
    Task,
    Channel,
    Actor,
    Self_,
    Super,
    Where,
    As,
    Is,
    Lambda,
    In,
    New,
    Unsafe,
    Extern,
    Inline,
    Native,
    Panic,
    Recover,
    True,
    False,
    Null,
    IntLit(String),
    FloatLit(String),
    StringLit(String),
    CharLit(char),
    Ident(String),
    Extends,
    Implements,
    Effect,
    Constructor,
    Dynamic,
    Function,
    At,
    DotDotDot,
    DotDot,
    DotDotEq,
    Hash,
    PlusPlus,
    MinusMinus,
    EqEq,
    Neq,
    LtEq,
    GtEq,
    AndAnd,
    OrOr,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    LtLt,
    GtGt,
    FatArrow,
    RArrow,
    PathSep,
    QuestionQuestion,
    QuestionDot,
    BangBang,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Lt,
    Gt,
    Comma,
    Dot,
    Semi,
    Colon,
    Eq,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    Ampersand,
    Pipe,
    Caret,
    Tilde,
    Question,
    Underscore,
    PipeRArrow,
    Error(String),
    Macro,
    Spawn,
    Select,
    With,
    Derive,
    Some,
    None,
    Yield,
    Ref,
}

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Module
                | Token::Import
                | Token::Pub
                | Token::Private
                | Token::Protected
                | Token::Class
                | Token::Struct
                | Token::Enum
                | Token::Trait
                | Token::Interface
                | Token::Type
                | Token::Fn
                | Token::Async
                | Token::Return
                | Token::Defer
                | Token::Let
                | Token::Var
                | Token::Mut
                | Token::Const
                | Token::Static
                | Token::If
                | Token::Else
                | Token::Match
                | Token::Switch
                | Token::Case
                | Token::Default
                | Token::Do
                | Token::For
                | Token::While
                | Token::Loop
                | Token::Break
                | Token::Continue
                | Token::Try
                | Token::Catch
                | Token::Throw
                | Token::Finally
                | Token::Await
                | Token::Task
                | Token::Channel
                | Token::Actor
                | Token::Self_
                | Token::Super
                | Token::Where
                | Token::As
                | Token::Is
                | Token::Lambda
                | Token::New
                | Token::In
                | Token::Unsafe
                | Token::Extern
                | Token::Inline
                | Token::Native
                | Token::True
                | Token::False
                | Token::Null
                | Token::Extends
                | Token::Implements
                | Token::Effect
                | Token::Constructor
                | Token::Dynamic
                | Token::Function
                | Token::At
                | Token::Macro
                | Token::Spawn
                | Token::Select
                | Token::With
                | Token::Derive
                | Token::Some
                | Token::None
                | Token::Yield
                | Token::Ref
        )
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            Token::IntLit(_) | Token::FloatLit(_) | Token::StringLit(_) | Token::CharLit(_)
        )
    }

    pub fn keyword(text: &str) -> Option<Token> {
        match text {
            "module" => Some(Token::Module),
            "import" => Some(Token::Import),
            "pub" => Some(Token::Pub),
            "private" => Some(Token::Private),
            "protected" => Some(Token::Protected),
            "class" => Some(Token::Class),
            "struct" => Some(Token::Struct),
            "enum" => Some(Token::Enum),
            "trait" => Some(Token::Trait),
            "interface" => Some(Token::Interface),
            "type" => Some(Token::Type),
            "fn" => Some(Token::Fn),
            "async" => Some(Token::Async),
            "return" => Some(Token::Return),
            "defer" => Some(Token::Defer),
            "let" => Some(Token::Let),
            "var" => Some(Token::Var),
            "mut" => Some(Token::Mut),
            "const" => Some(Token::Const),
            "static" => Some(Token::Static),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "match" => Some(Token::Match),
            "switch" => Some(Token::Switch),
            "case" => Some(Token::Case),
            "default" => Some(Token::Default),
            "do" => Some(Token::Do),
            "for" => Some(Token::For),
            "while" => Some(Token::While),
            "loop" => Some(Token::Loop),
            "break" => Some(Token::Break),
            "continue" => Some(Token::Continue),
            "try" => Some(Token::Try),
            "catch" => Some(Token::Catch),
            "throw" => Some(Token::Throw),
            "finally" => Some(Token::Finally),
            "await" => Some(Token::Await),
            "macro" => Some(Token::Macro),
            "task" => Some(Token::Task),
            "channel" => Some(Token::Channel),
            "actor" => Some(Token::Actor),
            "self" => Some(Token::Self_),
            "super" => Some(Token::Super),
            "where" => Some(Token::Where),
            "as" => Some(Token::As),
            "is" => Some(Token::Is),
            "lambda" => Some(Token::Lambda),
            "new" => Some(Token::New),
            "in" => Some(Token::In),
            "unsafe" => Some(Token::Unsafe),
            "extern" => Some(Token::Extern),
            "inline" => Some(Token::Inline),
            "native" => Some(Token::Native),
            "panic" => Some(Token::Panic),
            "recover" => Some(Token::Recover),
            "true" => Some(Token::True),
            "false" => Some(Token::False),
            "null" => Some(Token::Null),
            "extends" => Some(Token::Extends),
            "implements" => Some(Token::Implements),
            "effect" => Some(Token::Effect),
            "constructor" => Some(Token::Constructor),
            "dynamic" => Some(Token::Dynamic),
            "function" => Some(Token::Function),
            "@" => Some(Token::At),
            "spawn" => Some(Token::Spawn),
            "select" => Some(Token::Select),
            "with" => Some(Token::With),
            "derive" => Some(Token::Derive),
            "Some" => Some(Token::Some),
            "None" => Some(Token::None),
            "yield" => Some(Token::Yield),
            "ref" => Some(Token::Ref),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

impl SpannedToken {
    pub fn new(token: Token, span: Span) -> Self {
        SpannedToken { token, span }
    }
}
