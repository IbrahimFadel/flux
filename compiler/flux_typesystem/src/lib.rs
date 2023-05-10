mod check;
pub mod diagnostics;
mod env;
mod scope;
#[cfg(test)]
mod tests;
mod trait_solver;
pub mod r#type;

pub use check::TChecker;
pub use env::{TEnv, TraitRestriction};
pub use r#type::{ConcreteKind, Type, TypeId, TypeKind};
pub use trait_solver::TraitApplication;
