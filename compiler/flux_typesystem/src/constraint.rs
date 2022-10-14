use lasso::Spur;

use crate::r#type::TypeId;

pub enum Constraint {
    FieldAccess {
        struct_ty: TypeId,
        field: Spur,
        field_ty: TypeId,
    },
    // ImplementsTrait(Spur),
}
