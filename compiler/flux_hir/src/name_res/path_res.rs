use flux_span::{Span, Spanned};

use crate::hir::Path;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ResolvePathError {
    EmptyPath { path_span: Span },
    UnresolvedModule { path: Spanned<Path>, segment: usize },
    PrivateModule { path: Spanned<Path>, segment: usize },
}

impl ResolvePathError {
    pub fn to_lower_error(
        &self,
        file_id: FileId,
        string_interner: &'static ThreadedRodeo,
    ) -> LowerError {
        match self {
            Self::EmptyPath { path_span } => LowerError::CouldNotResolveEmptyPath {
                path_span: path_span.in_file(file_id),
            },
            Self::PrivateModule { path, segment } => LowerError::CannotAccessPrivatePathSegment {
                path: path
                    .map_ref(|path| path.to_string(string_interner))
                    .in_file(file_id),
                erroneous_segment: Path::spanned_segment(path, *segment, string_interner)
                    .unwrap()
                    .map(|spur| string_interner.resolve(&spur).to_string())
                    .in_file(file_id),
            },
            Self::UnresolvedModule { path, segment } => LowerError::CouldNotResolveUsePath {
                path: path
                    .map_ref(|path| path.to_string(string_interner))
                    .in_file(file_id),
                erroneous_segment: Path::spanned_segment(path, *segment, string_interner)
                    .unwrap()
                    .map(|spur| string_interner.resolve(&spur).to_string())
                    .in_file(file_id),
            },
        }
    }
}

impl DefMap {
    pub(crate) fn resolve_path(
        &self,
        path: &Spanned<Path>,
        original_module_id: LocalModuleId,
    ) -> Result<PerNs, ResolvePathError> {
        tracing::trace!(
            "resolving path {:?} in module {}",
            path,
            original_module_id.into_raw()
        );
        let mut segments = path.segments.iter().enumerate();
        let name = match segments.next() {
            Some((_, segment)) => segment,
            None => {
                return Err(ResolvePathError::EmptyPath {
                    path_span: path.span,
                })
            }
        };
        let mut curr_per_ns = self[original_module_id].scope.get(name);
        if curr_per_ns.types.is_none() && curr_per_ns.values.is_none() {
            return Err(ResolvePathError::UnresolvedModule {
                path: path.clone(),
                segment: 0,
            });
        }

        for (i, segment) in segments {
            let (curr, m, vis) = match curr_per_ns.take_types_vis() {
                Some((curr, m, vis)) => (curr, m, vis),
                None => {
                    return Err(ResolvePathError::UnresolvedModule {
                        path: path.clone(),
                        segment: i,
                    })
                }
            };

            curr_per_ns = match curr {
                ModuleDefId::ModuleId(m) => self[m].scope.get(segment),
                s => {
                    return Ok(PerNs::types(s, m, vis));
                }
            };
            if curr_per_ns.types.is_none() && curr_per_ns.values.is_none() {
                return Err(ResolvePathError::UnresolvedModule {
                    path: path.clone(),
                    segment: i,
                });
            } else if vis == Visibility::Private {
                return Err(ResolvePathError::PrivateModule {
                    path: path.clone(),
                    segment: i,
                });
            }
        }

        Ok(curr_per_ns)
    }
}
