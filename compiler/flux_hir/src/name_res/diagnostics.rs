use crate::hir::Path;

#[derive(Debug)]
pub(crate) enum ResolutionError {
    EmptyPath { path: Path },
    UnresolvedPath { path: Path, segment: usize },
    PrivateModule { path: Path, segment: usize },
}
