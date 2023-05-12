use std::{collections::HashMap, fmt::Display};

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};
use itertools::Itertools;
use lasso::{Spur, ThreadedRodeo};
use owo_colors::OwoColorize;
use std::collections::HashSet;

use crate::{
    diagnostics::TypeError,
    r#type::{ConcreteKind, Type, TypeId, TypeKind},
    scope::Scope,
};

#[derive(Debug)]
pub struct TEnv {
    pub string_interner: &'static ThreadedRodeo,
    entries: Vec<FileSpanned<TEntry>>,
    pub locals: Vec<Scope>,
    pub(super) int_paths: HashSet<Spur>,
    pub(super) float_paths: HashSet<Spur>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TraitRestriction {
    pub trait_id: u32,
    pub trait_name: FileSpanned<Spur>,
    pub args: Vec<TypeId>,
}

impl TraitRestriction {
    pub fn new(trait_id: u32, trait_name: FileSpanned<Spur>, args: Vec<TypeId>) -> Self {
        Self {
            trait_id,
            trait_name,
            args,
        }
    }
}

/// A `flux_typesystem` type entry
///
/// Stores the [`Key`] of the type constructor and a list of constraints
#[derive(Debug, Clone)]
pub struct TEntry {
    keys: Vec<TypeKind>,
}

impl TEntry {
    pub fn new(constr: TypeKind) -> Self {
        Self { keys: vec![constr] }
    }

    pub fn with_args(constr: TypeKind, args: impl Iterator<Item = TypeKind>) -> Self {
        let keys = std::iter::once(constr).chain(args).collect();
        Self { keys }
    }

    pub fn get_constr(&self) -> &TypeKind {
        &self.keys[0]
    }

    pub fn get_params(&self) -> Option<&[TypeKind]> {
        self.keys.get(1..)
    }

    pub fn set_constr(&mut self, key: TypeKind) {
        self.keys[0] = key;
    }

