use flux_diagnostics::ice;
use flux_id::{id, Map};
use flux_util::{FileId, FileSpanned, InFile, Interner, Span, WithSpan, Word};

use crate::{
    r#trait::{ThisCtx, TraitResolution},
    r#type::{ThisPath, Type},
    scope::Scope,
    ConcreteKind,
};

pub struct TEnv<'res> {
    pub(super) types: Map<id::Ty, FileSpanned<Type>>,
    locals: Vec<Scope>,
    trait_resolution: &'res TraitResolution,
    pub(super) interner: &'static Interner,
}

impl<'res> TEnv<'res> {
    pub fn new(trait_resolution: &'res TraitResolution, interner: &'static Interner) -> Self {
        Self {
            types: Map::new(),
            locals: vec![Scope::new()],
            trait_resolution,
            interner,
        }
    }

    pub fn insert(&mut self, ty: FileSpanned<Type>) -> id::Ty {
        self.types.insert(ty)
    }

    pub fn insert_unknown(&mut self, file_id: FileId, span: Span) -> id::Ty {
        self.types.insert(Type::Unknown.file_span(file_id, span))
    }

    pub fn insert_unit(&mut self, file_id: FileId, span: Span) -> id::Ty {
        self.types.insert(Type::unit().file_span(file_id, span))
    }

    pub fn insert_int(&mut self, file_id: FileId, span: Span) -> id::Ty {
        self.types.insert(Type::int().file_span(file_id, span))
    }

    pub fn insert_float(&mut self, file_id: FileId, span: Span) -> id::Ty {
        self.types.insert(Type::float().file_span(file_id, span))
    }

    pub fn insert_local(&mut self, name: Word, tid: id::Ty) {
        self.locals
            .last_mut()
            .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
            .insert_local(name, tid);
    }

    pub fn try_get_local(&self, name: &Word) -> Option<&id::Ty> {
        self.locals
            .last()
            .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
            .try_get_local(name)
    }

    pub fn get_filespan(&self, tid: id::Ty) -> InFile<Span> {
        self.types
            .try_get(tid)
            .unwrap_or_else(|| ice("invalid `TypeId` when getting filespan"))
            .to_file_span()
    }

    pub fn get_span(&self, tid: id::Ty) -> Span {
        self.types
            .try_get(tid)
            .unwrap_or_else(|| ice("invalid `TypeId` when getting filespan"))
            .span
    }

    pub(crate) fn push_arg_to_path(&mut self, tid: id::Ty, arg: Type) {
        if let Type::Concrete(ConcreteKind::Path(path)) = &mut self.types.get_mut(tid).inner.inner {
            path.args.push(arg);
        } else {
            ice("cannot push generic arg to non path type");
        }
    }

    pub(crate) fn resolve_this_path(&self, this_path: &ThisPath) -> &Type {
        match &this_path.ctx {
            ThisCtx::TraitApplication((trait_id, apply_id)) => {
                if this_path.path.len() == 0 {
                    self.trait_resolution.get_this_type(apply_id)
                } else {
                    self.trait_resolution
                        .get_trait_application(trait_id, apply_id)
                        .get_associated_type(&this_path.path)
                }
            }
            ThisCtx::TypeApplication(apply_id) => {
                if this_path.path.len() != 0 {
                    ice("`ThisPath` with `ThisCtx::TypeApplication` should have length 0");
                }
                self.trait_resolution.get_this_type(apply_id)
            }
            ThisCtx::TraitDecl(_) => ice("cannot resolve `ThisPath` with `ThisCtx::TraitDecl`"),
            ThisCtx::None => ice("cannot resolve `ThisPath` with `ThisCtx::None`"),
        }
    }
}
