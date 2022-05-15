use std::collections::HashMap;

use flux_error::filesystem::FileId;
use indexmap::IndexMap;
use smol_str::SmolStr;

mod database;
pub use database::Database;
use flux_syntax::ast::{self, Spanned};
use la_arena::Idx;

#[derive(Debug)]
pub struct HirModule {
	pub db: Database,
	pub functions: Vec<FnDecl>,
	pub types: Vec<TypeDecl>,
}

pub fn lower(ast: ast::Root, file_id: FileId) -> HirModule {
	let mut db = Database::new(file_id);
	let functions = ast.functions().filter_map(|f| db.lower_fn(f)).collect();
	let types = ast.types().filter_map(|ty| db.lower_ty_decl(ty)).collect();
	HirModule {
		db,
		functions,
		types,
	}
}

#[derive(Debug)]
pub struct TypeDecl {
	pub_: bool,
	pub name: String,
	pub ty: Spanned<Type>,
}

#[derive(Debug)]
pub struct FnDecl {
	pub name: Option<String>,
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

#[derive(Debug)]
pub enum Stmt {
	VarDecl(VarDecl),
	Expr(ExprIdx),
	If(If),
	Return(Return),
}

#[derive(Debug)]
pub struct Return {
	pub value: ExprIdx,
}

#[derive(Debug)]
pub struct VarDecl {
	pub ty: Spanned<Type>,
	pub name: SmolStr,
	pub value: Idx<Spanned<Expr>>,
}

#[derive(Debug)]
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
