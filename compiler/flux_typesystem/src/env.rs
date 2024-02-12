use flux_diagnostics::ice;

use crate::r#type::{TypeId, TypeKind};

#[derive(Debug, Clone)]
pub struct TEnv {
    types: Vec<TypeKind>,
    // entries: Vec<FileSpanned<TEntry>>,
    // pub locals: Vec<Scope>,
    // pub(super) assoc_types: Vec<(FileSpanned<Spur>, TypeId)>,
    // pub(super) int_paths: HashSet<Spur>,
    // pub(super) float_paths: HashSet<Spur>,
}

impl TEnv {
    pub fn get(&self, id: &TypeId) -> &TypeKind {
        &self
            .types
            .get(id.raw())
            .unwrap_or_else(|| ice(format!("typesystem could not get typekind with id {id}")))
    }
}
