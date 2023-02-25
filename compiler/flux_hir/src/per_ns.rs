use hashbrown::HashSet;
use lasso::Spur;

use crate::{hir::Visibility, name_res::LocalModuleId, ModuleDefId};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct PerNs {
    pub types: Option<(ModuleDefId, Visibility)>,
    pub values: Option<(ModuleDefId, Visibility)>,
}

impl PerNs {
    pub fn values(t: ModuleDefId, v: Visibility) -> PerNs {
        PerNs {
            types: None,
            values: Some((t, v)),
        }
    }

    pub fn types(t: ModuleDefId, v: Visibility) -> PerNs {
        PerNs {
            types: Some((t, v)),
            values: None,
        }
    }

    pub fn from_def(def: ModuleDefId, v: Visibility) -> PerNs {
        match def {
            ModuleDefId::ModuleId(_) => PerNs::types(def, v),
            ModuleDefId::FunctionId(_) => PerNs::values(def, v),
        }
    }

    pub fn with_visibility(self, vis: Visibility) -> PerNs {
        PerNs {
            types: self.types.map(|(it, _)| (it, vis)),
            values: self.values.map(|(it, _)| (it, vis)),
        }
    }
}

#[derive(Debug, Default)]
pub struct PerNsGlobImports {
    pub types: HashSet<(LocalModuleId, Spur)>,
    pub values: HashSet<(LocalModuleId, Spur)>,
}
