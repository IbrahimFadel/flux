use std::fmt::Display;

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};
use itertools::Itertools;
use lasso::{Spur, ThreadedRodeo};
use owo_colors::OwoColorize;
use std::collections::HashSet;

use crate::{
    diagnostics::TypeError,
    intern::{Interner, Key},
    r#type::{ConcreteKind, Type, TypeId, TypeKind},
    scope::Scope,
};

#[derive(Debug)]
pub struct TEnv {
    pub type_interner: Interner,
    pub string_interner: &'static ThreadedRodeo,
    entries: Vec<FileSpanned<TEntry>>,
    pub locals: Vec<Scope>,
    pub(super) int_paths: HashSet<Spur>,
    pub(super) float_paths: HashSet<Spur>,
}

#[derive(Debug, Clone)]
pub struct TraitRestriction {
    pub name: FileSpanned<Spur>,
    pub args: Vec<TypeId>,
}

impl TraitRestriction {
    pub fn new(name: FileSpanned<Spur>, args: Vec<TypeId>) -> Self {
        Self { name, args }
    }
}

/// A `flux_typesystem` type entry
///
/// Stores the [`Key`] of the type constructor and a list of constraints
#[derive(Debug, Clone)]
pub struct TEntry {
    keys: Vec<Key>,
    pub restrictions: Vec<TraitRestriction>,
}

impl TEntry {
    pub fn new(constr: Key) -> Self {
        Self {
            keys: vec![constr],
            restrictions: vec![],
        }
    }

    pub fn with_restrictions(constr: Key, restrictions: Vec<TraitRestriction>) -> Self {
        Self {
            keys: vec![constr],
            restrictions,
        }
    }

    pub fn with_args(constr: Key, args: impl Iterator<Item = Key>) -> Self {
        let keys = std::iter::once(constr).chain(args).collect();
        Self {
            keys,
            restrictions: vec![],
        }
    }

    pub fn with_args_and_restrictions(
        constr: Key,
        args: impl Iterator<Item = Key>,
        restrictions: Vec<TraitRestriction>,
    ) -> Self {
        let keys = std::iter::once(constr).chain(args).collect();
        Self { keys, restrictions }
    }

    pub fn get_constr(&self) -> Key {
        self.keys[0]
    }

    pub fn get_params(&self) -> Option<&[Key]> {
        self.keys.get(1..)
    }

    pub fn set_constr(&mut self, key: Key) {
        self.keys[0] = key;
    }

    pub fn set_params(&mut self, params: impl Iterator<Item = Key>) {
        self.keys.splice(1.., params.collect::<Vec<_>>());
    }
}