    pub fn set_params(&mut self, params: impl Iterator<Item = TypeKind>) {
        self.keys.splice(1.., params.collect::<Vec<_>>());
    }
}

impl TEnv {
    /// Construct a new `flux_typesystem` [`TEnv`]
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            string_interner,
            entries: vec![],
            // Global scope
            locals: vec![Scope::new()],
            int_paths: HashSet::from([
                string_interner.get_or_intern_static("u8"),
                string_interner.get_or_intern_static("u16"),
                string_interner.get_or_intern_static("u32"),
                string_interner.get_or_intern_static("u64"),
                string_interner.get_or_intern_static("s8"),
                string_interner.get_or_intern_static("s16"),
                string_interner.get_or_intern_static("s32"),
                string_interner.get_or_intern_static("s64"),
            ]),
            float_paths: HashSet::from([
                string_interner.get_or_intern_static("f32"),
                string_interner.get_or_intern_static("f64"),
            ]),
        }
    }

    /// Get a `FileSpanned<TEntry>` given a [`TypeId`]
    ///
    /// If the entry does not exist, the [`TypeId`] is invalid, which is considered an ICE.
    pub fn get_entry(&self, id: TypeId) -> &FileSpanned<TEntry> {
        self.entries
            .get(id.get())
            .expect("internal compiler error: tried retrieving type with invalid type id")
    }

    /// Get a `&mut FileSpanned<TEntry>` given a [`TypeId`]
    ///
    /// If the entry does not exist, the [`TypeId`] is invalid, which is considered an ICE.
    pub fn get_entry_mut(&mut self, id: TypeId) -> &mut FileSpanned<TEntry> {
        self.entries
            .get_mut(id.get())
            .expect("internal compiler error: tried retrieving type with invalid type id")
    }

    /// Get a `Spanned<TypeKind>` given a [`TypeId`]
    ///
    /// If the [`TEntry`] for the given [`TypeId`] does not exist, the [`TypeId`] is invalid, which
    /// is considered an ICE.
    pub fn get_typekind_with_id(&self, id: TypeId) -> FileSpanned<TypeKind> {
        let entry = self.get_entry(id);
        entry.map_inner_ref(|entry| entry.get_constr().clone())
        // entry.map_ref(|entry| entry.map_ref(|entry| self.type_interner.resolve(entry.key).clone()))
    }

    /// Get the `InFile<Span>` of a [`Type`] given its [`TypeId`]
    ///
    /// If there is no [`TEntry`] associated with the given [`TypeId`] in the [`TEnv`], the
    /// [`TypeId`] is invalid, which is considered an ICE.
    pub fn get_type_filespan(&self, id: TypeId) -> InFile<Span> {
        let entry = self.get_entry(id);
        entry.map_ref(|spanned| spanned.span)
    }

    pub fn get_call_return_type(&self, _path: &[Spur]) -> TypeId {
        todo!()
    }

    /// Insert a unit type `()` into the [`TEnv`]
    #[inline]
    pub fn insert_unit(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Concrete(ConcreteKind::Tuple(vec![])));
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert an unknown type into the [`TEnv`]
    #[inline]
    pub fn insert_unknown(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Unknown);
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert an int type into the [`TEnv`]
    #[inline]
    pub fn insert_int(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Int(None));
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert a float type into the [`TEnv`]
    #[inline]
    pub fn insert_float(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Float(None));
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert a bool type into the [`TEnv`]
    #[inline]
    pub fn insert_bool(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Concrete(ConcreteKind::Path(
            self.string_interner.get_or_intern_static("bool"),
        )));
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    #[inline]
    pub fn insert_str(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Concrete(ConcreteKind::Path(
            self.string_interner.get_or_intern_static("str"),
        )));
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert a `Spanned<Type>` into the type environment
    ///
    /// This interns the [`TypeKind`] and pushes a new [`TEntry`] to th environment, returning
    /// a valid [`TypeId`]
    pub fn insert(&mut self, ty: FileSpanned<Type>) -> TypeId {
        let kind = ty.constr();
        let idx = self.entries.len();
        self.entries
            .push(ty.map_inner_ref(|_| TEntry::new(kind.clone())));
        TypeId::new(idx)
    }

    pub fn set_type(&mut self, id: TypeId, ty: Type) {
        let kind = ty.constr();
        self.get_entry_mut(id).inner.inner.set_constr(kind.clone());
        if let Some(kinds) = ty.params() {
            self.get_entry_mut(id)
                .inner
                .inner
                .set_params(kinds.iter().cloned());
        };
    }

    /// Insert a local to the current [`Scope`]
    pub fn insert_local_to_scope(&mut self, name: Spur, id: TypeId) {
        self.locals
            .last_mut()
            .expect("internal compiler error: no active scope in type environment")
            .insert_local(name, id);
    }

    /// Get the [`TypeId`] of a path in any currently accessible [`Scope`]
    pub fn get_local_typeid(
        &mut self,
        name: FileSpanned<Spur>,
        // path: FileSpanned<impl Iterator<Item = Spur>>,
    ) -> Result<TypeId, Diagnostic> {
        self.locals
            .last()
            .expect("internal compiler error: no active scope in type environment")
            .try_get_local(&name)
            .map_or_else(
                || {
                    Err(TypeError::UnknownVariable {
                        name: self.string_interner.resolve(&name).to_string(),
                        name_file_span: name.to_filespan(),
                    }
                    .to_diagnostic())
                },
                Ok,
            )
    }

    pub fn reconstruct(&self, tid: TypeId) -> Result<TypeKind, Diagnostic> {
        let tkind = self.get_typekind_with_id(tid);
        match &tkind.inner.inner {
            TypeKind::Concrete(_) => Ok(tkind.inner.inner),
            TypeKind::Float(float) => match float {
                Some(id) => self.reconstruct(*id),
                None => Ok(TypeKind::Concrete(ConcreteKind::Path(
                    self.string_interner.get_or_intern_static("f32"),
                ))),
            },
            TypeKind::Generic(_, _) => todo!(),
            TypeKind::Int(int) => match int {
                Some(id) => self.reconstruct(*id),
                None => Ok(TypeKind::Concrete(ConcreteKind::Path(
                    self.string_interner.get_or_intern_static("u32"),
                ))),
            },
            TypeKind::Ref(id) => self.reconstruct(*id),
            TypeKind::Never => todo!(),
            TypeKind::Unknown => Err(TypeError::CouldNotInferType {
                ty: (),
                ty_file_span: self.get_type_filespan(tid),
            }
            .to_diagnostic()),
        }
    }

    pub fn reconstruct_concrete(&self, concrete: &ConcreteKind) -> Result<TypeKind, Diagnostic> {
        match concrete {
            ConcreteKind::Array(_, _) => todo!(),
            ConcreteKind::Ptr(_) => todo!(),
            ConcreteKind::Path(_) => todo!(),
            ConcreteKind::Tuple(_) => todo!(),
        }
    }

    /// Format a `flux_typesystem` [`TypeId`] to a `String`
    ///
    /// Not to be confused with [`TypeId`]'s `Display` implementation which prints the `usize`
    /// representation preceded by `'`. This method retreives the [`TypeKind`] from the
    /// [`Interner`] and formats it using information stored in the [`TEnv`]. This formatting is
    /// not possible with a simple `Display` implementation as some [`TypeKind`]s store things such
    /// as [`TypeId`]s which need access to the [`TEnv`] in order to be formatted prettily.
    ///
    /// This method is good for debugging, and error messages.
    pub fn fmt_ty_id(&self, id: TypeId) -> String {
        let typekind = self.get_typekind_with_id(id);
        self.fmt_typekind(&typekind.inner.inner)
    }

    pub fn fmt_ty_id_constr(&self, id: TypeId) -> String {
        let typekind = self.get_typekind_with_id(id);
        self.fmt_typekind_constr(&typekind.inner.inner)
    }

    pub fn fmt_typekind_constr(&self, kind: &TypeKind) -> String {
        match kind {
            TypeKind::Generic(name, _) => self.string_interner.resolve(name).to_string(),
            TypeKind::Unknown => "unknown".to_string(),
            TypeKind::Int(_) => "int".to_string(),
            TypeKind::Float(_) => "float".to_string(),
            TypeKind::Ref(id) => self.fmt_ty_id_constr(*id),
            TypeKind::Never => "!".to_string(),
            TypeKind::Concrete(concrete) => self.fmt_concrete_kind_constr(concrete),
        }
    }

    fn fmt_concrete_kind_constr(&self, kind: &ConcreteKind) -> String {
        match kind {
            ConcreteKind::Array(ty, n) => format!("[{}; {}]", self.fmt_ty_id_constr(*ty), n),
            ConcreteKind::Ptr(id) => format!("*{}", self.fmt_ty_id_constr(*id)),
            ConcreteKind::Path(spur) => self.string_interner.resolve(spur).to_string(),
            ConcreteKind::Tuple(ids) => {
                format!(
                    "({})",
                    ids.iter().map(|id| self.fmt_ty_id_constr(*id)).join(", ")
                )
            }
        }
    }

    pub fn fmt_typekind(&self, kind: &TypeKind) -> String {
        match kind {
            TypeKind::Generic(name, restrictions) => format!(
                "{}{}",
                self.string_interner.resolve(name),
                if restrictions.is_empty() {
                    "".to_string()
                } else {
                    format!(
                        ": {}",
                        restrictions
                            .iter()
                            .map(|restriction| format!(
                                "{}{}",
                                self.string_interner.resolve(&restriction.trait_name),
                                if restriction.args.is_empty() {
                                    "".to_string()
                                } else {
                                    restriction
                                        .args
                                        .iter()
                                        .map(|arg| self.fmt_ty_id(*arg))
                                        .join(", ")
                                }
                            ))
                            .join(", ")
                    )
                }
            ),
            TypeKind::Unknown => "unknown".to_string(),
            TypeKind::Int(_) => "int".to_string(),
            TypeKind::Float(_) => "float".to_string(),
            TypeKind::Ref(id) => self.fmt_ty_id(*id),
            TypeKind::Never => "!".to_string(),
            TypeKind::Concrete(concrete) => self.fmt_concrete_kind(concrete),
        }
    }

    /// Format a `flux_typesystem` [`TypeId`] to a `String`
    ///
    /// Not to be confused with the [`ConcreteKind`] `Display` implementation which will format the
    /// kind to the best of its ability without having any information about the [`TEnv`]. This
    /// method is able to format [`TypeId`]s, and therefore compound types such as pointers, and
    /// tuples, as well as types which require access to the string interner, such as paths.
    fn fmt_concrete_kind(&self, kind: &ConcreteKind) -> String {
        match kind {
            ConcreteKind::Array(ty, n) => format!("[{}; {}]", self.fmt_ty_id(*ty), n),
            ConcreteKind::Ptr(id) => format!("*{}", self.fmt_ty_id(*id)),
            ConcreteKind::Path(spur) => self.string_interner.resolve(spur).to_string(),
            ConcreteKind::Tuple(ids) => {
                format!("({})", ids.iter().map(|id| self.fmt_ty_id(*id)).join(", "))
            }
        }
    }

    fn fmt_tentry(&self, entry: &TEntry) -> String {
        format!(
            "{}{}",
            self.fmt_typekind(&entry.get_constr()),
            if let Some(params) = entry.get_params() {
                if params.is_empty() {
                    "".to_string()
                } else {
                    params
                        .iter()
                        .map(|param| self.fmt_typekind(param))
                        .join(", ")
                }
            } else {
                "".to_string()
            }
        )
    }

    pub fn fmt_trait_restriction(&self, trait_restriction: &TraitRestriction) -> String {
        format!(
            "{}{}",
            self.string_interner.resolve(&trait_restriction.trait_name),
            if trait_restriction.args.is_empty() {
                "".to_string()
            } else {
                format!(
                    "<{}>",
                    trait_restriction
                        .args
                        .iter()
                        .map(|id| self.fmt_ty_id(*id))
                        .join(", ")
                )
            }
        )
    }
}

