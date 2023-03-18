use hashbrown::HashSet;
use lasso::Spur;

use crate::{hir::Visibility, ModuleDefId, ModuleId};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, ModuleId, Visibility)>,
    pub values: Option<(ModuleDefId, ModuleId, Visibility)>,
}

impl PerNs {
    pub fn none() -> PerNs {
        PerNs {
            types: None,
            values: None,
        }
    }

    pub fn values(t: ModuleDefId, m: ModuleId, v: Visibility) -> PerNs {
        PerNs {
            types: None,
            values: Some((t, m, v)),
        }
    }

    pub fn types(t: ModuleDefId, m: ModuleId, v: Visibility) -> PerNs {
        PerNs {
            types: Some((t, m, v)),
            values: None,
        }
    }

    pub fn from_def(def: ModuleDefId, m: ModuleId, v: Visibility) -> PerNs {
        match def {
            ModuleDefId::ApplyId(_) => todo!(),
            ModuleDefId::EnumId(_) => todo!(),
            ModuleDefId::FunctionId(_) => PerNs::values(def, m, v),
            ModuleDefId::ModuleId(_) => PerNs::types(def, m, v),
            ModuleDefId::StructId(_) => PerNs::types(def, m, v),
            ModuleDefId::TraitId(_) => PerNs::types(def, m, v),
            ModuleDefId::UseId(_) => todo!(),
            ModuleDefId::BuiltinType(_) => todo!(),
        }
    }

    pub fn with_visibility(self, vis: Visibility) -> PerNs {
        PerNs {
            types: self.types.map(|(it, m, _)| (it, m, vis)),
            values: self.values.map(|(it, m, _)| (it, m, vis)),
        }
    }

    pub fn take_types_vis(self) -> Option<(ModuleDefId, ModuleId, Visibility)> {
        self.types
    }

    pub fn take_types(self) -> Option<ModuleDefId> {
        self.types.map(|it| it.0)
    }

    pub fn take_values(self) -> Option<ModuleDefId> {
        self.values.map(|it| it.0)
    }

    pub fn take_values_vis(self) -> Option<(ModuleDefId, ModuleId, Visibility)> {
        self.values
    }

    pub fn or(self, other: PerNs) -> PerNs {
        PerNs {
            types: self.types.or(other.types),
            values: self.values.or(other.values),
        }
    }

    pub fn or_else(self, f: impl FnOnce() -> PerNs) -> PerNs {
        if self.is_full() {
            self
        } else {
            self.or(f())
        }
    }

    pub fn is_full(&self) -> bool {
        self.types.is_some() && self.values.is_some()
    }
}

#[derive(Debug, Default)]
pub struct PerNsGlobImports {
    pub types: HashSet<(ModuleId, Spur)>,
    pub values: HashSet<(ModuleId, Spur)>,
}
