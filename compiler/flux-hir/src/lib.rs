use std::fmt::Debug;

use flux_error::{filesystem::FileId, FluxError, Span};
use flux_syntax::ast::{self, Spanned};
use flux_typesystem::{TypeData, TypeKind};
use hir::{Expr, FnDecl, ModDecl, Type, TypeDecl, UseDecl};
use la_arena::{Arena, Idx};
use smol_str::SmolStr;

pub mod hir;
mod lower;
mod print;

#[derive(Clone, Debug)]
pub struct HirModule {
	pub path: Vec<SmolStr>,
	pub exprs: Arena<Spanned<Expr>>,
	pub mods: Vec<ModDecl>,
	pub uses: Vec<UseDecl>,
	pub functions: Vec<FnDecl>,
	pub types: Vec<TypeDecl>,
}

pub fn lower(path: Vec<SmolStr>, ast: ast::Root, file_id: FileId) -> (HirModule, Vec<FluxError>) {
	let mut ctx = lower::LoweringCtx::new(file_id);

	let mut errors = vec![];
	let functions = ast
		.functions()
		.map(|f| ctx.lower_fn_decl(f))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();

	errors.append(&mut ctx.errors);

	(
		HirModule {
			path,
			exprs: ctx.exprs,
			mods: vec![],
			uses: vec![],
			functions,
			types: vec![],
		},
		errors,
	)
}

impl TypeData for Type {}

fn type_system_reconstruction_to_hir_type(ty: &flux_typesystem::Type<Type>) -> Type {
	match &ty.0 {
		TypeKind::Concrete(t) => t.clone(),
		TypeKind::Int(id) => {
			if let Some(_) = id {
				unreachable!()
			} else {
				Type::UInt(32)
			}
		}
		TypeKind::Float(id) => {
			if let Some(_) = id {
				unreachable!()
			} else {
				Type::F32
			}
		}
		TypeKind::Ref(_) => unreachable!(),
		TypeKind::Unknown => unreachable!(),
	}
}

// fn type_to_type_info(ty: &Type) -> TypeInfo {
// 	match ty {
// 		Type::F32 => TypeInfo::F32,
// 		Type::F64 => TypeInfo::F64,
// 		Type::SInt(n) => TypeInfo::SInt(*n),
// 		Type::UInt(n) => TypeInfo::UInt(*n),
// 		Type::Unit => TypeInfo::Unit,
// 		Type::Unknown => TypeInfo::Unknown,
// 		_ => todo!("uimplemented: {:#?}", ty),
// 	}
// }

// fn typsys_type_to_type(ty: &flux_typesystem::Type) -> Type {
// 	match ty {
// 		flux_typesystem::Type::Bool => todo!(),
// 		flux_typesystem::Type::F32 => Type::F32,
// 		flux_typesystem::Type::F64 => Type::F64,
// 		flux_typesystem::Type::SInt(n) => Type::SInt(*n),
// 		flux_typesystem::Type::UInt(n) => Type::UInt(*n),
// 		flux_typesystem::Type::Int => Type::UInt(32),
// 		flux_typesystem::Type::Unit => Type::Unit,
// 		flux_typesystem::Type::Func(_, _) => todo!(),
// 	}
// }
