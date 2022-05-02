use smol_str::SmolStr;

mod database;
pub use database::Database;
use la_arena::Idx;
use pi_syntax::generated::ast;

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
}

type ExprIdx = Idx<Expr>;

#[derive(Debug)]
pub enum Expr {
	Binary {
		op: InfixOp,
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
pub enum InfixOp {
	Add,
	Sub,
	Mul,
	Div,
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
