use std::sync::Arc;

use flux_diagnostics::ice;

use crate::r#type::{SIntTy, Type, UIntTy};

use super::*;

impl ToType for hir::TypeIdx {
    type Variant = ();

    fn to_type(&self, ctx: &LoweringCtx) -> TypeRef<Self::Variant> {
        ctx.bodies.types[self.raw()].to_type(ctx)
    }
}

impl ToType for hir::Type {
    type Variant = ();

    fn to_type(&self, ctx: &LoweringCtx) -> TypeRef<Self::Variant> {
        let v = match self {
            hir::Type::Array(t, n) => {
                let t = t.to_type(ctx);
                Type::Array(t, *n)
            }
            hir::Type::Generic(_, _) => todo!(),
            hir::Type::Path(path) => {
                let first = *path.nth(0);
                if first == ctx.string_interner.get_or_intern_static("u64") {
                    Type::UIntTy(UIntTy::U64)
                } else if first == ctx.string_interner.get_or_intern_static("u32") {
                    Type::UIntTy(UIntTy::U32)
                } else if first == ctx.string_interner.get_or_intern_static("u16") {
                    Type::UIntTy(UIntTy::U16)
                } else if first == ctx.string_interner.get_or_intern_static("u8") {
                    Type::UIntTy(UIntTy::U8)
                } else if first == ctx.string_interner.get_or_intern_static("s64") {
                    Type::SIntTy(SIntTy::S64)
                } else if first == ctx.string_interner.get_or_intern_static("s32") {
                    Type::SIntTy(SIntTy::S32)
                } else if first == ctx.string_interner.get_or_intern_static("s16") {
                    Type::SIntTy(SIntTy::S16)
                } else if first == ctx.string_interner.get_or_intern_static("s8") {
                    Type::SIntTy(SIntTy::S8)
                } else {
                    todo!()
                }
            }
            hir::Type::Ptr(ptr) => Type::Ptr(ptr.to_type(ctx)),
            hir::Type::Tuple(types) => Type::Tuple(types.iter().map(|t| t.to_type(ctx)).collect()),
            hir::Type::Unknown => ice("unknown type found when lowering to MIR"),
        };
        TypeRef::new(Arc::new(v))
    }
}
