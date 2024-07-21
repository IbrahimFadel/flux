mod check;
mod diagnostics;
mod env;
mod fmt;
mod prettyprint;
mod scope;
mod r#trait;
mod r#type;

pub use check::TChecker;
pub use env::{Insert, TEnv};
pub use r#trait::{ApplicationId, ThisCtx, Trait, TraitApplication, TraitId, TraitRestriction};
pub use r#type::{
    ConcreteKind, FnSignature, Generic, Path, ThisPath, TypeId, TypeKind, Typed, WithType,
};
