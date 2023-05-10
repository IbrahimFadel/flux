use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{InFile, Span};
use lasso::ThreadedRodeo;

use crate::{
    diagnostics::TypeError, trait_solver::TraitApplicationTable, ConcreteKind, TEnv, Type, TypeId,
    TypeKind,
};

mod traits;
mod unify;

#[derive(Debug)]
pub struct TChecker {
    pub tenv: TEnv,
    string_interner: &'static ThreadedRodeo,
    pub trait_applications: TraitApplicationTable,
}

impl TChecker {
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            tenv: TEnv::new(string_interner),
            string_interner,
            trait_applications: TraitApplicationTable::new(),
        }
    }
}
