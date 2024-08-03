mod build;
mod input_file;
pub mod path;
mod span;

pub use build::*;
use cstree::interning::TokenKey;
pub use input_file::*;
use lasso::ThreadedRodeo;
pub use path::{Path, PathSegment};
pub use span::*;

pub type Interner = ThreadedRodeo<TokenKey>;
