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
	name: String,
	ty: Type,
}

#[derive(Debug)]
pub struct FnDecl {
	block: Vec<Option<Stmt>>,
	return_type: Type,
}

#[derive(Debug)]
pub enum Stmt {
	VarDecl {
		ty: Type,
		name: SmolStr,
		value: Expr,
	},
	Expr(Expr),
	If(If),
}

#[derive(Debug)]
pub struct If {
	condition: Expr,
	then: Vec<Option<Stmt>>,
	else_ifs: Vec<Option<Stmt>>,
	else_: Vec<Option<Stmt>>,
}

impl If {
	pub fn new(
		condition: Expr,
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

#[derive(Debug)]
pub enum Expr {
	Binary {
		op: BinaryOp,
		lhs: ExprIdx,
		rhs: ExprIdx,
	},
	Int {
		n: u64,
	},
	Prefix {
		op: PrefixOp,
		expr: ExprIdx,
	},
	Ident {
		val: SmolStr,
	},
	PrimitiveType {
		ty: PrimitiveKind,
	},
	Missing,
}

#[derive(Debug)]
pub enum PrimitiveKind {
	I32,
}

#[derive(Debug)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,
	CmpEq,
}

#[derive(Debug)]
pub enum PrefixOp {
	Neg,
}

#[derive(Debug)]
pub enum Type {
	INType(INType),
	UNType(UNType),
	StructType(StructType),
	Missing,
}

#[derive(Debug)]
pub struct StructType(IndexMap<String, StructField>);

#[derive(Debug)]
pub struct StructField {
	public: bool,
	mutable: bool,
	ty: Type,
}

#[derive(Debug)]
pub struct INType {
	bits: u32,
}

#[derive(Debug)]
pub struct UNType {
	bits: u32,
}
