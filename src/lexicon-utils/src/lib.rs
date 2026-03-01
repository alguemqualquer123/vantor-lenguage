pub mod runtime;

pub use runtime::{
    benchmark, BufferedOutput, FastLoopExecutor, InternedStr, StringInterner, Value,
};

#[cfg(feature = "vm")]
pub use runtime::vm::{Chunk, OpCode, VM};
