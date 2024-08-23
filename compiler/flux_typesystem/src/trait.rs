use std::collections::HashMap;

use flux_id::id::{self, InPkg};
use flux_util::Word;

use crate::TypeKind;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ThisCtx {
    Function,
    TypeApplication(Box<TypeKind>),
    TraitApplication(Box<TypeKind>, Vec<(Word, TypeKind)>),
}

#[derive(Debug, Clone)]
pub struct TraitApplication {
    pub to: TypeKind,
    pub args: Vec<TypeKind>,
}

impl TraitApplication {
    pub fn new(to: TypeKind, args: Vec<TypeKind>) -> Self {
        Self { to, args }
    }
}

pub struct TraitResolver {
    traits: HashMap<InPkg<id::TraitDecl>, Vec<TraitApplication>>,
}

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub struct ThisCtx {
//     pub(super) this: id::Ty,
//     pub(super) assoc_types: Vec<(Word, id::Ty)>,
// }

// impl ThisCtx {
//     pub const fn new(this: id::Ty, assoc_types: Vec<(Word, id::Ty)>) -> Self {
//         Self { this, assoc_types }
//     }
// }

// use std::collections::HashMap;

// use flux_diagnostics::ice;
// use flux_id::id::{self, P};
// use flux_util::Word;

// use crate::r#type::Type;

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub enum ThisCtx {
//     TraitDecl(P<id::TraitDecl>),
//     TraitApplication((P<id::TraitDecl>, P<id::ApplyDecl>)),
//     TypeApplication(P<id::ApplyDecl>),
//     None,
// }

// pub struct ApplicationTypes {
//     ty: Type,
//     assoc_types: Vec<(Word, Type)>,
// }

// impl ApplicationTypes {
//     pub fn new(ty: Type, assoc_types: Vec<(Word, Type)>) -> Self {
//         Self { ty, assoc_types }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct ApplicationInfo {
//     pub(super) trait_args: Vec<Type>,
//     pub apply_id: P<id::ApplyDecl>,
// }

// impl ApplicationInfo {
//     pub fn new(trait_args: Vec<Type>, apply_id: P<id::ApplyDecl>) -> Self {
//         Self {
//             trait_args,
//             apply_id,
//         }
//     }
// }

// pub struct TraitResolution {
//     applications: HashMap<P<id::ApplyDecl>, ApplicationTypes>,
//     traits: HashMap<P<id::TraitDecl>, Vec<ApplicationInfo>>,
// }

// impl TraitResolution {
//     pub fn new(
//         applications: HashMap<P<id::ApplyDecl>, ApplicationTypes>,
//         traits: HashMap<P<id::TraitDecl>, Vec<ApplicationInfo>>,
//     ) -> Self {
//         Self {
//             applications,
//             traits,
//         }
//     }

//     pub fn get_this_type(&self, apply_id: &P<id::ApplyDecl>) -> &Type {
//         &self
//             .applications
//             .get(apply_id)
//             .unwrap_or_else(|| ice(format!("invalid `ApplyId`: {:?}", apply_id)))
//             .ty
//     }

//     pub fn get_associated_type(&self, name: &Word, apply_id: &P<id::ApplyDecl>) -> &Type {
//         self.applications
//             .get(apply_id)
//             .unwrap_or_else(|| ice(format!("invalid `ApplyId`: {:?}", apply_id)))
//             .assoc_types
//             .iter()
//             .find_map(|(assoc_name, ty)| if assoc_name == name { Some(ty) } else { None })
//             .unwrap_or_else(|| ice("no such associated type"))
//     }

//     pub fn get_trait_applications(&self, trait_id: &P<id::TraitDecl>) -> &[ApplicationInfo] {
//         &self
//             .traits
//             .get(trait_id)
//             .unwrap_or_else(|| ice(format!("invalid `TraitId`: {:?}", trait_id)))
//     }
// }

// // pub struct TraitApplication {
// //     pub(super) trait_args: Vec<Type>,
// //     pub(super) apply_id: P<id::ApplyDecl>,
// //     pub(super) info: ApplicationInfo,
// // }

// // impl TraitApplication {
// //     pub fn new(trait_args: Vec<Type>, apply_id: P<id::ApplyDecl>, info: ApplicationInfo) -> Self {
// //         Self {
// //             trait_args,
// //             apply_id,
// //             info,
// //         }
// //     }
// // }

// // pub struct TraitResolution {
// //     this_types: HashMap<P<id::ApplyDecl>, Type>,
// //     pub(super) trait_applications: HashMap<P<id::TraitDecl>, Vec<TraitApplication>>,
// // }

// // impl TraitResolution {
// //     pub fn new(
// //         this_types: HashMap<P<id::ApplyDecl>, Type>,
// //         trait_applications: HashMap<P<id::TraitDecl>, Vec<TraitApplication>>,
// //     ) -> Self {
// //         Self {
// //             // traits,
// //             this_types,
// //             trait_applications,
// //         }
// //     }

// //     pub fn get_this_type(&self, apply_id: &P<id::ApplyDecl>) -> &Type {
// //         self.this_types
// //             .get(apply_id)
// //             .unwrap_or_else(|| ice(format!("invalid `ApplyId`: {:?}", apply_id)))
// //     }

// //     // pub fn get_trait_applications(
// //     //     &self,
// //     //     trait_id: &P<id::TraitDecl>,
// //     //     trait_args: &[Type],
// //     //     // ) -> impl Iterator<Item = &(P<id::ApplyDecl>, ApplicationInfo)> {
// //     // ) {

// //     // }

// //     // pub fn get_trait_application(
// //     //     &self,
// //     //     trait_id: &P<id::TraitDecl>,
// //     //     trait_args: &[Type],
// //     //     apply_id: &P<id::ApplyDecl>,
// //     // ) -> &ApplicationInfo {
// //     //     // self.get_trait_application(trait_id, apply_id)
// //     //     // todo!()
// //     //     // self.get_trait_applications(trait_id)
// //     //     //     .find_map(|(application, app)| {
// //     //     //         if application == apply_id {
// //     //     //             Some(app)
// //     //     //         } else {
// //     //     //             None
// //     //     //         }
// //     //     //     })
// //     //     //     .unwrap_or_else(|| ice(format!("invalid `ApplyId`: {:?}", apply_id)))
// //     // }
// // }

// // pub struct ApplicationInfo {
// //     pub(crate) ty: Type,
// //     assoc_types: Vec<(Word, Type)>,
// // }

// // impl ApplicationInfo {
// //     pub fn new(ty: Type, assoc_types: Vec<(Word, Type)>) -> Self {
// //         Self { ty, assoc_types }
// //     }

// //     pub(crate) fn get_associated_type(&self, this_path: &Path<Word, Type>) -> &Type {
// //         if this_path.len() != 1 {
// //             ice("`ThisPath` of associated type should have length of 1");
// //         }
// //         self.assoc_types
// //             .iter()
// //             .find_map(|(name, ty)| {
// //                 if name == this_path.get_nth(0) {
// //                     Some(ty)
// //                 } else {
// //                     None
// //                 }
// //             })
// //             .unwrap_or_else(|| ice(format!("no associated type `{:?}`", this_path.get_nth(0))))
// //     }
// // }
