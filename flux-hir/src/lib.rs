use std::collections::HashMap;

use flux_error::filesystem::FileId;
use indexmap::IndexMap;
use smol_str::SmolStr;

mod database;
pub use database::Database;
use flux_syntax::ast;
use la_arena::Idx;

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
	pub ty: Type,
}

#[derive(Debug)]
pub struct FnDecl {
	pub block: Vec<Option<Stmt>>,
	pub return_type: Type,
}

#[derive(Debug, PartialEq)]
pub struct FnParam {
	mutable: bool,
	ty: Type,
	name: Option<SmolStr>,
}

#[derive(Debug)]
pub enum Stmt {
	VarDecl(VarDecl),
	Expr(Expr),
	If(If),
}

#[derive(Debug)]
pub struct VarDecl {
	pub ty: Type,
	pub name: SmolStr,
	pub value: Idx<Expr>,
}

#[derive(Debug)]
pub struct If {
	condition: Idx<Expr>,
	then: Vec<Option<Stmt>>,
	else_ifs: Vec<Option<Stmt>>,
	else_: Vec<Option<Stmt>>,
}

impl If {
	pub fn new(
		condition: Idx<Expr>,
		then: Vec<Option<Stmt>>,
		else_ifs: Vec<Option<Stmt>>,
		else_: Vec<Option<Stmt>>,
	) -> Self {
		Self {
			condition,
			then,
			else_ifs,
			else_,
		}
	}
}

type ExprIdx = Idx<Expr>;

#[derive(Debug, Clone)]
pub enum Expr {
	Binary {
		op: BinaryOp,
		lhs: ExprIdx,
		rhs: ExprIdx,
	},
	Int {
		n: u64,
	},
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
	Missing,
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

#[derive(Debug, PartialEq)]
pub enum Type {
	INType(INType),
	UNType(UNType),
	F64Type,
	F32Type,
	IdentType(SmolStr),
	StructType(StructType),
	InterfaceType(InterfaceType),
	VoidType,
	Missing,
}

#[derive(Debug, PartialEq)]
pub struct InterfaceType(HashMap<String, InterfaceMethod>);

#[derive(Debug, PartialEq)]
pub struct InterfaceMethod {
	public: bool,
	params: Vec<FnParam>,
	return_ty: Type,
}

#[derive(Debug, PartialEq)]
pub struct StructType(IndexMap<String, StructField>);

#[derive(Debug, PartialEq)]
pub struct StructField {
	public: bool,
	mutable: bool,
	ty: Type,
}

#[derive(Debug, PartialEq)]
pub struct INType {
	pub bits: u32,
}

#[derive(Debug, PartialEq)]
pub struct UNType {
	pub bits: u32,
}
