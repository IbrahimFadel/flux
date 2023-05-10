// use flux_diagnostics::Diagnostic;
// use flux_typesystem::{TChecker, TypeId};
// use la_arena::ArenaMap;

// use crate::TraitId;

// pub struct TraitApplicationTable {
//     table: ArenaMap<TraitId, Vec<TraitApplication>>,
// }

// impl TraitApplicationTable {
//     pub fn new() -> Self {
//         Self {
//             table: ArenaMap::default(),
//         }
//     }

//     pub fn push_application(&mut self, trait_id: TraitId, application: TraitApplication) {
//         self.table.entry(trait_id).or_default().push(application);
//     }

//     pub fn verify_no_duplicate_applications(
//         &self,
//         trait_id: TraitId,
//         tchk: &TChecker,
//     ) -> Result<(), Diagnostic> {
//         match self.table.get(trait_id) {
//             Some(applications) => {
//                 applications.iter().for_each(|application| {});
//                 Ok(())
//             }
//             None => Ok(()),
//         }
//     }
// }

// pub struct TraitApplication {
//     /// First `TypeId` is the implementor, followed by the trait parameters, then the implementor parameters
//     ids: Vec<TypeId>,
//     num_trait_params: usize,
// }

// impl TraitApplication {
//     pub(crate) fn new(
//         trait_params: Vec<TypeId>,
//         impltor: TypeId,
//         impltor_args: Vec<TypeId>,
//     ) -> Self {
//         let num_trait_params = trait_params.len();
//         Self {
//             ids: std::iter::once(impltor)
//                 .chain(trait_params)
//                 .chain(impltor_args)
//                 .collect(),
//             num_trait_params,
//         }
//     }

//     pub(crate) fn get_impltor(&self) -> TypeId {
//         unsafe { *self.ids.get_unchecked(0) }
//     }

//     pub(crate) fn get_trait_params(&self) -> Vec<TypeId> {
//         if self.num_trait_params == 0 {
//             vec![]
//         } else {
//             unsafe {
//                 self.ids
//                     .get_unchecked(1..self.num_trait_params + 1)
//                     .to_vec()
//             }
//         }
//     }

//     pub(crate) fn get_impltor_params(&self) -> Vec<TypeId> {
//         self.ids
//             .get(self.num_trait_params..)
//             .map(Vec::from)
//             .unwrap_or_else(Vec::new)
//     }
// }
