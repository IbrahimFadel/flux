use std::{
	collections::{HashMap, HashSet},
	fmt::Debug,
};

use flux_span::{FileId, Span, Spanned};
use flux_syntax::ast;
use flux_typesystem::r#type::{ConcreteKind, TypeId, TypeKind};
use hir::{ApplyDecl, Expr, FnDecl, GenericList, ModDecl, TraitDecl, Type, TypeDecl, UseDecl};
use indexmap::IndexMap;
use la_arena::{Arena, Idx};
use lasso::{Rodeo, Spur};
use lower::error::LowerError;
use tracing::{event, info, Level};

pub mod hir;
mod lower;

#[derive(Clone, Debug)]
pub struct HirModule {
	pub path: Vec<Spur>,
	pub exprs: Arena<Spanned<Expr>>,
	pub mods: Vec<ModDecl>,
	pub uses: Vec<UseDecl>,
	pub functions: Vec<FnDecl>,
	pub types: Vec<TypeDecl>,
	pub applies: Vec<ApplyDecl>,
	pub traits: Vec<TraitDecl>,
}

pub fn lower(
	path: Vec<Spur>,
	root: ast::Root,
	file_id: FileId,
	resolver: &Rodeo,
) -> (HirModule, Vec<LowerError>) {
	let mut ctx = lower::LoweringCtx::new(file_id, resolver);

	let mut errors = vec![];

	let span = tracing::span!(Level::INFO, "type decls");
	let _enter = span.enter();
	let types: Vec<TypeDecl> = root
		.types()
		.map(|ty| ctx.lower_type_decl(ty))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();
	drop(_enter);
	// info!("stop lowering type decls");

	info!("start lowering trait decls");
	// We need to populate LoweringCtx::traits before we can lower the applies, so it is necessary to lower these first
	let traits: Vec<TraitDecl> = root
		.traits()
		.map(|trt| ctx.lower_trait_decl(trt))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();
	info!("stop lowering trait decls");

	traits.iter().for_each(|trt| {
		ctx.traits.insert(trt.name.inner.clone(), trt);
	});

	info!("start lowering apply decls (first pass)");
	let applications: Vec<(
		(Option<(Spanned<Spur>, Vec<TypeId>)>, Spanned<Type>),
		Spanned<GenericList>,
		Option<ast::ApplyBlock>,
	)> = root
		.applies()
		.map(|apply| ctx.apply_decl_first_pass(apply))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();
	info!("stop lowering apply decls (first pass)");

	info!("start lowering apply decls (second pass)");
	let applies = applications
		.iter()
		.map(|((trt, ty), generics, block)| ctx.lower_apply_decl(block, trt, ty, generics))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();
	info!("stop lowering apply decls (second pass)");

	info!("start lowering fn decls");
	let functions = root
		.functions()
		.map(|f| ctx.lower_fn_decl(f, None, &IndexMap::new()))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();
	info!("stop lowering fn decls");

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
