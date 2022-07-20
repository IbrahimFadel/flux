use std::collections::{HashMap, HashSet};

use flux_span::Spanned;
use flux_typesystem::r#type::TypeId;
use indexmap::IndexMap;
use la_arena::Idx;
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
	Private,
	Public,
}

#[derive(Debug, Clone)]
pub struct ModDecl {
	pub name: Spanned<SmolStr>,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
	pub path: Vec<Spanned<SmolStr>>,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
	pub visibility: Spanned<Visibility>,
	pub name: Spanned<SmolStr>,
	pub ty: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct Block(pub Vec<Spanned<Stmt>>);

#[derive(Debug, Clone)]
pub struct FnDecl {
	pub visibility: Spanned<Visibility>,
	pub name: Spanned<SmolStr>,
	pub params: Spanned<Vec<Spanned<FnParam>>>,
	pub body: ExprIdx,
	pub return_type: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct FnParam {
	pub mutable: bool,
	pub ty: Spanned<Type>,
	pub name: Option<SmolStr>,
}

#[derive(Debug, Clone)]
pub struct ApplyDecl {
	pub trait_: Option<Spanned<SmolStr>>,
	pub ty: Spanned<Type>,
	pub methods: Vec<FnDecl>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
	VarDecl(VarDecl),
	Expr(ExprIdx),
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
	pub value: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct If {
	pub condition: ExprIdx,
	pub then: ExprIdx,
	pub else_: ExprIdx, // Expr::Missing if no else block
}

impl If {
	pub fn new(condition: ExprIdx, then: ExprIdx, else_: ExprIdx) -> Self {
		Self {
			condition,
			then,
			else_,
		}
	}
}

pub type ExprIdx = Idx<Spanned<Expr>>;

#[derive(Debug, Clone)]
pub enum Expr {
	Binary(Binary),
	Int(Int),
	Float(Float),
	Prefix { op: PrefixOp, expr: ExprIdx },
	Call(Call),
	Path(Path),
	Struct(Struct),
	If(If),
	Block(Block),
	Tuple(Tuple),
	Intrinsic(Intrinsic),
	Missing,
}

impl Into<Option<Binary>> for Expr {
	fn into(self) -> Option<Binary> {
		match self {
			Self::Binary(b) => Some(b),
			_ => None,
		}
	}
}

#[derive(Debug, Clone)]
pub enum Intrinsic {
	Malloc(ExprIdx),
}

#[derive(Debug, Clone)]
pub struct Tuple(pub Vec<ExprIdx>);

#[derive(Debug, Clone)]
pub struct Float {
	pub n: f64,
	pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Binary {
	pub op: BinaryOp,
	pub lhs: ExprIdx,
	pub rhs: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct Struct {
	pub name: Path,
	pub fields: Spanned<Vec<(Spanned<SmolStr>, ExprIdx)>>,
}

pub type Path = Vec<Spanned<SmolStr>>;

#[derive(Debug, Clone)]
pub struct Int {
	pub n: u64,
	pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct Call {
	pub callee: ExprIdx,
	pub args: Spanned<Vec<ExprIdx>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,
	CmpEq,
	DoubleColon,
}

#[derive(Debug, Clone)]
pub enum PrefixOp {
	Neg,
}

type BitSize = u32;

#[derive(Debug, Clone)]
pub enum Type {
	SInt(BitSize),
	UInt(BitSize),
	Int,
	F64,
	F32,
	Float,
	Ptr(TypeId),
	Ref(usize),
	Ident((SmolStr, Vec<TypeId>)),
	Generic((SmolStr, HashSet<SmolStr>)),
	Struct(StructType),
	Tuple(Vec<Type>),
	Func(Box<Type>, Box<Type>),
	Unknown,
}

#[derive(Debug, Clone)]
pub struct TraitDecl {
	pub name: Spanned<SmolStr>,
	pub methods: HashMap<SmolStr, TraitMethod>,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
	pub name: Spanned<SmolStr>,
	pub params: Spanned<Vec<Spanned<FnParam>>>,
	pub return_type: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct StructType(pub Spanned<IndexMap<SmolStr, StructTypeField>>);

#[derive(Debug, Clone)]
pub struct StructTypeField {
	pub visibility: Visibility,
	pub mutable: bool,
	pub ty: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct WhereClause(pub Vec<TypeRestriction>);

#[derive(Debug, Clone)]
pub struct TypeRestriction {
	pub name: Spanned<SmolStr>,
	pub trt: Spanned<SmolStr>,
}
