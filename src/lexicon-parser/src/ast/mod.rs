use lexicon_core::span::Span;

#[derive(Debug, Clone)]
pub struct Module {
    pub name: Path,
    pub imports: Vec<Import>,
    pub declarations: Vec<Decl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub path: Path,
    pub alias: Option<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Path {
    pub segments: Vec<Ident>,
    pub span: Span,
}

impl Path {
    pub fn from_ident(ident: &Ident) -> Self {
        Path {
            segments: vec![ident.clone()],
            span: ident.span,
        }
    }

    pub fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|i| i.text.clone())
            .collect::<Vec<_>>()
            .join("::")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ident {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    PubCrate,
}

#[derive(Debug, Clone)]
pub enum Decl {
    Function(Function),
    Class(Class),
    Struct(Struct),
    Enum(Enum),
    Trait(Trait),
    Interface(Interface),
    GlobalVar(GlobalVar),
    TypeAlias(TypeAlias),
    Effect(Effect),
    /// RPC service definition: `service Name { rpc F(x: T) -> R; ... }`.
    /// Checked for duplicate rpc names; codegen treats it as metadata
    /// (like traits) until a backend lowers it.
    Service(ServiceDecl),
}

#[derive(Debug, Clone)]
pub struct ServiceDecl {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub rpcs: Vec<ServiceRpc>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ServiceRpc {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub is_async: bool,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub where_clause: Vec<TypeConstraint>,
    pub preconditions: Vec<Expr>,
    pub postconditions: Vec<Expr>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Class {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub extends: Option<Type>,
    pub implements: Vec<Type>,
    pub members: Vec<ClassMember>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ClassMember {
    Field(Field),
    Method(Function),
    Constructor(Constructor),
}

#[derive(Debug, Clone)]
pub struct Struct {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<Field>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Enum {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Ident,
    pub data: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Trait {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub extends: Vec<Type>,
    pub methods: Vec<MethodSig>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Interface {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub extends: Vec<Type>,
    pub methods: Vec<MethodSig>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MethodSig {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub is_async: bool,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub default_body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub is_mutable: bool,
    pub name: Ident,
    pub ty: Type,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Constructor {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct GlobalVar {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub is_mutable: bool,
    pub is_static: bool,
    pub name: Ident,
    pub ty: Type,
    pub init: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeAlias {
    pub attrs: Vec<Attribute>,
    pub visibility: Visibility,
    pub name: Ident,
    pub type_params: Vec<TypeParam>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Effect {
    pub name: Ident,
    pub operations: Vec<EffectOperation>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EffectOperation {
    pub name: Ident,
    pub param_types: Vec<Type>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub type_param: Type,
    pub bounds: Vec<Type>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub attrs: Vec<Attribute>,
    pub is_mutable: bool,
    pub name: Ident,
    pub ty: Type,
    /// `name: T...` — variadic parameter (Spec §4). Must be the last
    /// parameter; callers may pass zero or more trailing arguments.
    pub is_variadic: bool,
    pub default: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: Ident,
    pub bounds: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

impl Attribute {
    /// First string-literal argument, if present
    /// (`#[abi("C")]` records its calling convention this way).
    pub fn str_arg(&self) -> Option<String> {
        match self.args.first() {
            Some(Expr::Literal(Literal::String(s))) => Some(s.clone()),
            _ => None,
        }
    }
}

impl Function {
    /// Calling convention from `#[abi("C")]` / `@abi("C")`, if present.
    pub fn abi(&self) -> Option<String> {
        self.attrs
            .iter()
            .find(|a| a.name.text == "abi")
            .and_then(|a| a.str_arg())
    }

    /// Whether `#[inline]` / `@inline` was recorded on this function.
    pub fn is_inline(&self) -> bool {
        self.attrs.iter().any(|a| a.name.text == "inline")
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr(ExprStmt),
    Decl(DeclStmt),
    If(IfStmt),
    Match(MatchStmt),
    Switch(SwitchStmt),
    For(ForStmt),
    While(WhileStmt),
    DoWhile(DoWhileStmt),
    Loop(LoopStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Defer(DeferStmt),
    AsyncBlock(AsyncBlock),
    Block(Block),
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DeclStmt {
    pub pattern: Pattern,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Ident(Ident),
    Wildcard(Span),
    Tuple(Vec<Pattern>),
    Array(Vec<Pattern>),
    Object(Vec<FieldPattern>),
    EnumVariant(Ident, Vec<Pattern>),
    Named(Ident, Box<Pattern>),
    Literal(Literal),
    Or(Vec<Pattern>),
    Range(Option<Box<Pattern>>, Option<Box<Pattern>>),
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub field: Ident,
    pub pattern: Pattern,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Block,
    pub else_branch: Option<Box<IfStmt>>,
    pub else_body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub expr: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
    pub span: Span,
}

/// `switch` statement (Spec §3): scrutinee + ordered cases + optional default.
/// Falls through only when a case body ends without `break`/`return`.
#[derive(Debug, Clone)]
pub struct SwitchStmt {
    pub scrutinee: Expr,
    pub cases: Vec<SwitchCase>,
    pub default: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SwitchCase {
    pub exprs: Vec<Expr>,
    /// Optional `if cond` guard: `case a, b if cond:` (Spec §3).
    /// A guarded case never falls through implicitly; the guard must be
    /// `bool`. Parity with `match` guards (`pattern if cond => body`).
    pub guard: Option<Expr>,
    pub body: Block,
    pub span: Span,
}

/// `do { ... } while (cond);` (Spec §3 SHOULD).
#[derive(Debug, Clone)]
pub struct DoWhileStmt {
    pub body: Block,
    pub condition: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub pattern: Pattern,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub label: Option<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub label: Option<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DeferStmt {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Call(CallExpr),
    Index(IndexExpr),
    FieldAccess(FieldAccessExpr),
    OptionalFieldAccess(OptionalFieldAccessExpr),
    MethodCall(MethodCallExpr),
    Literal(Literal),
    Ident(Ident),
    Lambda(Lambda),
    Block(Box<Block>),
    If(Box<IfExpr>),
    Match(Box<MatchExpr>),
    Try(Box<TryExpr>),
    Await(Box<AwaitExpr>),
    Cast(CastExpr),
    New(NewExpr),
    AsyncBlock(AsyncBlock),
    NullCoalesce(Box<Expr>, Box<Expr>),
    Range(Box<Expr>, Box<Expr>, bool),
    Tuple(Vec<Expr>),
    Attribute(AttributeExpr),
    With(WithExpr),
    Spawn(Box<SpawnExpr>),
    Select(Box<SelectExpr>),
    Option(OptionLiteral),
    Destructuring(DestructuringExpr),
    MacroCall(MacroCall),
    InterpolatedString(Vec<InterpolatedPart>),
    Spread(Box<Expr>, Span),
    /// `unsafe { ... }` block (Spec §32). Contents are checked with
    /// diagnostics still active; raw operations are confined here.
    UnsafeBlock(Box<Block>, Span),
}

#[derive(Debug, Clone)]
pub struct SpreadExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum InterpolatedPart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqEq,
    Neq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    AndAnd,
    OrOr,
    AmpAmp,
    PipePipe,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
    AddEq,
    SubEq,
    MulEq,
    DivEq,
    ModEq,
    Pipe,
}

#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnOp,
    pub operand: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
    BitNot,
    Ref,
    RefMut,
    Deref,
    Question,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub object: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct OptionalFieldAccessExpr {
    pub object: Box<Expr>,
    pub field: Ident,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpr {
    pub object: Box<Expr>,
    pub field: Ident,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    pub object: Box<Expr>,
    pub method: Ident,
    pub type_args: Vec<Type>,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Bool(bool),
    Int(i64, Option<IntSuffix>),
    Float(f64, Option<FloatSuffix>),
    String(String),
    Char(char),
    Array(Vec<Expr>),
    Tuple(Vec<Expr>),
    Unit,
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IntSuffix {
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatSuffix {
    F32,
    F64,
}

#[derive(Debug, Clone)]
pub struct Lambda {
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_expr: Box<Expr>,
    pub else_expr: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub expr: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct TryExpr {
    pub expr: Box<Expr>,
    pub catches: Vec<CatchClause>,
    pub finally: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CatchClause {
    pub pattern: Pattern,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AwaitExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct CastExpr {
    pub expr: Box<Expr>,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NewExpr {
    pub ty: Type,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AsyncBlock {
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct NullCoalesce {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub start: Box<Expr>,
    pub end: Box<Expr>,
    pub inclusive: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AttributeExpr {
    pub name: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariantPattern {
    pub name: Ident,
    pub args: Vec<Pattern>,
}

#[derive(Debug, Clone)]
pub struct NamedPattern {
    pub name: Ident,
    pub pattern: Box<Pattern>,
}

#[derive(Debug, Clone)]
pub struct LiteralPattern {
    pub literal: Literal,
}

#[derive(Debug, Clone)]
pub enum Type {
    Path(Path),
    Nullable(Box<Type>),
    Array(Box<Type>, Option<Box<Expr>>),
    Slice(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Function(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Reference(bool, Box<Type>),
    Pointer(bool, Box<Type>),
    Primitive(PrimitiveType),
    Generic(Path, Vec<Type>),
    Dynamic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Bool,
    Char,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Decimal,
    Void,
    String,
    Dynamic,
}

impl Type {
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Primitive(p) if matches!(p,
            PrimitiveType::I8 | PrimitiveType::I16 | PrimitiveType::I32 |
            PrimitiveType::I64 | PrimitiveType::I128 |
            PrimitiveType::U8 | PrimitiveType::U16 | PrimitiveType::U32 |
            PrimitiveType::U64 | PrimitiveType::U128 |
            PrimitiveType::F32 | PrimitiveType::F64 | PrimitiveType::Decimal
        ))
    }
}

#[derive(Debug, Clone)]
pub struct WithExpr {
    pub target: Box<Expr>,
    pub updates: Vec<FieldUpdate>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldUpdate {
    pub field: Ident,
    pub value: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SpawnExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SelectExpr {
    pub arms: Vec<SelectArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SelectArm {
    pub pattern: Pattern,
    pub guard: Option<Box<Expr>>,
    pub body: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum OptionLiteral {
    Some(Box<Expr>),
    None(Type),
}

#[derive(Debug, Clone)]
pub struct DestructuringExpr {
    pub pattern: Pattern,
    pub initializer: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MacroCall {
    pub name: Ident,
    pub args: Vec<Expr>,
    pub span: Span,
}
