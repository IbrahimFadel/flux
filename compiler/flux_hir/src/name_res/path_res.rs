use crate::hir::Path;

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReachedFixedPoint {
    Yes,
    No,
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvePathResult {
    pub(crate) resolved_def: PerNs,
    pub(crate) segment_index: Option<usize>,
    pub(crate) reached_fixedpoint: ReachedFixedPoint,
}

impl ResolvePathResult {
    fn empty(reached_fixedpoint: ReachedFixedPoint) -> ResolvePathResult {
        ResolvePathResult::with(PerNs::none(), reached_fixedpoint, None)
    }

    fn with(
        resolved_def: PerNs,
        reached_fixedpoint: ReachedFixedPoint,
        segment_index: Option<usize>,
    ) -> ResolvePathResult {
        ResolvePathResult {
            resolved_def,
            segment_index,
            reached_fixedpoint,
        }
    }
}

impl DefMap {
    pub(crate) fn resolve_path(
        &self,
        path: &Path,
        original_module_id: LocalModuleId,
    ) -> ResolvePathResult {
        tracing::debug!(
            "resolving path {:?} in module {:?}",
            path,
            original_module_id
        );
        let mut segments = path.segments.iter().enumerate();
        let mut curr_per_ns = match segments.next() {
            Some((_, segment)) => self[original_module_id].scope.get(segment),
            None => return ResolvePathResult::empty(ReachedFixedPoint::Yes),
        };

        for (i, segment) in segments {
            let (curr, vis) = match curr_per_ns.take_types_vis() {
                Some(r) => r,
                None => return ResolvePathResult::empty(ReachedFixedPoint::No),
            };

            curr_per_ns = match curr {
                ModuleDefId::ModuleId(m) => self[m].scope.get(segment),
                s => {
                    return ResolvePathResult::with(
                        PerNs::types(s, vis),
                        ReachedFixedPoint::Yes,
                        Some(i),
                    )
                }
            }
        }
        ResolvePathResult::with(curr_per_ns, ReachedFixedPoint::Yes, None)
    }
}
