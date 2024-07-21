use std::collections::HashMap;

use flux_diagnostics::ice;
use flux_span::{FileId, FileSpanned, InFile, Interner, Span, WithSpan, Word};

use crate::{
    r#trait::{ApplicationId, Trait, TraitApplication},
    r#type::{TypeId, TypeKind},
    scope::Scope,
    ConcreteKind, Path, ThisCtx, TraitId,
};

pub trait Insert<T> {
    fn insert(&mut self, ty: FileSpanned<T>) -> TypeId;
}

impl Insert<TypeKind> for TEnv {
    fn insert(&mut self, ty: FileSpanned<TypeKind>) -> TypeId {
        let idx = self.types.len();
        self.types.push(ty);
        TypeId::new(idx)
    }
}

#[derive(Debug)]
pub struct TEnv {
    types: Vec<FileSpanned<TypeKind>>,
    traits: Vec<Trait>,
    pub(super) trait_applications: HashMap<TraitId, Vec<TraitApplication>>,
    locals: Vec<Scope>,
    pub interner: &'static Interner,
}

macro_rules! insert_methods {
    ($($name:ident -> $path:literal),*) => {
        paste::paste! {
            $(
                pub fn [<insert_$name>](&mut self, file_id: FileId, span: Span) -> TypeId {
                    self.insert_simple_path($path, file_id, span)
                }
            )*
        }
    };
}

impl TEnv {
    pub fn new(interner: &'static Interner) -> Self {
        Self {
            types: vec![],
            traits: vec![],
            trait_applications: HashMap::new(),
            locals: vec![Scope::new()],
            interner,
        }
    }

    pub fn get(&self, tid: &TypeId) -> &FileSpanned<TypeKind> {
        self.types
            .get(tid.raw())
            .unwrap_or_else(|| ice(format!("typesystem could not get typekind with id {tid}")))
    }

    pub fn get_mut(&mut self, tid: &TypeId) -> &mut FileSpanned<TypeKind> {
        self.types
            .get_mut(tid.raw())
            .unwrap_or_else(|| ice(format!("typesystem could not get typekind with id {tid}")))
    }

    pub fn set(&mut self, tid: TypeId, tkind: TypeKind) {
        self.types[tid.raw()] = self.types[tid.raw()].map_inner_ref(|_| tkind);
    }

    fn insert_simple_path(&mut self, name: &'static str, file_id: FileId, span: Span) -> TypeId {
        self.insert(
            TypeKind::Concrete(ConcreteKind::Path(Path::new(
                vec![self.interner.get_or_intern_static(name)],
                vec![],
            )))
            .file_span(file_id, span),
        )
    }

    insert_methods!(
        s64 -> "s64",
        s32 -> "s32",
        s16 -> "s16",
        s8 -> "s8",
        u64 -> "u64",
        u32 -> "u32",
        u16 -> "u16",
        u8 -> "u8",
        f64 -> "f64",
        f32 -> "f32",
        str -> "str",
        bool -> "bool"
    );

    pub fn insert_unknown(&mut self, file_id: FileId, span: Span) -> TypeId {
        self.insert(TypeKind::Unknown.file_span(file_id, span))
    }

    pub fn insert_unit(&mut self, file_id: FileId, span: Span) -> TypeId {
        self.insert(TypeKind::Concrete(ConcreteKind::Tuple(vec![])).file_span(file_id, span))
    }

    pub fn insert_never(&mut self, file_id: FileId, span: Span) -> TypeId {
        self.insert(TypeKind::Never.file_span(file_id, span))
    }

    pub fn insert_trait(&mut self, trt: Trait) -> TraitId {
        let idx = self.traits.len();
        self.traits.push(trt);
        TraitId::new(idx)
    }

    pub fn insert_application(
        &mut self,
        trid: TraitId,
        application: TraitApplication,
    ) -> ApplicationId {
        let applications = self.trait_applications.entry(trid).or_insert(vec![]);
        let idx = applications.len() + 1;
        applications.push(application);

        debug_assert_ne!(idx, 0);
        ApplicationId::new(idx)
    }

    pub fn attach_this_ctx(&mut self, tid: &TypeId, this_ctx: ThisCtx) {
        let tid = self.get_inner_tid(tid);
        if let TypeKind::ThisPath(this_path) = &mut self.get_mut(&tid).inner.inner {
            this_path.this_ctx = this_ctx;
        }
    }

    pub fn get_application(&self, trid: &TraitId, aid: &ApplicationId) -> &TraitApplication {
        &self.trait_applications[trid][aid.raw() - 1]
    }

    pub fn make_ref(&mut self, tid: TypeId, new_span: Span) -> TypeId {
        self.insert(TypeKind::Ref(tid).file_span(self.get_fileid(&tid), new_span))
    }

    pub fn get_filespan(&self, tid: &TypeId) -> InFile<Span> {
        self.get(tid).to_file_span()
    }

    pub fn get_fileid(&self, tid: &TypeId) -> FileId {
        self.get(tid).file_id
    }

    pub fn get_span(&self, tid: &TypeId) -> Span {
        self.get(tid).inner.span
    }

    pub fn insert_local(&mut self, name: Word, tid: TypeId) {
        debug_assert!(!self.locals.is_empty());
        self.locals
            .last_mut()
            .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
            .insert_local(name, tid);
    }

    pub fn try_get_local(&mut self, name: &FileSpanned<Word>) -> Option<TypeId> {
        debug_assert!(!self.locals.is_empty());
        self.locals
            .last()
            .unwrap_or_else(|| ice("there should always be a scope on the stack in `TEnv`"))
            .try_get_local(name)
            // Duplicate the type to give a new filespan
            .map(|tid| self.insert(TypeKind::Ref(tid).file_span(name.file_id, name.span)))
    }

    pub fn debug_tid_dependency(&self, tid: TypeId) -> Vec<TypeId> {
        let mut tids = vec![tid];
        let mut cur = tid;
        while let TypeKind::Ref(parent_tid) = self.get(&cur).inner.inner {
            cur = parent_tid;
            tids.push(cur);
        }
        tids
    }

    fn get_inner_tid(&self, tid: &TypeId) -> TypeId {
        let mut inner_tid = tid;
        while let TypeKind::Ref(id) = &self.get(inner_tid).inner.inner {
            inner_tid = id;
        }
        *inner_tid
    }
}