impl TEnv {
    /// Construct a new `flux_typesystem` [`TEnv`]
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            type_interner: Interner::new(string_interner),
            string_interner,
            entries: vec![],
            // name_resolver: NameResolver::new(),
            // Global scope
            locals: vec![Scope::new()],
            // function_signatures: HashMap::new(),
            int_paths: HashSet::from([
                string_interner.get_or_intern_static("u8"),
                string_interner.get_or_intern_static("u16"),
                string_interner.get_or_intern_static("u32"),
                string_interner.get_or_intern_static("u64"),
                string_interner.get_or_intern_static("i8"),
                string_interner.get_or_intern_static("i16"),
                string_interner.get_or_intern_static("i32"),
                string_interner.get_or_intern_static("i64"),
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
        entry.map_inner_ref(|entry| self.type_interner.resolve(entry.get_constr()).clone())
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
        let ty = Type::new(
            TypeKind::Concrete(ConcreteKind::Tuple(vec![])),
            &mut self.type_interner,
        );
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert an unknown type into the [`TEnv`]
    #[inline]
    pub fn insert_unknown(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Unknown, &mut self.type_interner);
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert an int type into the [`TEnv`]
    #[inline]
    pub fn insert_int(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Int(None), &mut self.type_interner);
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert a float type into the [`TEnv`]
    #[inline]
    pub fn insert_float(&mut self, span: InFile<Span>) -> TypeId {
        let ty = Type::new(TypeKind::Float(None), &mut self.type_interner);
        self.insert(FileSpanned::new(Spanned::new(ty, span.inner), span.file_id))
    }

    /// Insert a `Spanned<Type>` into the type environment
    ///
    /// This interns the [`TypeKind`] and pushes a new [`TEntry`] to th environment, returning
    /// a valid [`TypeId`]
    pub fn insert(&mut self, ty: FileSpanned<Type>) -> TypeId {
        let key = ty.constr();
        let idx = self.entries.len();
        self.entries.push(ty.map_inner_ref(|_| TEntry::new(key)));
        TypeId::new(idx)
    }

    /// Insert a `Spanned<Type>` with trait constraints into the type environment
    ///
    /// This interns the [`TypeKind`] and pushes a new [`TEntry`] to th environment, returning
    /// a valid [`TypeId`]
    pub fn insert_with_constraints(
        &mut self,
        ty: FileSpanned<Type>,
        restrictions: Vec<TraitRestriction>,
    ) -> TypeId {
        let key = ty.constr();
        let idx = self.entries.len();
        self.entries
            .push(ty.map_ref(|ty| ty.map_ref(|_| TEntry::with_restrictions(key, restrictions))));
        TypeId::new(idx)
    }

    pub fn set_type(&mut self, id: TypeId, ty: Type) {
        let key = ty.constr();
        self.get_entry_mut(id).inner.inner.set_constr(key);
        if let Some(keys) = ty.params() {
            self.get_entry_mut(id)
                .inner
                .inner
                .set_params(keys.iter().cloned());
        };
    }

    /// Insert a local to the current [`Scope`]
    pub fn insert_local_to_scope(&mut self, name: Spur, id: TypeId) {
        self.locals
            .last_mut()
            .expect("internal compiler error: no active scope in type environment")
            .insert_local(name, id);
    }

    // pub fn insert_function_signature(&mut self, path: Vec<Spur>, signature: FunctionSignature) {
    //     self.function_signatures.insert(path, signature);
    // }

    // fn hir_path_to_spur(&self, path: impl Iterator<Item = Spur>) -> Spur {
    //     let path_string = path
    //         .map(|spur| self.string_interner.resolve(&spur))
    //         .join("::");
    //     self.string_interner.get_or_intern(path_string)
    // }

    // pub fn get_function_signature(
    //     &mut self,
    //     path: &FileSpanned<Vec<Spur>>,
    // ) -> Result<FunctionSignature, Diagnostic> {
    //     self.function_signatures
    //         .get(&path.inner.inner)
    //         .map_or_else(
    //             || {
    //                 Err(TypeError::UnknownFunction {
    //                     path: path.map_inner_ref(|path| {
    //                         path.iter()
    //                             .map(|spur| self.string_interner.resolve(spur))
    //                             .join("::")
    //                     }),
    //                 }
    //                 .to_diagnostic())
    //             },
    //             Ok,
    //         )
    //         .cloned()
    // }

    // pub fn get_struct_field_types(
    //     &self,
    //     path: FileSpanned<impl Iterator<Item = Spur>>,
    // ) -> Result<(StructConcreteKind, InFile<Span>), Diagnostic> {
    //     let (file_id, span) = (path.file_id, path.span);
    //     let spur = path.map_inner(|path| self.hir_path_to_spur(path));
    //     let struct_type_id = self
    //         .name_resolver
    //         .resolve_type(spur.clone(), self.string_interner)?;
    //     let typekind = self.get_typekind_with_id(struct_type_id);
    //     let filespan = InFile::new(typekind.span, typekind.file_id);
    //     match typekind.inner.inner {
    //         TypeKind::Concrete(ConcreteKind::Struct(fields)) => Ok((fields, filespan)),
    //         _ => Err(TypeError::UnknownStruct {
    //             path: FileSpanned::new(
    //                 Spanned::new(self.string_interner.resolve(&spur).to_string(), span),
    //                 file_id,
    //             ),
    //         }
    //         .to_diagnostic()),
    //     }
    // }

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
                        name: name
                            .map_inner_ref(|spur| self.string_interner.resolve(spur).to_string()),
                    }
                    .to_diagnostic())
                },
                Ok,
            )
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
            TypeKind::Generic(name) => self.string_interner.resolve(name).to_string(),
            TypeKind::Unknown => "unknown".to_string(),
            TypeKind::Int(_) => "int".to_string(),
            TypeKind::Float(_) => "float".to_string(),
            TypeKind::Ref(id) => self.fmt_ty_id_constr(*id),
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
            ConcreteKind::Struct(strukt) => format!(
                "{{{}}}",
                strukt
                    .fields
                    .iter()
                    .map(|(name, ty)| format!(
                        "{}: {}",
                        self.string_interner.resolve(name),
                        self.fmt_ty_id_constr(*ty)
                    ))
                    .join(",\n"),
            ),
        }
    }

    pub fn fmt_typekind(&self, kind: &TypeKind) -> String {
        match kind {
            TypeKind::Generic(name) => self.string_interner.resolve(name).to_string(),
            TypeKind::Unknown => "unknown".to_string(),
            TypeKind::Int(_) => "int".to_string(),
            TypeKind::Float(_) => "float".to_string(),
            TypeKind::Ref(id) => self.fmt_ty_id(*id),
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
            ConcreteKind::Struct(strukt) => format!(
                "{{{}}}",
                strukt
                    .fields
                    .iter()
                    .map(|(name, ty)| format!(
                        "{}: {}",
                        self.string_interner.resolve(name),
                        self.fmt_ty_id(*ty)
                    ))
                    .join(",\n"),
            ),
        }
    }

    fn fmt_tentry(&self, entry: &TEntry) -> String {
        format!(
            "{}{}",
            self.fmt_typekind(self.type_interner.resolve(entry.get_constr())),
            if entry.restrictions.is_empty() {
                String::new()
            } else {
                todo!()
            }
        )
    }

    pub(super) fn fmt_trait_restriction(&self, trait_restriction: &TraitRestriction) -> String {
        format!(
            "{}{}",
            self.string_interner
                .resolve(&trait_restriction.name.inner.inner),
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
            "{}\n{}\n  {}\n",
            self.type_interner,
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
            // self.scopes
            //     .iter()
            //     .enumerate()
            //     .map(|(idx, scope)| {
            //         format!(
            //             "scope {idx} {{\n  {}\n}}\n",
            //             scope.0.iter().format_with("\n  ", |(name, ty), f| {
            //                 f(&format_args!(
            //                     "{} {} {}",
            //                     self.string_interner.resolve(name).yellow(),
            //                     "->".purple(),
            //                     ty
            //                 ))
            //             })
            //         )
            //     })
            //     .join("\n")
        )
    }
}

