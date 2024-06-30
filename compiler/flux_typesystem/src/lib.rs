mod check;
mod env;
mod prettyprint;
mod r#trait;
mod r#type;

pub use env::{Insert, TEnv};
pub use r#trait::TraitRestriction;
pub use r#type::{ConcreteKind, Generic, TypeId, TypeKind};
