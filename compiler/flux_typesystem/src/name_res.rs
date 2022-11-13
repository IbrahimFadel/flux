use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::FileSpanned;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};

use crate::{diagnostics::TypeError, TypeId};

#[derive(Debug)]
pub(crate) struct NameResolver {
    /// Path and alias
    uses: Vec<(Spur, Option<Spur>)>,
    variables: Vec<Scope>,
    types: HashMap<Spur, TypeId>,
}

impl NameResolver {
    pub fn new() -> Self {
        Self {
            uses: vec![],
            variables: vec![Scope::new()],
            types: HashMap::new(),
        }
    }

    pub fn insert_use(&mut self, path: Spur, alias: Option<Spur>) {
        self.uses.push((path, alias));
    }

    pub fn insert_local_to_current_scope(&mut self, name: Spur, id: TypeId) {
        self.variables
            .last_mut()
            .expect("internal compiler error: no current scope in name resolver")
            .insert_local(name, id);
    }

    pub fn insert_type(&mut self, path: Spur, id: TypeId) {
        self.types.insert(path, id);
    }

    pub fn resolve_variable(
        &self,
        path: FileSpanned<Spur>,
        string_interner: &'static ThreadedRodeo,
    ) -> Result<TypeId, Diagnostic> {
        for scope in self.variables.iter().rev() {
            if let Some(id) = scope.get_local_typeid(path.inner.inner) {
                return Ok(id);
            }
        }

        for (use_path, use_alias) in &self.uses {
            if let Some(alias) = use_alias {
                if path.inner.inner == *alias {
                    todo!()
                }
            }
        }

        Err(TypeError::UnknownVariable {
            name: path.map_inner_ref(|spur| string_interner.resolve(spur).to_string()),
        }
        .to_diagnostic())
    }

    pub fn resolve_type(
        &self,
        path: FileSpanned<Spur>,
        string_interner: &'static ThreadedRodeo,
    ) -> Result<TypeId, Diagnostic> {
        self.types.get(&path).cloned().map_or(
            Err(TypeError::UnknownType {
                path: path.map_inner_ref(|spur| string_interner.resolve(spur).to_string()),
            }
            .to_diagnostic()),
            Ok,
        )
    }
}

#[derive(Debug)]
struct Scope(HashMap<Spur, TypeId>);

impl Scope {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert_local(&mut self, name: Spur, id: TypeId) {
        self.0.insert(name, id);
    }

    pub fn get_local_typeid(&self, name: Spur) -> Option<TypeId> {
        self.0.get(&name).cloned()
    }
}
