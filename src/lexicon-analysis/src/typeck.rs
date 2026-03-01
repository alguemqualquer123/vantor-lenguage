use lexicon_parser::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Unit,
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
    String,
    Never,
    Unknown,
    Function(Vec<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Array(Box<Type>),
    Option(Box<Type>),
    Custom(String),
}

impl Type {
    pub fn from_ast(ty: &lexicon_parser::ast::Type) -> Self {
        match ty {
            lexicon_parser::ast::Type::Path(p) => {
                let name = p
                    .segments
                    .iter()
                    .map(|i| i.text.clone())
                    .collect::<Vec<_>>()
                    .join("::");
                match name.as_str() {
                    "i8" => Type::I8,
                    "i16" => Type::I16,
                    "i32" => Type::I32,
                    "i64" => Type::I64,
                    "i128" => Type::I128,
                    "u8" => Type::U8,
                    "u16" => Type::U16,
                    "u32" => Type::U32,
                    "u64" => Type::U64,
                    "u128" => Type::U128,
                    "f32" => Type::F32,
                    "f64" => Type::F64,
                    "bool" => Type::Bool,
                    "char" => Type::Char,
                    "String" => Type::String,
                    "void" | "()" => Type::Unit,
                    "never" => Type::Never,
                    _ => Type::Custom(name),
                }
            }
            lexicon_parser::ast::Type::Nullable(t) => Type::Option(Box::new(Type::from_ast(t))),
            lexicon_parser::ast::Type::Array(t, _) => Type::Array(Box::new(Type::from_ast(t))),
            lexicon_parser::ast::Type::Tuple(ts) => {
                Type::Tuple(ts.iter().map(Type::from_ast).collect())
            }
            lexicon_parser::ast::Type::Function(params, ret) => Type::Function(
                params.iter().map(Type::from_ast).collect(),
                Box::new(Type::from_ast(ret)),
            ),
            lexicon_parser::ast::Type::Reference(_, t) => Type::from_ast(t),
            lexicon_parser::ast::Type::Generic(p, _) => {
                let name = p
                    .segments
                    .iter()
                    .map(|i| i.text.clone())
                    .collect::<Vec<_>>()
                    .join("::");
                Type::Custom(name)
            }
            lexicon_parser::ast::Type::Pointer(_, t) => Type::from_ast(t),
            lexicon_parser::ast::Type::Dynamic => Type::Unknown,
            lexicon_parser::ast::Type::Primitive(p) => match p {
                lexicon_parser::ast::PrimitiveType::I8 => Type::I8,
                lexicon_parser::ast::PrimitiveType::I16 => Type::I16,
                lexicon_parser::ast::PrimitiveType::I32 => Type::I32,
                lexicon_parser::ast::PrimitiveType::I64 => Type::I64,
                lexicon_parser::ast::PrimitiveType::I128 => Type::I128,
                lexicon_parser::ast::PrimitiveType::U8 => Type::U8,
                lexicon_parser::ast::PrimitiveType::U16 => Type::U16,
                lexicon_parser::ast::PrimitiveType::U32 => Type::U32,
                lexicon_parser::ast::PrimitiveType::U64 => Type::U64,
                lexicon_parser::ast::PrimitiveType::U128 => Type::U128,
                lexicon_parser::ast::PrimitiveType::F32 => Type::F32,
                lexicon_parser::ast::PrimitiveType::F64 => Type::F64,
                lexicon_parser::ast::PrimitiveType::Bool => Type::Bool,
                lexicon_parser::ast::PrimitiveType::Char => Type::Char,
                lexicon_parser::ast::PrimitiveType::String => Type::String,
                lexicon_parser::ast::PrimitiveType::Void => Type::Unit,
                _ => Type::Unknown,
            },
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Type::Unit => "void".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::String => "String".to_string(),
            Type::Unknown => "unknown".to_string(),
            Type::Function(params, ret) => format!(
                "fn({}) -> {}",
                params
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
                ret.to_string()
            ),
            Type::Tuple(ts) => format!(
                "({})",
                ts.iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Type::Array(t) => format!("[{}]", t.to_string()),
            Type::Option(t) => format!("{}?", t.to_string()),
            Type::Custom(name) => name.clone(),
            _ => "?".to_string(),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::I8
                | Type::I16
                | Type::I32
                | Type::I64
                | Type::I128
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::U128
                | Type::F32
                | Type::F64
        )
    }
}

pub struct TypeChecker {
    scopes: Vec<HashMap<String, Type>>,
    current_fn_return: Option<Type>,
    errors: Vec<String>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut scopes = Vec::new();
        scopes.push(HashMap::new());

        scopes[0].insert("i8".to_string(), Type::I8);
        scopes[0].insert("i16".to_string(), Type::I16);
        scopes[0].insert("i32".to_string(), Type::I32);
        scopes[0].insert("i64".to_string(), Type::I64);
        scopes[0].insert("i128".to_string(), Type::I128);
        scopes[0].insert("u8".to_string(), Type::U8);
        scopes[0].insert("u16".to_string(), Type::U16);
        scopes[0].insert("u32".to_string(), Type::U32);
        scopes[0].insert("u64".to_string(), Type::U64);
        scopes[0].insert("u128".to_string(), Type::U128);
        scopes[0].insert("f32".to_string(), Type::F32);
        scopes[0].insert("f64".to_string(), Type::F64);
        scopes[0].insert("bool".to_string(), Type::Bool);
        scopes[0].insert("char".to_string(), Type::Char);
        scopes[0].insert("String".to_string(), Type::String);
        scopes[0].insert("void".to_string(), Type::Unit);

        TypeChecker {
            scopes,
            current_fn_return: None,
            errors: Vec::new(),
        }
    }

    pub fn check_module(&mut self, module: &Module) -> Result<(), Vec<String>> {
        self.push_scope();

        for decl in &module.declarations {
            self.check_decl(decl);
        }

        self.pop_scope();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_decl(&mut self, decl: &Decl) {
        match decl {
            Decl::Function(f) => {
                let ret = f
                    .return_type
                    .as_ref()
                    .map(|t| Type::from_ast(t))
                    .unwrap_or(Type::Unit);
                self.declare_var(
                    f.name.text.clone(),
                    Type::Function(
                        f.params.iter().map(|p| Type::from_ast(&p.ty)).collect(),
                        Box::new(ret.clone()),
                    ),
                );
            }
            Decl::Class(c) => {
                self.declare_var(c.name.text.clone(), Type::Custom(c.name.text.clone()));
            }
            Decl::Struct(s) => {
                self.declare_var(s.name.text.clone(), Type::Custom(s.name.text.clone()));
            }
            Decl::Enum(e) => {
                self.declare_var(e.name.text.clone(), Type::Custom(e.name.text.clone()));
            }
            _ => {}
        }
    }

    pub fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    fn declare_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