impl Display for TEnv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n  {}\n",
            "TEnv".green(),
            self.entries
                .iter()
                .enumerate()
                .format_with("\n  ", |(idx, entry), f| {
                    f(&format_args!(
                        "{}{} {} Key({}) {} {} ",
                        "'".blue(),
                        idx.blue(),
                        "->".purple(),
                        entry.get_constr(),
                        "->".purple(),
                        self.fmt_tentry(entry),
                    ))
                }),
        )
    }
}

impl Display for TEntry {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.constr())
    }
}

impl Display for ConcreteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(ty, n) => write!(f, "['{ty}; {n}]"),
            Self::Path(path) => write!(f, "{path:?}"),
            Self::Ptr(ptr) => write!(f, "*'{ptr}"),
            Self::Tuple(_) => write!(f, "()"),
        }
    }
}

#[cfg(test)]
mod tests {
    // use crate::{ConcreteKind, TEnv, Type, TypeKind};

    // static STRING_INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

    // fn file_spanned(ty: Type) -> FileSpanned<Type> {
    //     FileSpanned::new(
    //         Spanned::new(ty, Span::new(TextRange::new(0.into(), 0.into()))),
    //         FileId(Spur::default()),
    //     )
    // }

    // #[test]
    // fn fmt_ty_ids() {
    //     let mut tenv = TEnv::new(&STRING_INTERNER);

