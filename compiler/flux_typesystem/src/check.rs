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
pub struct TChecker<'tenv> {
    pub tenv: &'tenv mut TEnv,
    string_interner: &'static ThreadedRodeo,
    pub trait_applications: TraitApplicationTable,
}

impl<'tenv> TChecker<'tenv> {
    pub fn new(tenv: &'tenv mut TEnv, string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            tenv,
            string_interner,
            trait_applications: TraitApplicationTable::new(),
        }
    }
}
