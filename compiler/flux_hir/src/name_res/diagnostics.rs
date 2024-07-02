use crate::hir::Path;

pub(crate) enum ResolutionError {
    EmptyPath { path: Path },
    UnresolvedPath { path: Path, segment: usize },
    PrivateModule { path: Path, segment: usize },
}
