use hashbrown::{hash_map::Entry, HashMap};
use lasso::Spur;

use crate::{
    hir::Visibility,
    name_res::LocalModuleId,
    per_ns::{PerNs, PerNsGlobImports},
    ModuleDefId, ModuleId,
};

#[derive(Debug, Default)]
pub(crate) struct ItemScope {
    types: HashMap<Spur, (ModuleDefId, ModuleId, Visibility)>,
    values: HashMap<Spur, (ModuleDefId, ModuleId, Visibility)>,
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
        println!("pushing: {:#?} {:#?}", lookup, def);
        if let Some(value) = def.values {
            self.values.insert(lookup.1, value);
            glob_imports.values.insert(lookup);
        }
        if let Some(ty) = def.types {
            self.types.insert(lookup.1, ty);
            glob_imports.types.insert(lookup);
        }
        // macro_rules! check_changed {
        //     (
        //         $changed:ident,
        //         ( $this:ident / $def:ident ) . $field:ident,
        //         $glob_imports:ident [ $lookup:ident ],
        //     ) => {{
        //         if let Some(fld) = $def.$field {
        //             let existing = $this.$field.entry($lookup.1.clone());
        //             match existing {
        //                 Entry::Vacant(entry) => {
        //                     // match $def_import_type {
        //                     // ImportType::Glob => {
        //                     // $glob_imports.$field.insert($lookup.clone());
        //                     // }
        //                     // ImportType::Named => {
        //                     $glob_imports.$field.remove(&$lookup);
        //                     // }
        //                     // }

        //                     entry.insert(fld);
        //                 }
        //                 Entry::Occupied(mut entry) =>
        //                     // if matches!($def_import_type, ImportType::Named) =>
        //                 // {
        //                     if $glob_imports.$field.remove(&$lookup) {
        //                         entry.insert(fld);
        //                     }
        //                 // }
        //                 _ => {}
        //             }
        //         }
        //     }};
        // }

        // check_changed!(changed, (self / def).types, glob_imports[lookup],);
        // check_changed!(changed, (self / def).values, glob_imports[lookup],);

        // if def.is_none() && self.unresolved.insert(lookup.1) {
        //     changed = true;
        // }

        // changed
        // if let Some(value) = def.values {
        //     self.values.insert(lookup.1, value);
        //     glob_imports.values.insert(lookup);
        // }
        // if let Some(ty) = def.types {
        //     self.types.insert(lookup.1, ty);
        //     glob_imports.types.insert(lookup);
        // }
    }
}
