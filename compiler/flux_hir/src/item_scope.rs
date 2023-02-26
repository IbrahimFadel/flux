use hashbrown::HashMap;
use lasso::Spur;

use crate::{
    hir::Visibility,
    name_res::LocalModuleId,
    per_ns::{PerNs, PerNsGlobImports},
    ModuleDefId,
};

#[derive(Debug, Default)]
pub(crate) struct ItemScope {
    types: HashMap<Spur, (ModuleDefId, Visibility)>,
    values: HashMap<Spur, (ModuleDefId, Visibility)>,
    declarations: Vec<ModuleDefId>,
}

impl ItemScope {
    pub(crate) fn get(&self, name: &Spur) -> PerNs {
        PerNs {
            types: self.types.get(name).copied(),
            values: self.values.get(name).copied(),
        }
    }

    pub(crate) fn declare(&mut self, id: ModuleDefId) {
        self.declarations.push(id);
    }

    pub(crate) fn push_res_with_import(
        &mut self,
        glob_imports: &mut PerNsGlobImports,
        lookup: (LocalModuleId, Spur),
        def: PerNs,
    ) {
        if def.types.is_some() {
            glob_imports.types.insert(lookup);
        }
        if def.values.is_some() {
            glob_imports.values.insert(lookup);
        }
    }
}
