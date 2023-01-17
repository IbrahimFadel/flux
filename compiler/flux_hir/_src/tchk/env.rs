use std::collections::VecDeque;

use crate::tchk::{
    diagnostics::TypeError,
    r#type::{ConcreteKind, TypeKind},
};

use super::{
    constraints::Constraint,
    name_resolver::NameResolver,
    r#type::{Type, TypeId},
};
use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileSpanned, InFile, Span, WithSpan};
use hashbrown::{HashMap, HashSet};
use itertools::Itertools;
use lasso::{Spur, ThreadedRodeo};
use tracing::{debug, info, trace};

struct Scope {
    vars: HashMap<Spur, TypeId>,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }
}

pub(super) struct TEnv {
    pub name_resolver: NameResolver,
    constraints: VecDeque<Constraint>,
    types: Vec<FileSpanned<Type>>,
    scopes: Vec<Scope>,
    pub diagnostics: Vec<Diagnostic>,
    string_interner: &'static ThreadedRodeo,
    int_paths: HashSet<Spur>,
}

impl TEnv {
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            name_resolver: NameResolver::new(string_interner),
            constraints: VecDeque::new(),
            types: vec![],
            scopes: vec![Scope::new()],
            diagnostics: vec![],
            string_interner,
            int_paths: HashSet::from([
                string_interner.get_or_intern_static("i8"),
                string_interner.get_or_intern_static("i16"),
                string_interner.get_or_intern_static("i32"),
                string_interner.get_or_intern_static("i64"),
                string_interner.get_or_intern_static("u8"),
                string_interner.get_or_intern_static("u16"),
                string_interner.get_or_intern_static("u32"),
                string_interner.get_or_intern_static("u64"),
            ]),
        }
    }

    pub fn reset(&mut self) {
        self.constraints.clear();
        self.types.clear();
    }

    pub fn insert(&mut self, ty: FileSpanned<Type>) -> TypeId {
        let len = self.types.len();
        self.types.push(ty);
        let id = TypeId::new(len);
        trace!("inserting type id {} to env", id);
        id
    }

    pub fn push_constraint(&mut self, constraint: Constraint) {
        trace!("pushing new constraint to env: {}", constraint);
        self.constraints.push_back(constraint);
    }

    pub fn insert_var(&mut self, name: Spur, id: TypeId) {
        let scope = self
            .scopes
            .last_mut()
            .expect("internal compiler error: no scope in type environment");
        scope.vars.insert(name, id);
    }

    pub fn set_type(&mut self, id: TypeId, kind: FileSpanned<Type>) {
        self.types[id.get()] = kind;
    }

    pub fn get_path_typeid(&self, path: &Spur) -> TypeId {
        let scope = self
            .scopes
            .last()
            .expect("internal compiler error: no scope in type environment");
        scope.vars.get(path).cloned().unwrap()
    }

    pub fn get_type_with_id(&self, id: TypeId) -> &FileSpanned<Type> {
        &self.types[id.get()]
    }

    pub fn solve_constraints(&mut self) {
        info!("solving constraints");
        let mut tries = self.constraints.len();

        while tries > 0 {
            if let Some(c) = self.constraints.pop_front() {
                tries -= 1;
                match self.solve_constraint(c.clone()) {
                    // Constraint resolved
                    Some(res) => {
                        // Record any errors while resolving the constraint
                        if let Err(e) = res {
                            self.diagnostics.push(e.to_diagnostic());
                        }
                        // A constraint being resolved resets the counter
                        tries = self.constraints.len();
                    }
                    None => self.constraints.push_back(c), // Still unresolved...
                }
            } else {
                break;
            }
        }
    }

    fn solve_constraint(&mut self, constraint: Constraint) -> Option<Result<(), TypeError>> {
        match constraint {
            Constraint::TypeEq(a, b, span) => {
                let res = Some(self.unify(a, b, span));
                debug!("solved type equality constraint between {} and {}", a, b);
                res
            }
            _ => None,
        }
    }

    fn unify(
        &mut self,
        a: TypeId,
        b: TypeId,
        unification_span: InFile<Span>,
    ) -> Result<(), TypeError> {
        let aa = self.get_type_with_id(a);
        let bb = self.get_type_with_id(b);
        match (&aa.constr, &bb.constr) {
            (TypeKind::Unknown, _) => {
                self.set_type(a, bb.inner.inner.clone().in_file(aa.file_id, aa.span));
                Ok(())
            }
            (TypeKind::Int(a_int), TypeKind::Int(b_int)) => match (a_int, b_int) {
                (Some(a_int), Some(b_int)) => self.unify(*a_int, *b_int, unification_span),
                (Some(a_int), None) => {
                    self.set_type(b, self.get_type_with_id(*a_int).clone());
                    Ok(())
                }
                (None, Some(b_int)) => {
                    self.set_type(a, self.get_type_with_id(*b_int).clone());
                    Ok(())
                }
                (None, None) => Ok(()),
            },
            (TypeKind::Concrete(ConcreteKind::Path(path)), TypeKind::Int(int_id)) => match int_id {
                Some(int_id) => self.unify(a, *int_id, unification_span),
                None => {
                    if self.int_paths.get(path).is_some() {
                        self.set_type(
                            b,
                            Type::new(TypeKind::Int(Some(a))).in_file(bb.file_id, bb.span),
                        );
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span))
                    }
                }
            },
            (TypeKind::Int(int_id), TypeKind::Concrete(ConcreteKind::Path(path))) => match int_id {
                Some(int_id) => self.unify(*int_id, a, unification_span),
                None => {
                    if self.int_paths.get(path).is_some() {
                        self.set_type(
                            a,
                            Type::new(TypeKind::Int(Some(b))).in_file(aa.file_id, aa.span),
                        );
                        Ok(())
                    } else {
                        Err(self.type_mismatch(a, b, unification_span))
                    }
                }
            },
            (TypeKind::Concrete(concrete_a), TypeKind::Concrete(concrete_b)) => {
                match (concrete_a, concrete_b) {
                    (ConcreteKind::Path(path_a), ConcreteKind::Path(path_b))
                        if path_a == path_b =>
                    {
                        Ok(())
                    }
                    (_, _) => Err(TypeError::TypeMismatch {
                        a: self.fmt_type_with_id(a),
                        b: self.fmt_type_with_id(b),
                        span: unification_span,
                    }),
                }
            }
            (_, _) => Err(self.type_mismatch(a, b, unification_span)),
        }
    }

    fn type_mismatch(&self, a: TypeId, b: TypeId, unification_span: InFile<Span>) -> TypeError {
        TypeError::TypeMismatch {
            a: self.fmt_type_with_id(a),
            b: self.fmt_type_with_id(b),
            span: unification_span,
        }
    }

    pub(super) fn fmt_type_with_id(&self, id: TypeId) -> FileSpanned<String> {
        let ty = self.get_type_with_id(id);
        ty.map_inner_ref(|ty| match &ty.constr {
            TypeKind::Concrete(concrete) => self.fmt_concrete_kind(concrete),
            TypeKind::Float(_) => format!("float"),
            TypeKind::Generic(name) => format!("{}", self.string_interner.resolve(name)),
            TypeKind::Int(_) => format!("int"),
            TypeKind::Ref(id) => self.fmt_type_with_id(*id).inner.inner,
            TypeKind::Unknown => format!("unknown"),
        })
    }

    fn fmt_concrete_kind(&self, concrete: &ConcreteKind) -> String {
        match concrete {
            ConcreteKind::Array(ty, n) => {
                format!("[{}; {}]", self.fmt_type_with_id(*ty).inner.inner, n)
            }
            ConcreteKind::Path(path) => self.string_interner.resolve(path).to_string(),
            ConcreteKind::Ptr(ty) => format!("{}*", self.fmt_type_with_id(*ty).inner.inner),
            ConcreteKind::Struct(strukt) => {
                format!(
                    "{{ {} }}",
                    strukt
                        .fields
                        .iter()
                        .map(|(name, ty)| format!(
                            "{} {}",
                            self.string_interner.resolve(name),
                            self.fmt_type_with_id(*ty).inner.inner
                        ))
                        .join(", ")
                )
            }
            ConcreteKind::Tuple(types) => format!(
                "({})",
                types
                    .iter()
                    .map(|id| self.fmt_type_with_id(*id).inner.inner)
                    .join(", ")
            ),
        }
    }
}
