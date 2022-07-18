use std::{collections::HashMap, fmt::Debug};

use flux_error::Error;
use flux_span::{FileId, Span, Spanned};
use flux_syntax::ast;
use flux_typesystem::{
	infer::TypeEnv,
	r#type::{ConcreteKind, TypeId, TypeKind},
};
use hir::{ApplyDecl, Expr, FnDecl, ModDecl, TraitDecl, Type, TypeDecl, UseDecl};
use la_arena::{Arena, Idx};
use lower::error::LowerError;
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
	pub applies: Vec<ApplyDecl>,
	pub traits: Vec<TraitDecl>,
}

pub fn lower(path: Vec<SmolStr>, root: ast::Root, file_id: FileId) -> (HirModule, Vec<LowerError>) {
	let mut ctx = lower::LoweringCtx::new(file_id);

	let mut errors = vec![];

	// We need to populate LoweringCtx::traits before we can lower the applies, so it is necessary to lower these first
	let traits: Vec<TraitDecl> = root
		.traits()
		.map(|trt| ctx.lower_trait_decl(trt))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();

	traits.iter().for_each(|trt| {
		ctx.traits.insert(trt.name.inner.clone(), trt);
	});

	let types = root
		.types()
		.map(|ty| ctx.lower_type_decl(ty))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();

	let applies = root
		.applies()
		.map(|apply| ctx.lower_apply_decl(apply))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();

	let functions = root
		.functions()
		.map(|f| ctx.lower_fn_decl(f))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();

	// let applies = root
	// 	.applies()
	// 	.map(|apply| ctx.lower_apply_decl(apply))
	// 	.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
	// 	.collect();

	errors.append(&mut ctx.errors);

	(
		HirModule {
			path,
			exprs: ctx.exprs,
			mods: vec![],
			uses: vec![],
			functions,
			types,
			applies,
			traits,
		},
		errors,
	)
}

fn to_ty_kind(ty: &Spanned<Type>) -> Spanned<TypeKind> {
	let kind = match &ty.inner {
		Type::SInt(n) => TypeKind::Concrete(ConcreteKind::SInt(*n)),
		Type::UInt(n) => TypeKind::Concrete(ConcreteKind::UInt(*n)),
		Type::Int => TypeKind::Int(None),
		Type::F64 => TypeKind::Concrete(ConcreteKind::F64),
		Type::F32 => TypeKind::Concrete(ConcreteKind::F32),
		Type::Float => TypeKind::Float(None),
		Type::Ident(name) => TypeKind::Concrete(ConcreteKind::Ident(name.clone())),
		Type::Unknown => TypeKind::Unknown,
		_ => todo!(),
	};
	Spanned {
		inner: kind,
		span: ty.span.clone(),
	}
}
