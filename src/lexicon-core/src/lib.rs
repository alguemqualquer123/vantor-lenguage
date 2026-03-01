pub mod error;
pub mod span;

pub use error::{Error, Result};
pub use span::Span;

pub type FileId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceId(pub FileId);

impl SourceId {
    pub fn new(id: FileId) -> Self {
        SourceId(id)
    }
}
