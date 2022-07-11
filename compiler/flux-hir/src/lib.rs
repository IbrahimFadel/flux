use std::{collections::HashMap, fmt::Debug};

use flux_error::{FileId, FluxError};
use flux_syntax::ast;
use flux_typesystem::{ConcreteKind, Insert, TypeEnv, TypeKind};
use hir::{ApplyDecl, Expr, FnDecl, ModDecl, Spanned, TraitDecl, Type, TypeDecl, UseDecl};
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
	pub applies: Vec<ApplyDecl>,
	pub traits: Vec<TraitDecl>,
}

pub fn lower(path: Vec<SmolStr>, root: ast::Root, file_id: FileId) -> (HirModule, Vec<FluxError>) {
	let mut ctx = lower::LoweringCtx::new(lower::error::TypeCheckErrHandler, file_id);

	let mut errors = vec![];

	let traits = root
		.traits()
		.map(|trt| ctx.lower_trait_decl(trt))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();

	let functions = vec![];
	let applies = vec![];
	let types = root
		.types()
		.map(|ty| ctx.lower_type_decl(ty))
		.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
		.collect();

	// let functions = root
	// 	.functions()
	// 	.map(|f| ctx.lower_fn_decl(f))
	// 	.filter_map(|r| r.map_err(|e| errors.push(e)).ok())
	// 	.collect();

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

impl Insert<Spanned<Type>> for TypeEnv {
	fn insert(&mut self, ty: Spanned<Type>) -> flux_typesystem::TypeId {
		match ty.node {
			Type::SInt(n) => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Concrete(ConcreteKind::SInt(n)),
				span: ty.span,
			}),
			Type::UInt(n) => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Concrete(ConcreteKind::UInt(n)),
				span: ty.span,
			}),
			Type::Int => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Int(None),
				span: ty.span,
			}),
			Type::F64 => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Concrete(ConcreteKind::F64),
				span: ty.span,
			}),
			Type::F32 => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Concrete(ConcreteKind::F32),
				span: ty.span,
			}),
			Type::Float => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Float(None),
				span: ty.span,
			}),
			Type::Ident(name) => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Concrete(ConcreteKind::Ident(name)),
				span: ty.span,
			}),
			Type::Ref(id) => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Ref(id),
				span: ty.span,
			}),
			Type::Unknown => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Unknown,
				span: ty.span,
			}),
			Type::Tuple(types) => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Concrete(ConcreteKind::Tuple(types)),
				span: ty.span,
			}),
			_ => unreachable!(),
		}
	}
}

impl Into<Type> for TypeKind {
	fn into(self) -> Type {
		match self {
			TypeKind::Concrete(t) => match t {
				ConcreteKind::SInt(n) => Type::SInt(n),
				ConcreteKind::UInt(n) => Type::UInt(n),
				ConcreteKind::F64 => Type::F64,
				ConcreteKind::F32 => Type::F32,
				ConcreteKind::Ident(name) => Type::Ident(name),
				ConcreteKind::Tuple(types) => Type::Tuple(types.iter().map(|t| t.clone().into()).collect()),
				ConcreteKind::Func(i, o) => panic!(),
			},
			TypeKind::Int(_) => Type::Int,
			TypeKind::Float(_) => Type::Float,
			TypeKind::Unknown => Type::Unknown,
			TypeKind::Ref(id) => Type::Ref(id),
		}
	}
}

impl Into<Spanned<Type>> for flux_typesystem::Spanned<TypeKind> {
	fn into(self) -> Spanned<Type> {
		Spanned::new(self.inner.into(), self.span)
	}
}

impl Into<TypeKind> for Type {
	fn into(self) -> TypeKind {
		match self {
			Type::SInt(n) => TypeKind::Concrete(ConcreteKind::SInt(n)),
			Type::UInt(n) => TypeKind::Concrete(ConcreteKind::UInt(n)),
			Type::Int => TypeKind::Int(None),
			_ => unreachable!("{:?}", self),
		}
	}
}
