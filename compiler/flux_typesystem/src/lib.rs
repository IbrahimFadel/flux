use flux_diagnostics::Diagnostic;
use flux_id::{id, Map};
use flux_span::{InFile, Span};
use tenv::TEnv;

pub use r#type::{ConcreteKind, Generic, Path, Type, Typed, WithType};

mod tenv;
mod r#trait;
mod r#type;

pub struct TypeSystem<'env> {
    tenv: &'env mut TEnv,
    traits: Map<id::Tr, ()>,
    applications: Map<id::App, ()>,
}

pub fn unify(
    a: id::Ty,
    b: id::Ty,
    unification_span: InFile<Span>,
    ts: &mut TypeSystem,
) -> Result<(), Diagnostic> {
    Ok(())
}
