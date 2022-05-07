use smol_str::SmolStr;

mod database;
pub use database::Database;
use la_arena::Idx;
use flux_syntax::ast;

pub fn lower(ast: ast::Root) -> (Database, Vec<FnDecl>) {
	let mut db = Database::default();
	let functions = ast.functions().filter_map(|f| db.lower_fn(f)).collect();
	(db, functions)
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
	Missing,
}

#[derive(Debug)]
pub struct INType {
	bits: u32,
}
