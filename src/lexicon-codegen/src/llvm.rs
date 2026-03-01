use lexicon_analysis::Type as Ty;
use lexicon_analysis::TypeChecker;
use lexicon_parser::ast::{Decl, Function, Module, Param};
use lexicon_core::{Error, Result};

/// Backend responsável por transformar a AST em LLVM IR textual.
///
/// Este é um primeiro passo: gera apenas IR simples com
/// declarações de funções e um `main` mínimo.
pub struct LlvmBackend;

impl LlvmBackend {
    pub fn new() -> Self {
        LlvmBackend
    }

    /// Compila um módulo de AST para uma string de LLVM IR.
    ///
    /// Pipeline:
    /// - roda o type checker básico;
    /// - gera IR textual com prototypes e um `main` mínimo.
    pub fn compile_module_to_ir(&self, module: &Module) -> Result<String> {
        // 1. Type checking básico (usa o motor existente)
        let mut checker = TypeChecker::new();
        if let Err(errors) = checker.check_module(module) {
            let msg = errors.join("\n");
            return Err(Error::Type(format!("Type checking failed:\n{}", msg)));
        }

        // 2. Geração de IR textual simples
        let mut ir = String::new();
        ir.push_str("; Lexicon LLVM IR (preview)\n");
        ir.push_str("; Gerado pelo LlvmBackend inicial\n\n");

        // Mapa de funções, para decidir qual é o entrypoint
        let mut has_main = false;

        for decl in &module.declarations {
            if let Decl::Function(f) = decl {
                if f.name.text == "main" {
                    has_main = true;
                    ir.push_str(&self.emit_main_stub(f));
                    ir.push('\n');
                } else {
                    ir.push_str(&self.emit_function_decl(f));
                    ir.push('\n');
                }
            }
        }

        // Se não houver main explícito, ainda assim geramos um main vazio
        if !has_main {
            ir.push_str("define i32 @main() {\n  ret i32 0\n}\n");
        }

        Ok(ir)
    }

    fn emit_function_decl(&self, f: &Function) -> String {
        let ret_ty = f
            .return_type
            .as_ref()
            .map(|t| Ty::from_ast(t))
            .unwrap_or(Ty::Unit);
        let ret_ir = map_type_to_llvm(&ret_ty);
        let params_ir: Vec<String> = f.params.iter().map(param_to_llvm).collect();

        format!(
            "declare {} @{}({})",
            ret_ir,
            f.name.text,
            params_ir.join(", ")
        )
    }

    fn emit_main_stub(&self, f: &Function) -> String {
        // No futuro podemos mapear `fn main(args: List<String>)` etc.
        // Por enquanto, sempre gera `i32 @main()` que retorna 0.
        let _ = f;
        "define i32 @main() {\n  ret i32 0\n}\n".to_string()
    }
}

fn map_type_to_llvm(ty: &Ty) -> &str {
    use Ty::*;
    match ty {
        Bool => "i1",
        Char => "i8",
        I8 => "i8",
        I16 => "i16",
        I32 => "i32",
        I64 => "i64",
        I128 => "i128",
        U8 => "i8",
        U16 => "i16",
        U32 => "i32",
        U64 => "i64",
        U128 => "i128",
        F32 => "float",
        F64 => "double",
        String => "i8*",
        Unit | Never | Unknown => "void",
        Function(_, _) => "void*",
        Tuple(_) | Array(_) | Option(_) | Custom(_) => "void*",
    }
}

fn param_to_llvm(p: &Param) -> String {
    let ty = Ty::from_ast(&p.ty);
    let ir_ty = map_type_to_llvm(&ty);
    format!("{} %{}", ir_ty, p.name.text)
}