    //     let id = tenv.insert(file_spanned(Type::new(TypeKind::Unknown)));
    //     let fmt = tenv.fmt_ty_id(id);
    //     assert_eq!(fmt, "unknown");
    //     let id = tenv.insert(file_spanned(Type::new(TypeKind::Int(None))));
    //     let fmt = tenv.fmt_ty_id(id);
    //     assert_eq!(fmt, "int");
    //     let id = tenv.insert(file_spanned(Type::new(TypeKind::Float(None))));
    //     let fmt = tenv.fmt_ty_id(id);
    //     assert_eq!(fmt, "float");
    //     let other_id = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
    //         ConcreteKind::Tuple(vec![]),
    //     ))));
    //     let id = tenv.insert(file_spanned(Type::new(TypeKind::Ref(other_id))));
    //     let fmt = tenv.fmt_ty_id(id);
    //     assert_eq!(fmt, "()");
    //     let a = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
    //         ConcreteKind::Path(STRING_INTERNER.get_or_intern("test")),
    //     ))));
    //     let b = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
    //         ConcreteKind::Path(STRING_INTERNER.get_or_intern("test::foo")),
    //     ))));
    //     let c = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
    //         ConcreteKind::Ptr(a),
    //     ))));
    //     let id = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
    //         ConcreteKind::Tuple(vec![a, b, c]),
    //     ))));
    //     let fmt = tenv.fmt_ty_id(id);
    //     assert_eq!(fmt, "(test, test::foo, *test)");
    // }
}
