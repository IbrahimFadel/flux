use std::collections::{HashMap, HashSet, VecDeque};

use flux_span::Spanned;
use flux_typesystem::r#type::TypeId;
use indexmap::IndexMap;
use la_arena::Idx;
use lasso::Spur;

mod print;

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
	Private,
	Public,
}

#[derive(Debug, Clone)]
pub struct ModDecl {
	pub name: Spanned<Spur>,
}

#[derive(Debug, Clone)]
pub struct UseDecl {
	pub path: Vec<Spanned<Spur>>,
}

#[derive(Debug, Clone)]
pub struct TypeDecl {
	pub visibility: Spanned<Visibility>,
	pub name: Spanned<Spur>,
	pub generics: Spanned<GenericList>,
	pub ty: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct Block(pub Vec<Spanned<Stmt>>);

#[derive(Debug, Clone)]
pub struct FnDecl {
	pub visibility: Spanned<Visibility>,
	pub name: Spanned<Spur>,
	pub params: Spanned<FnParams>,
	pub return_type: Spanned<Type>,
	pub body: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct FnParams(pub VecDeque<Spanned<FnParam>>); // TODO: VecDeque because we will prepend &self a lot, but probably benchmark this

#[derive(Debug, Clone)]
pub struct FnParam {
	pub mutable: bool,
	pub ty: Spanned<Type>,
	pub name: Spur,
}

#[derive(Debug, Clone)]
pub struct ApplyDecl {
	pub trait_: Option<(Spanned<Spur>, Vec<TypeId>)>,
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
	pub name: Spanned<Spur>,
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
	Access(Access),
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
	Address(Address),
	IdxMem(IdxMem),
	For(For),
	Enum(Enum),
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
pub struct Enum {
	pub path: Path,
	pub arg: Option<ExprIdx>,
}

#[derive(Debug, Clone)]
pub struct For {
	pub item: Spanned<Spur>,
	pub iterator: ExprIdx,
	pub block: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct IdxMem {
	pub val: ExprIdx,
	pub idx: ExprIdx,
}

#[derive(Debug, Clone)]
pub struct Access {
	pub lhs: ExprIdx,
	pub field: Spanned<Spur>,
}

#[derive(Debug, Clone)]
pub struct Address(pub ExprIdx);

#[derive(Debug, Clone)]
pub enum Intrinsic {
	Malloc(ExprIdx),
	Free(ExprIdx),
	Memcpy(Memcpy),
	Nullptr,
}

#[derive(Debug, Clone)]
pub struct Memcpy {
	pub dest: ExprIdx,
	pub src: ExprIdx,
	pub n: ExprIdx,
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
	pub fields: Spanned<Vec<(Spanned<Spur>, ExprIdx)>>,
}

pub type Path = Vec<Spanned<Spur>>;

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
	Access,
	Assign,
	CmpNeq,
	CmpGt,
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
	Ident((Spur, Vec<TypeId>)),
	Generic((Spur, HashSet<(Spur, Vec<TypeId>)>)),
	Struct(StructType),
	Enum(EnumType),
	Tuple(Vec<TypeId>),
	Func(Box<Type>, Box<Type>),
	Unknown,
}

pub type GenericList = IndexMap<Spur, HashSet<(Spur, Vec<TypeId>)>>;

#[derive(Debug, Clone)]
pub struct TraitDecl {
	pub name: Spanned<Spur>,
	pub generics: Spanned<GenericList>,
	pub methods: HashMap<Spur, TraitMethod>,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
	pub name: Spanned<Spur>,
	pub params: FnParams,
	pub return_type: Spanned<Type>,
}

#[derive(Debug, Clone)]
pub struct StructType(pub Spanned<IndexMap<Spur, StructTypeField>>);

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
	pub name: Spanned<Spur>,
	pub trt: Spanned<(Spur, Vec<TypeId>)>,
}

#[derive(Debug, Clone)]
pub struct EnumType(pub IndexMap<Spur, Option<Spanned<Type>>>);
