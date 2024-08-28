use flux_diagnostics::ice;
use flux_id::{id, Map};
use flux_util::{Interner, Span, Spanned, WithSpan, Word};

use crate::{
    r#trait::ThisCtx,
    r#type::{Restriction, ThisPath, Type},
    resolve::TraitResolver,
    scope::Scope,
    TraitApplication, TraitRestriction, TypeKind,
};

pub struct TEnv<'a> {
    pub(super) this_ctx: Option<ThisCtx>,
    pub(super) types: Map<id::Ty, Spanned<Type>>,
    pub(super) scopes: Vec<Scope>,
    trait_resolver: &'a TraitResolver,
    pub(super) interner: &'static Interner,
}

impl<'a> TEnv<'a> {
    pub fn new(trait_resolver: &'a TraitResolver, interner: &'static Interner) -> Self {
        Self {
            this_ctx: None,
            types: Map::new(),
            scopes: vec![Scope::new()],
            trait_resolver,
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

    pub fn get_inner(&self, tid: id::Ty) -> &Spanned<Type> {
        let mut tid = tid;
        while let TypeKind::Ref(id) = self.types.get(tid).kind {
            tid = id;
        }
        self.get(tid)
    }

    pub fn get_mut(&mut self, tid: id::Ty) -> &mut Spanned<Type> {
        self.types.get_mut(tid)
    }

    pub fn get_inner_mut(&mut self, tid: id::Ty) -> &mut Spanned<Type> {
        let mut tid = tid;
        while let TypeKind::Ref(id) = self.types.get(tid).kind {
            tid = id;
        }
        self.get_mut(tid)
    }

    pub fn add_equality(&mut self, a: id::Ty, b: id::Ty) {
        self.types
            .get_mut(a)
            .push_restriction(Restriction::Equals(b));
        self.types
            .get_mut(b)
            .push_restriction(Restriction::Equals(a));
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

    pub fn add_assoc_type_restriction(&mut self, tid: id::Ty, of: id::Ty, trt: TraitRestriction) {
        let ty = self.types.get_mut(tid);
        ty.push_restriction(Restriction::AssocTypeOf(of, trt));
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

    pub fn resolve_this_path<'b>(&'b self, this_path: &'b ThisPath) -> Vec<&TypeKind> {
        this_path
            .potential_this_ctx
            .iter()
            .filter_map(|this_ctx| match this_ctx {
                ThisCtx::Function | ThisCtx::TraitDecl => None,
                ThisCtx::TypeApplication(this) => Some(&**this),
                ThisCtx::TraitApplication(this, assoc_types) => match this_path.path.len() {
                    0 => Some(this),
                    1 => {
                        let name = this_path.path.get_nth(0);
                        let types = assoc_types
                            .iter()
                            .find_map(
                                |(assoc_name, ty)| {
                                    if assoc_name == name {
                                        Some(ty)
                                    } else {
                                        None
                                    }
                                },
                            )
                            .unwrap_or_else(|| {
                                ice(format!(
                                    "no associated type `{}` in `ThisCtx`: {:#?}",
                                    self.interner.resolve(name),
                                    this_path.potential_this_ctx
                                ))
                            });
                        Some(types)
                    }
                    2.. => unimplemented!(),
                },
            })
            .collect()
    }

    pub fn resolve_trait_restriction(
        &mut self,
        tid: id::Ty,
        trait_restriction: &TraitRestriction,
    ) -> Result<Vec<TraitApplication>, ()> {
        let to_ty = &self.get(tid).kind;

        if let Some(applications) = self.trait_resolver.traits.get(&trait_restriction.trait_id) {
            let potential_applications: Vec<_> = applications
                .iter()
                .filter(|app| {
                    let args_match = app
                        .args
                        .iter()
                        .zip(trait_restriction.args.iter())
                        .filter(|(a, b)| self.types_unify(a, &self.get(**b).kind))
                        .count()
                        == trait_restriction.args.len();
                    self.types_unify(&app.to, to_ty) && args_match
                })
                .cloned()
                .collect();

            let tkinds: Vec<_> = potential_applications
                .iter()
                .map(|app| &app.to)
                .cloned()
                .collect();
            if !tkinds.is_empty() {
                self.get_mut(tid)
                    .push_restriction(Restriction::EqualsOneOf(tkinds));
            }
            for (i, arg) in trait_restriction.args.iter().enumerate() {
                let tkinds: Vec<_> = potential_applications
                    .iter()
                    .map(|app| &app.args[i])
                    .cloned()
                    .collect();
                if !tkinds.is_empty() {
                    self.get_mut(*arg)
                        .push_restriction(Restriction::EqualsOneOf(tkinds));
                }
            }

            if !potential_applications.is_empty() {
                return Ok(potential_applications);
            }
        }

        Err(())
    }
}