impl Display for TEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}{}{}",
            self.get_constr(),
            if !self.restrictions.is_empty() {
                ": ".to_string()
            } else {
                String::new()
            },
            self.restrictions
                .iter()
                .format_with(", ", |_, f| f(&format_args!("{:?}", self.restrictions)))
        )
    }
}

// impl Display for TraitRestriction {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?}", self.name)
//     }
// }

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.constr())
        // match self.constr() {
        //     TypeKind::Concrete(cncrt) => write!(f, "{}", cncrt),
        //     TypeKind::Float(_) => write!(f, "float"),
        //     TypeKind::Generic => write!(f, "generic"),
        //     TypeKind::Int(_) => write!(f, "int"),
        //     TypeKind::Ref(id) => write!(f, "ref('{id})"),
        //     TypeKind::Unknown => write!(f, "unknown"),
        // }
    }
}

impl Display for ConcreteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Array(ty, n) => write!(f, "['{ty}; {n}]"),
            Self::Path(path) => write!(f, "{path:?}"),
            Self::Ptr(ptr) => write!(f, "*'{ptr}"),
            Self::Tuple(_) => write!(f, "()"),
            Self::Struct(_) => write!(f, "todo (lazy pos)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use flux_span::{FileId, FileSpanned, Span, Spanned};
    use lasso::{Spur, ThreadedRodeo};
    use once_cell::sync::Lazy;
    use text_size::TextRange;

    use crate::{ConcreteKind, TEnv, Type, TypeKind};

    static STRING_INTERNER: Lazy<ThreadedRodeo> = Lazy::new(ThreadedRodeo::new);

    fn file_spanned(ty: Type) -> FileSpanned<Type> {
        FileSpanned::new(
            Spanned::new(ty, Span::new(TextRange::new(0.into(), 0.into()))),
            FileId(Spur::default()),
        )
    }

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
