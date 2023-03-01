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
    pub(crate) reached_fixedpoint: ReachedFixedPoint,
}

impl ResolvePathResult {
    fn empty(reached_fixedpoint: ReachedFixedPoint) -> ResolvePathResult {
        ResolvePathResult::with(PerNs::none(), reached_fixedpoint)
    }

    fn with(
        resolved_def: PerNs,
        reached_fixedpoint: ReachedFixedPoint,
        // module_id: ModuleId,
    ) -> ResolvePathResult {
        ResolvePathResult {
            resolved_def,
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
        let mut segments = path.segments.iter();
        let name = match segments.next() {
            Some(segment) => segment,
            None => return ResolvePathResult::empty(ReachedFixedPoint::Yes),
        };
        let mut curr_per_ns = self[original_module_id].scope.get(name);

        for segment in segments {
            let (curr, m, vis) = match curr_per_ns.take_types_vis() {
                Some(r) => r,
                None => return ResolvePathResult::empty(ReachedFixedPoint::No),
            };

            curr_per_ns = match curr {
                ModuleDefId::ModuleId(m) => self[m].scope.get(segment),
                s => {
                    return ResolvePathResult::with(
                        PerNs::types(s, m, vis),
                        ReachedFixedPoint::Yes,
                    );
                }
            }
        }
        ResolvePathResult::with(curr_per_ns, ReachedFixedPoint::Yes)
    }
}
