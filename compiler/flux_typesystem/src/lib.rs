mod check;
mod env;
mod r#trait;
mod r#type;

pub use env::{Insert, TEnv};
pub use r#trait::TraitRestriction;
pub use r#type::{ConcreteKind, Generic, TypeId, TypeKind};
