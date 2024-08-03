use std::convert::Infallible;

use flux_id::id;

use crate::name_res::diagnostics::ResolutionError;

#[derive(Debug, Clone)]
pub struct Import {
    pub module_id: id::Mod,
    pub use_id: id::UseDecl,
}

#[derive(Debug, Clone)]
pub enum PartialResolvedImport {
    Unresolved(ResolutionError<Infallible>),
    Resolved(()),
}
