use flux_diagnostics::{Diagnostic, ToDiagnostic};
use flux_span::{FileId, FileSpanned, InFile, Span, WithSpan};
use itertools::Itertools;
use lasso::{Spur, ThreadedRodeo};

use crate::{
    diagnostics::TypeError,
    env::TraitRestriction,
    trait_solver::{TraitImplementation, TraitImplementationTable},
    ConcreteKind, TEnv, Type, TypeId, TypeKind,
};

mod traits;
mod unify;

#[derive(Debug)]
pub struct TChecker {
    pub tenv: TEnv,
    trait_implementation_table: TraitImplementationTable,
    string_interner: &'static ThreadedRodeo,
}

impl TChecker {
    pub fn new(string_interner: &'static ThreadedRodeo) -> Self {
        Self {
            tenv: TEnv::new(string_interner),
            trait_implementation_table: TraitImplementationTable::new(),
            string_interner,
        }
    }
}
