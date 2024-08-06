use flux_diagnostics::ice;
use flux_id::{id, Map};
use flux_util::{Interner, Path, Span, Spanned, Word};

use crate::{
    r#trait::ThisCtx,
    r#type::{Restriction, Type},
    scope::Scope,
    TraitRestriction, TypeKind,
};

pub struct TEnv {
    pub(super) this_ctx: Option<ThisCtx>,
    pub(super) types: Map<id::Ty, Spanned<Type>>,
    pub(super) scopes: Vec<Scope>,
    pub(super) interner: &'static Interner,
}

impl TEnv {
    pub fn new(interner: &'static Interner) -> Self {
        Self {
            this_ctx: None,
            types: Map::new(),
            scopes: vec![Scope::new()],
            interner,
        }
    }

    pub fn set_this_ctx(&mut self, this_ctx: ThisCtx) {
        self.this_ctx = Some(this_ctx);
    }

    pub fn insert(&mut self, ty: Spanned<Type>) -> id::Ty {
        self.types.insert(ty)
    }

    pub fn get(&self, tid: id::Ty) -> &Spanned<Type> {
        self.types.get(tid)
    }

    pub fn get_mut(&mut self, tid: id::Ty) -> &mut Spanned<Type> {
        self.types.get_mut(tid)
    }

    pub fn add_equality(&mut self, a: id::Ty, b: id::Ty) {
        self.types
            .get_mut(a)
            .push_restriction(Restriction::Equals(b));
    }

    pub fn add_field_requirement(&mut self, tid: id::Ty, name: Word) {
        let ty = self.types.get_mut(tid);
        if !ty.has_field(&name) {
            ty.push_restriction(Restriction::Field(name));
        }
    }

    pub fn add_trait_restriction(&mut self, tid: id::Ty, restriction: TraitRestriction) {
        let ty = self.types.get_mut(tid);
        ty.push_restriction(Restriction::Trait(restriction));
    }

    pub fn insert_local(&mut self, name: Word, tid: id::Ty) {
        self.scopes
            .last_mut()
            .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
            .insert_local(name, tid);
    }

    pub fn try_get_local(&self, name: &Word) -> Option<&id::Ty> {
        self.scopes
            .last()
            .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
            .try_get_local(name)
    }

    pub fn try_get_local_by_tid(&self, tid: id::Ty) -> Option<Word> {
        self.scopes
            .last()
            .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
            .try_get_local_by_tid(tid)
    }

    pub fn get_span(&self, tid: id::Ty) -> Span {
        self.get(tid).span
    }

    pub fn resolve_this_path(&self, this_path: &Path<Word, Type>) -> id::Ty {
        let this_ctx = self
            .this_ctx
            .as_ref()
            .unwrap_or_else(|| ice("`TEnv` missing `ThisCtx` when resolving `ThisPath`"));
        let this_tid = this_ctx.this;

        match this_path.len() {
            0 => this_tid,
            1 => {
                let name = this_path.get_nth(0);
                this_ctx
                    .assoc_types
                    .iter()
                    .find_map(
                        |(assoc_name, ty)| {
                            if assoc_name == name {
                                Some(*ty)
                            } else {
                                None
                            }
                        },
                    )
                    .unwrap_or_else(|| {
                        ice(format!(
                            "no associated type `{}` in `ThisCtx`: {:#?}",
                            self.interner.resolve(name),
                            this_ctx
                        ))
                    })
            }
            2.. => unimplemented!(),
        }
    }
}

// pub struct TEnv<'res> {
//     pub(super) types: Map<id::Ty, FileSpanned<Type>>,
//     locals: Vec<Scope>,
//     pub trait_resolution: &'res TraitResolution,
//     pub(super) interner: &'static Interner,
//     unification_restrictions: Map<id::Ty, Vec<UnificationRestriction>>,
// }

// impl<'res> TEnv<'res> {
//     pub fn new(trait_resolution: &'res TraitResolution, interner: &'static Interner) -> Self {
//         Self {
//             types: Map::new(),
//             locals: vec![Scope::new()],
//             trait_resolution,
//             interner,
//             unification_restrictions: Map::new(),
//         }
//     }

//     pub fn insert(&mut self, ty: FileSpanned<Type>) -> id::Ty {
//         self.types.insert(ty)
//     }

//     pub fn get(&self, tid: id::Ty) -> &FileSpanned<Type> {
//         self.types.get(tid)
//     }

//     pub fn insert_unknown(&mut self, file_id: FileId, span: Span) -> id::Ty {
//         self.types.insert(Type::Unknown.file_span(file_id, span))
//     }

//     pub fn insert_unit(&mut self, file_id: FileId, span: Span) -> id::Ty {
//         self.types.insert(Type::unit().file_span(file_id, span))
//     }

//     pub fn insert_int(&mut self, file_id: FileId, span: Span) -> id::Ty {
//         self.types.insert(Type::int().file_span(file_id, span))
//     }

