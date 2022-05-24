use std::{collections::HashMap, fmt::Debug};

use flux_error::{filesystem::FileId, FluxError};
use indexmap::IndexMap;
use smol_str::SmolStr;

mod database;
pub use database::Database;
use flux_syntax::ast::{self, Spanned};
use la_arena::Idx;

#[derive(Clone)]
pub struct HirModule {
	pub name: String,
	pub db: Database,
	pub mods: Vec<ModDecl>,
	pub uses: Vec<UseDecl>,
	pub functions: Vec<FnDecl>,
	pub types: Vec<TypeDecl>,
}

impl Debug for HirModule {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name)
	}
}

pub fn lower(name: String, ast: ast::Root, file_id: FileId) -> (HirModule, Vec<FluxError>) {
	let mut db = Database::new(file_id);
	let functions: Vec<FnDecl> = ast.functions().filter_map(|f| db.lower_fn(f)).collect();
	let types: Vec<TypeDecl> = ast.types().filter_map(|ty| db.lower_ty_decl(ty)).collect();
	let mods: Vec<ModDecl> = ast.mods().filter_map(|m| db.lower_mod(m)).collect();
	let uses = ast.uses().filter_map(|u| db.lower_use(u)).collect();
	let mut errors = vec![];
	for f in &functions {
		if let Some(f_name) = &f.name {
			for ty in &types {
				if f_name.node == ty.name.node {
					errors.push(
						FluxError::default()
							.with_msg(format!("duplicate definitions for `{}`", f_name.node))
							.with_label(
								format!("function `{}` defined here", f_name.node),
								Some(f_name.span.clone()),
							)
							.with_label(
								format!("type `{}` defined here", ty.name.node),
								Some(ty.name.span.clone()),
							),
					);
				}
			}

			for m in &mods {
				if f_name.node == m.name.node {
					errors.push(
						FluxError::default()
							.with_msg(format!("duplicate definitions for `{}`", f_name.node))
							.with_label(
								format!("function `{}` defined here", f_name.node),
								Some(f_name.span.clone()),
							)
							.with_label(
								format!("mod `{}` defined here", m.name.node),
								Some(m.name.span.clone()),
							),
					);
				}
			}
		}
	}
	for ty in &types {
		for m in &mods {
			if ty.name.node == m.name.node {
				errors.push(
					FluxError::default()
						.with_msg(format!("duplicate definitions for `{}`", ty.name.node))
						.with_label(
							format!("type `{}` defined here", ty.name.node),
							Some(ty.name.span.clone()),
						)
						.with_label(
							format!("mod `{}` defined here", m.name.node),
							Some(m.name.span.clone()),
						),
				);
			}
		}
	}

	(
		HirModule {
			name,
			db,
			mods,
			uses,
			functions,
			types,
		},
		errors,
	)
}

#[derive(Debug, Clone)]
pub struct ModDecl {
	pub name: Spanned<String>,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
	pub path: Vec<Spanned<String>>,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
	pub public: Spanned<bool>,
	pub name: Spanned<String>,
	pub ty: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
	pub public: Spanned<bool>,
	pub name: Option<Spanned<String>>,
	pub params: Vec<Spanned<FnParam>>,
	pub block: Vec<Option<Spanned<Stmt>>>,
	pub return_type: Spanned<Type>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnParam {
	mutable: bool,
	pub ty: Spanned<Type>,
	pub name: Option<SmolStr>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
	VarDecl(VarDecl),
	Expr(ExprIdx),
	If(If),
	Return(Return),
}

#[derive(Debug, Clone)]
pub struct Return {
	pub value: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct VarDecl {
	pub ty: Spanned<Type>,
	pub name: SmolStr,
	pub value: Idx<Spanned<Expr>>,
}

#[derive(Debug, Clone)]
pub struct If {
	condition: ExprIdx,
	then: Vec<Option<Spanned<Stmt>>>,
	else_ifs: Vec<Option<Spanned<Stmt>>>,
	else_: Vec<Option<Spanned<Stmt>>>,
}

impl If {
	pub fn new(
		condition: ExprIdx,
		then: Vec<Option<Spanned<Stmt>>>,
		else_ifs: Vec<Option<Spanned<Stmt>>>,
		else_: Vec<Option<Spanned<Stmt>>>,
	) -> Self {
		Self {
			condition,
			then,
			else_ifs,
			else_,
		}
	}
}

pub type ExprIdx = Idx<Spanned<Expr>>;

#[derive(Debug, Clone)]
pub enum Expr {
	Binary {
		op: BinaryOp,
		lhs: ExprIdx,
		rhs: ExprIdx,
	},
	Int(Int),
	Float {
		n: f64,
	},
	Prefix {
		op: PrefixOp,
		expr: ExprIdx,
	},
	Ident {
		val: SmolStr,
	},
	Call(Call),
	Missing,
}

#[derive(Debug, Clone)]
pub struct Int {
	n: u64,
	pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Call {
	pub callee: ExprIdx,
	pub args: Vec<ExprIdx>,
}

#[derive(Debug, Clone)]
pub enum PrimitiveKind {
	I32,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,
	CmpEq,
}

#[derive(Debug, Clone)]
pub enum PrefixOp {
	Neg,
}

type BitSize = u32;

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
	INType(BitSize),
	UNType(BitSize),
	F64Type,
	F32Type,
	IdentType(SmolStr),
	StructType(StructType),
	InterfaceType(InterfaceType),
	VoidType,
	Missing,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InterfaceType(HashMap<String, InterfaceMethod>);

#[derive(Debug, PartialEq, Clone)]
pub struct InterfaceMethod {
	public: bool,
	params: Vec<Spanned<FnParam>>,
	return_ty: Spanned<Type>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructType(IndexMap<String, StructField>);

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
	public: bool,
	mutable: bool,
	ty: Spanned<Type>,
}
