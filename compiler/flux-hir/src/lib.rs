use std::fmt::Debug;

use flux_error::{FileId, FluxError};
use flux_syntax::ast;
use flux_typesystem::{ConcreteKind, Insert, TypeEnv, TypeKind};
use hir::{Expr, FnDecl, ModDecl, Spanned, Type, TypeDecl, UseDecl};
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
	let mut ctx = lower::LoweringCtx::new(lower::error::TypeCheckErrHandler, file_id);

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

impl Insert<Spanned<Type>> for TypeEnv {
	fn insert(&mut self, ty: Spanned<Type>) -> flux_typesystem::TypeId {
		match ty.node {
			Type::Unit => self.insert(flux_typesystem::Spanned {
				inner: TypeKind::Concrete(ConcreteKind::Unit),
				span: ty.span,
			}),
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
			_ => unreachable!(),
		}
	}
}

impl Into<Spanned<Type>> for flux_typesystem::Spanned<TypeKind> {
	fn into(self) -> Spanned<Type> {
		let ty = match self.inner {
			TypeKind::Concrete(t) => match t {
				ConcreteKind::SInt(n) => Type::SInt(n),
				ConcreteKind::UInt(n) => Type::UInt(n),
				ConcreteKind::F64 => Type::F64,
				ConcreteKind::F32 => Type::F32,
				ConcreteKind::Ident(name) => Type::Ident(name),
				ConcreteKind::Unit => Type::Unit,
			},
			TypeKind::Int(_) => Type::Int,
			TypeKind::Float(_) => Type::Float,
			TypeKind::Unknown => Type::Unknown,
			TypeKind::Ref(id) => Type::Ref(id),
		};
		Spanned::new(ty, self.span)
	}
}
