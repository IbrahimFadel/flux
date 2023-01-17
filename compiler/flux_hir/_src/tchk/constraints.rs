use std::fmt::Display;

use flux_span::{InFile, Span};
use lasso::Spur;

use super::r#type::TypeId;

#[derive(Debug, Clone)]
pub enum Constraint {
    TypeEq(TypeId, TypeId, InFile<Span>),
    FieldAccess {
        struct_ty: TypeId,
        field: Spur,
        field_ty: TypeId,
    },
}

impl Display for Constraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeEq(a, b, _) => write!(f, "{a} == {b}"),
            Self::FieldAccess {
                struct_ty,
                field,
                field_ty,
            } => write!(
                f,
                "{struct_ty} has field with name {field:?} of type {field_ty}",
            ),
        }
    }
}
