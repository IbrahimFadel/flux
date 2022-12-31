use std::fmt::Display;

use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, Spanned};
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use lasso::{Spur, ThreadedRodeo};
use owo_colors::OwoColorize;
use tracing::trace;

use crate::{
    diagnostics::TypeError,
    intern::{Interner, Key},
    name_res::NameResolver,
    r#type::{ConcreteKind, StructConcreteKind, Type, TypeId, TypeKind},
};

type FunctionSignature = (FileSpanned<Vec<TypeId>>, FileSpanned<TypeId>);
type FunctionSignatureMap = HashMap<Vec<Spur>, FunctionSignature>;

/*

module tree: function exports, type exports, submodule exports?
variable namespace
type namespace
use lookup table



*/

#[derive(Debug)]
pub struct TEnv {
    type_interner: Interner,
    pub string_interner: &'static ThreadedRodeo,
    entries: Vec<FileSpanned<TEntry>>,
    pub name_resolver: NameResolver,
    /// Scopes of the current [`TEnv`]
    ///
    /// Holds types of locals. Furthermore, the first element in the vector represents the global scope (outside of the current function / type environment).
    function_signatures: FunctionSignatureMap,
    pub(super) int_paths: HashSet<Spur>,
    pub(super) float_paths: HashSet<Spur>,
}

/// A `flux_typesystem` type entry
///
/// Stores the [`Key`] of the type constructor and a list of constraints
#[derive(Debug)]
pub struct TEntry {
    pub key: Key,
    pub constraints: Vec<TypeId>,
}

impl TEntry {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            constraints: vec![],
        }
    }

    pub fn with_constraints(key: Key, constraints: Vec<TypeId>) -> Self {
        Self { key, constraints }
    }

    pub fn set_key(&mut self, key: Key) {
        self.key = key;
    }
}

impl TEnv {
    /// Construct a new `flux_typesystem` [`TEnv`]
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            type_interner: Interner::new(string_interner),
            string_interner,
            entries: vec![],
            name_resolver: NameResolver::new(),
            // Global scope, and function scope
            function_signatures: HashMap::new(),
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
        entry.map_inner_ref(|entry| self.type_interner.resolve(entry.key).clone())
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
    pub fn insert_unit(&mut self, span: InFile<Span>) -> TypeId {
        self.insert(FileSpanned::new(
            Spanned::new(
                Type::new(TypeKind::Concrete(ConcreteKind::Tuple(vec![]))),
                span.inner,
            ),
            span.file_id,
        ))
    }

    /// Insert an unknown type into the [`TEnv`]
    pub fn insert_unknown(&mut self, span: InFile<Span>) -> TypeId {
        self.insert(FileSpanned::new(
            Spanned::new(Type::new(TypeKind::Unknown), span.inner),
            span.file_id,
        ))
    }

    /// Insert a `Spanned<Type>` into the type environment
    ///
    /// This interns the [`TypeKind`] and pushes a new [`TEntry`] to th environment, returning
    /// a valid [`TypeId`]
    pub fn insert(&mut self, ty: FileSpanned<Type>) -> TypeId {
        let key = self.type_interner.intern(ty.constr());
        let idx = self.entries.len();
        self.entries.push(ty.map_inner_ref(|_| TEntry::new(key)));
        let id = TypeId::new(idx);
        trace!("inserting type inference var {}", id);
        id
    }

    /// Insert a `Spanned<Type>` with trait constraints into the type environment
    ///
    /// This interns the [`TypeKind`] and pushes a new [`TEntry`] to th environment, returning
    /// a valid [`TypeId`]
    pub fn insert_with_constraints(
        &mut self,
        ty: FileSpanned<Type>,
        constraints: Vec<TypeId>,
    ) -> TypeId {
        let key = self.type_interner.intern(ty.constr());
        let idx = self.entries.len();
        self.entries
            .push(ty.map_ref(|ty| ty.map_ref(|_| TEntry::with_constraints(key, constraints))));
        TypeId::new(idx)
    }

    pub fn set_type(&mut self, id: TypeId, kind: TypeKind) {
        let key = self.type_interner.intern(kind);
        self.get_entry_mut(id).inner.inner.set_key(key);
    }

    /// Insert a local to the current [`Scope`]
    pub fn insert_local_to_scope(&mut self, name: Spur, id: TypeId) {
        self.name_resolver.insert_local_to_current_scope(name, id);
    }

    pub fn insert_function_signature(&mut self, path: Vec<Spur>, signature: FunctionSignature) {
        self.function_signatures.insert(path, signature);
    }

    fn hir_path_to_spur(&self, path: impl Iterator<Item = Spur>) -> Spur {
        let path_string = path
            .map(|spur| self.string_interner.resolve(&spur))
            .join("::");
        self.string_interner.get_or_intern(path_string)
    }