//     pub fn insert_float(&mut self, file_id: FileId, span: Span) -> id::Ty {
//         self.types.insert(Type::float().file_span(file_id, span))
//     }

//     pub fn insert_local(&mut self, name: Word, tid: id::Ty) {
//         self.locals
//             .last_mut()
//             .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
//             .insert_local(name, tid);
//     }

//     pub fn try_get_local(&self, name: &Word) -> Option<&id::Ty> {
//         self.locals
//             .last()
//             .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
//             .try_get_local(name)
//     }

//     pub fn push_unification_restriction(
//         &mut self,
//         ty: id::Ty,
//         restriction: UnificationRestriction,
//     ) {
//         trace!(
//             "'{} ({}) must unify with{}",
//             ty.raw(),
//             self.fmt_tid(ty),
//             match &restriction {
//                 UnificationRestriction::Or(types) => format!(
//                     " one of: {}",
//                     quote_and_listify(types.iter().map(|tid| self.fmt_tid(*tid)))
//                 ),
//                 UnificationRestriction::And(types) => format!(
//                     ": {}",
//                     quote_and_listify(types.iter().map(|tid| self.fmt_tid(*tid)))
//                 ),
//             }
//         );
//         self.unification_restrictions
//             .get_mut_or(vec![], ty)
//             .push(restriction)
//     }

//     pub fn get_filespan(&self, tid: id::Ty) -> InFile<Span> {
//         self.types
//             .try_get(tid)
//             .unwrap_or_else(|| ice("invalid `TypeId` when getting filespan"))
//             .to_file_span()
//     }

//     pub fn get_span(&self, tid: id::Ty) -> Span {
//         self.types
//             .try_get(tid)
//             .unwrap_or_else(|| ice("invalid `TypeId` when getting filespan"))
//             .span
//     }

//     pub(crate) fn push_arg_to_path(&mut self, tid: id::Ty, arg: Type) {
//         if let Type::Concrete(ConcreteKind::Path(path)) = &mut self.types.get_mut(tid).inner.inner {
//             path.args.push(arg);
//         } else {
//             ice("cannot push generic arg to non path type");
//         }
//     }

//     pub(crate) fn resolve_this_path<'a>(&'a self, this_path: &'a ThisPath) -> &Type {
//         match &this_path.ctx {
//             ThisCtx::TraitApplication((_, apply_id)) => {
//                 if this_path.path.len() == 0 {
//                     self.trait_resolution.get_this_type(apply_id)
//                 } else {
//                     self.trait_resolution
//                         .get_associated_type(this_path.path.get_nth(0), apply_id)
//                 }
//             }
//             ThisCtx::TypeApplication(apply_id) => {
//                 if this_path.path.len() != 0 {
//                     ice("`ThisPath` with `ThisCtx::TypeApplication` should have length 0");
//                 }
//                 self.trait_resolution.get_this_type(apply_id)
//             }
//             ThisCtx::TraitDecl(_) => ice("cannot resolve `ThisPath` with `ThisCtx::TraitDecl`"),
//             ThisCtx::None => ice("cannot resolve `ThisPath` with `ThisCtx::None`"),
//         }
//     }

//     pub fn get_valid_applications<'a>(
//         &'a self,
//         ty: &'a Type,
//         trait_id: P<id::TraitDecl>,
//         trait_args: &'a [Type],
//     ) -> impl Iterator<Item = &ApplicationInfo> + 'a {
//         self.trait_resolution
//             .get_trait_applications(&trait_id)
//             .iter()
//             .filter(|application_info| {
//                 let num_args = application_info.trait_args.len();
//                 let apply_ty = self
//                     .trait_resolution
//                     .get_this_type(&application_info.apply_id);

//                 self.unifies(ty, apply_ty)
//                     && num_args == trait_args.len()
//                     && application_info
//                         .trait_args
//                         .iter()
//                         .zip(application_info.trait_args.iter())
//                         .map(|(a, b)| self.unifies(a, b))
//                         .count()
//                         == num_args
//             })
//     }

//     fn get_trait_applications<'a>(&'a self, trait_id: &P<id::TraitDecl>, trait_args: &'a [Type]) {
//         todo!()
//         // self.trait_resolution
//         //     .trait_applications
//         //     .get(trait_id)
//         //     .unwrap_or_else(|| ice(format!("invalid `TraitId`: {:?}", trait_id)))
//         //     .iter()
//         //     .filter(|application| {
//         //         let args_len = application.trait_args.len();
//         //         args_len == trait_args.len()
//         //             && application
//         //                 .trait_args
//         //                 .iter()
//         //                 .zip(trait_args.iter())
//         //                 .filter(|(a, b)| self.unifies(a, b))
//         //                 .count()
//         //                 == args_len
//         //     })
//     }
// }

// /*

// Some notes:

// THERE SHOULD BE NO PUSHING GENERIC ARGS: USE UNKNOWN FOR ARGS NOT EXPLICITY WRITTEN BY USER

// Need to reword trait resolution so we can deal with   -> This::Output, and This, and checking fi a type implements a trait with given generic args

// */
