pub mod typeck;
pub mod di;

pub use lexicon_parser::ast::Module;
pub use typeck::{Type, TypeChecker};
pub use di::{CONTAINER, DependencyContainer};
