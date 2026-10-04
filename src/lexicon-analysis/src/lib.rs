pub mod typeck;
pub mod di;
pub mod hir;

pub use lexicon_parser::ast::Module;
pub use typeck::{cfg_enabled, eval_const, filter_cfg_decls, ConstValue, Type, TypeChecker};
pub use hir::{lower_to_hir, HirFunction, HirInstr, HirModule};
pub use di::{CONTAINER, DependencyContainer};