    pub fn insert_struct_type(&mut self, path: impl Iterator<Item = Spur>, struct_ty: TypeId) {
        let spur = self.hir_path_to_spur(path);
        self.name_resolver.insert_type(spur, struct_ty);
    }

    pub fn get_function_signature(
        &mut self,
        path: &FileSpanned<Vec<Spur>>,
    ) -> Result<FunctionSignature, Diagnostic> {
        self.function_signatures
            .get(&path.inner.inner)
            .map_or_else(
                || {
                    Err(TypeError::UnknownFunction {
                        path: path.map_inner_ref(|path| {
                            path.iter()
                                .map(|spur| self.string_interner.resolve(spur))
                                .join("::")
                        }),
                    }
                    .to_diagnostic())
                },
                Ok,
            )
            .cloned()
    }

    pub fn get_struct_field_types(
        &self,
        path: FileSpanned<impl Iterator<Item = Spur>>,
    ) -> Result<(StructConcreteKind, InFile<Span>), Diagnostic> {
        let (file_id, span) = (path.file_id, path.span);
        let spur = path.map_inner(|path| self.hir_path_to_spur(path));
        let struct_type_id = self
            .name_resolver
            .resolve_type(spur.clone(), self.string_interner)?;
        let typekind = self.get_typekind_with_id(struct_type_id);
        let filespan = InFile::new(typekind.span, typekind.file_id);
        match typekind.inner.inner {
            TypeKind::Concrete(ConcreteKind::Struct(fields)) => Ok((fields, filespan)),
            _ => Err(TypeError::UnknownStruct {
                path: FileSpanned::new(
                    Spanned::new(self.string_interner.resolve(&spur).to_string(), span),
                    file_id,
                ),
            }
            .to_diagnostic()),
        }
    }

    /// Get the [`TypeId`] of a path in any currently accessible [`Scope`]
    pub fn get_path_typeid(
        &mut self,
        path: FileSpanned<impl Iterator<Item = Spur>>,
    ) -> Result<TypeId, Diagnostic> {
        let spur = path.map_inner(|path| self.hir_path_to_spur(path));
        self.name_resolver
            .resolve_variable(spur, self.string_interner)
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

    pub fn fmt_typekind(&self, kind: &TypeKind) -> String {
        match kind {
            TypeKind::Generic => "todo: how to format generics?".to_string(),
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
            self.fmt_typekind(self.type_interner.resolve(entry.key)),
            if entry.constraints.is_empty() {
                String::new()
            } else {
                todo!()
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
                        entry.key,
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
            self.key,
            if !self.constraints.is_empty() {
                ": ".to_string()
            } else {
                String::new()
            },
            self.constraints
                .iter()
                .format_with(", ", |constraint, f| f(&format_args!("{}", constraint)))
        )
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.constr() {
            TypeKind::Concrete(cncrt) => write!(f, "{}", cncrt),
            TypeKind::Float(_) => write!(f, "float"),
            TypeKind::Generic => write!(f, "generic"),
            TypeKind::Int(_) => write!(f, "int"),
            TypeKind::Ref(id) => write!(f, "ref('{id})"),
            TypeKind::Unknown => write!(f, "unknown"),
        }
    }
}

impl Display for ConcreteKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Path(path) => write!(f, "{:?}", path),
            Self::Ptr(ptr) => write!(f, "*'{}", ptr),
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

    #[test]
    fn fmt_ty_ids() {
        let mut tenv = TEnv::new(&STRING_INTERNER);

        let id = tenv.insert(file_spanned(Type::new(TypeKind::Unknown)));
        let fmt = tenv.fmt_ty_id(id);
        assert_eq!(fmt, "unknown");
        let id = tenv.insert(file_spanned(Type::new(TypeKind::Int(None))));
        let fmt = tenv.fmt_ty_id(id);
        assert_eq!(fmt, "int");
        let id = tenv.insert(file_spanned(Type::new(TypeKind::Float(None))));
        let fmt = tenv.fmt_ty_id(id);
        assert_eq!(fmt, "float");
        let other_id = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
            ConcreteKind::Tuple(vec![]),
        ))));
        let id = tenv.insert(file_spanned(Type::new(TypeKind::Ref(other_id))));
        let fmt = tenv.fmt_ty_id(id);
        assert_eq!(fmt, "()");
        let a = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
            ConcreteKind::Path(STRING_INTERNER.get_or_intern("test")),
        ))));
        let b = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
            ConcreteKind::Path(STRING_INTERNER.get_or_intern("test::foo")),
        ))));
        let c = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
            ConcreteKind::Ptr(a),
        ))));
        let id = tenv.insert(file_spanned(Type::new(TypeKind::Concrete(
            ConcreteKind::Tuple(vec![a, b, c]),
        ))));
        let fmt = tenv.fmt_ty_id(id);
        assert_eq!(fmt, "(test, test::foo, *test)");
    }
}
