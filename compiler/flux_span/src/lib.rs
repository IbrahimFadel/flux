mod build;
mod input_file;
mod span;

pub use build::*;
use cstree::interning::TokenKey;
pub use input_file::*;
use lasso::ThreadedRodeo;
pub use span::*;

pub type Interner = ThreadedRodeo<TokenKey>;
