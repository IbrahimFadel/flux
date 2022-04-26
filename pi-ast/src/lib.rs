use indexmap::IndexMap;
use pi_error::Span;
use smol_str::SmolStr;
use std::{
	collections::HashMap,
	fmt,
	hash::Hash,
	ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, Eq)]
pub struct Spanned<T> {
	pub node: T,
	pub span: Span,
}

impl<T> Spanned<T> {
	pub fn new(node: T, span: Span) -> Self {
		Self { node, span }
	}
}

impl<T> Deref for Spanned<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.node
	}
}

impl<T> DerefMut for Spanned<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.node
	}
}

impl<T: Hash> Hash for Spanned<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.node.hash(state);
	}
}

impl<T: std::cmp::PartialEq> PartialEq for Spanned<T> {
	fn eq(&self, other: &Self) -> bool {
		self.node == other.node
	}
}

#[derive(Debug)]
pub struct AST {
	pub name: String,
	pub mods: Vec<Spanned<Mod>>,
	pub functions: Vec<Spanned<FnDecl>>,
	pub types: Vec<Spanned<TypeDecl>>,
	pub apply_blocks: Vec<Spanned<ApplyBlock>>,
}

impl AST {
	pub fn new(
		name: String,
		mods: Vec<Spanned<Mod>>,
		functions: Vec<Spanned<FnDecl>>,
		types: Vec<Spanned<TypeDecl>>,
		apply_blocks: Vec<Spanned<ApplyBlock>>,
	) -> AST {
		AST {
			name,
			mods,
			functions,
			types,
			apply_blocks,
		}
	}
}

impl fmt::Display for AST {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:#?}", self)
	}
}

#[derive(Debug, Clone)]
pub struct ApplyBlock {
	pub interface_name: Option<Spanned<Ident>>,
	pub struct_name: Spanned<Ident>,
	pub methods: Vec<Spanned<FnDecl>>,
}

impl ApplyBlock {
	pub fn new(
		interface_name: Option<Spanned<Ident>>,
		struct_name: Spanned<Ident>,
		methods: Vec<Spanned<FnDecl>>,
	) -> ApplyBlock {
		ApplyBlock {
			interface_name,
			struct_name,
			methods,
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnDecl {
	pub pub_: Spanned<bool>,
	pub name: Spanned<Ident>,
	pub generics: Spanned<GenericTypes>,
	pub params: Spanned<Vec<Spanned<FnParam>>>,
	pub ret_ty: Spanned<Expr>,
	pub block: BlockStmt,
}

impl FnDecl {
	pub fn new(
		pub_: Spanned<bool>,
		name: Spanned<Ident>,
		generics: Spanned<GenericTypes>,
		params: Spanned<Vec<Spanned<FnParam>>>,
		ret_ty: Spanned<Expr>,
		block: BlockStmt,
	) -> FnDecl {
		FnDecl {
			pub_,
			name,
			generics,
			params,
			ret_ty,
			block,
		}
	}
}

impl fmt::Display for FnDecl {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{:#?}", self)
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnParam {
	pub mut_: Spanned<bool>,
	pub type_: Spanned<Expr>,
	pub name: Spanned<Ident>,
}

impl FnParam {
	pub fn new(mut_: Spanned<bool>, type_: Spanned<Expr>, name: Spanned<Ident>) -> FnParam {
		FnParam { mut_, type_, name }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeDecl {
	pub_: Spanned<bool>,
	pub name: Spanned<Ident>,
	pub type_: Spanned<Expr>,
}

impl TypeDecl {
	pub fn new(pub_: Spanned<bool>, name: Spanned<Ident>, type_: Spanned<Expr>) -> TypeDecl {
		TypeDecl { pub_, name, type_ }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Field {
	pub_: Spanned<bool>,
	pub type_: Spanned<Expr>,
	pub val: Option<Spanned<Expr>>,
}

impl Field {
	pub fn new(pub_: Spanned<bool>, type_: Spanned<Expr>, val: Option<Spanned<Expr>>) -> Field {
		Field { pub_, type_, val }
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Method {
	pub pub_: Spanned<bool>,
	pub name: Spanned<Ident>,
	pub params: Spanned<Vec<Spanned<FnParam>>>,
	pub ret_ty: Spanned<Expr>,
}

impl Method {
	pub fn new(
		pub_: Spanned<bool>,
		name: Spanned<Ident>,
		params: Spanned<Vec<Spanned<FnParam>>>,
		ret_ty: Spanned<Expr>,
	) -> Method {
		Method {
			pub_,
			name,
			params,
			ret_ty,
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OpKind {
	Plus,
	Minus,
	Asterisk,
	Slash,
	CmpLT,
	CmpLTE,
	CmpGT,
	CmpGTE,
	CmpEQ,
	CmpNE,
	CmpAnd,
	CmpOr,
	Doublecolon,
	Period,
	Eq,
	Ampersand,
	Illegal,
}

impl fmt::Display for OpKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			OpKind::Asterisk => write!(f, "*"),
			OpKind::Slash => write!(f, "/"),
			OpKind::Plus => write!(f, "+"),
			OpKind::Minus => write!(f, "-"),
			OpKind::CmpEQ => write!(f, "=="),
			_ => write!(f, "{:?}", self),
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
	Ident(Ident),
	BinOp(BinOp),
	IntLit(IntLit),
	FloatLit(FloatLit),
	CharLit(CharLit),
	StringLit(StringLit),
	BoolLit(BoolLit),
	PrimitiveType(PrimitiveType),
	PtrType(PtrType),
	StructType(StructType),
	InterfaceType(InterfaceType),
	EnumType(EnumType),
	CallExpr(CallExpr),
	Paren(Box<Expr>),
	Unary(Unary),
	StructExpr(StructExpr),
	EnumExpr(EnumExpr),
	Void,
	Error,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumExpr {
	pub enum_name: Spanned<Ident>,
	pub tag_name: Spanned<Ident>,
	pub val: Box<Spanned<Expr>>,
}

impl EnumExpr {
	pub fn new(
		enum_name: Spanned<Ident>,
		tag_name: Spanned<Ident>,
		val: Box<Spanned<Expr>>,
	) -> EnumExpr {
		EnumExpr {
			enum_name,
			tag_name,
			val,
		}
	}
}

// impl Into<Option<BinOp>> for Expr {
// 	fn into(self) -> Option<BinOp> {
// 		match self {
// 			Expr::BinOp(b) => Some(b),
// 			_ => None,
// 		}
// 	}
// }

// impl Into<Option<Ident>> for Expr {
// 	fn into(self) -> Option<Ident> {
// 		match self {
// 			Expr::Ident(x) => Some(x),
// 			_ => None,
// 		}
// 	}
// }

// impl Expr {
// 	pub fn binop(&self) -> &BinOp {
// 		if let Expr::BinOp(b) = self {
// 			b
// 		} else {
// 			panic!("expected binop")
// 		}
// 	}
// }

impl fmt::Display for Expr {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Expr::PrimitiveType(prim) => write!(f, "{:?}", prim),
			Expr::Ident(ident) => write!(f, "{}", ident.to_string()),
			_ => write!(f, "{:?}", self),
		}
	}
}

pub type EnumType = IndexMap<Spanned<Ident>, Spanned<Expr>>;

#[derive(Debug, PartialEq, Clone)]
pub struct StructExpr {
	pub name: Ident,
	pub fields: Spanned<IndexMap<Spanned<Ident>, Option<Box<Spanned<Expr>>>>>,
}

impl StructExpr {
	pub fn new(
		name: Ident,
		fields: Spanned<IndexMap<Spanned<Ident>, Option<Box<Spanned<Expr>>>>>,
	) -> StructExpr {
		StructExpr { name, fields }
	}
}

pub type Ident = SmolStr;

pub type PtrType = Box<Spanned<Expr>>;

#[derive(Debug, PartialEq, Clone)]
pub struct IntLit {
	pub negative: Spanned<bool>,
	pub signed: bool,
	pub bits: u8,
	pub val: Spanned<u64>,
}

impl IntLit {
	pub fn new(negative: Spanned<bool>, signed: bool, bits: u8, val: Spanned<u64>) -> IntLit {
		IntLit {
			negative,
			signed,
			bits,
			val,
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct FloatLit {
	pub negative: Spanned<bool>,
	pub bits: u8,
	pub val: Spanned<f64>,
}

impl FloatLit {
	pub fn new(negative: Spanned<bool>, bits: u8, val: Spanned<f64>) -> FloatLit {
		FloatLit {
			negative,
			bits,
			val,
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Unary {
	pub op: OpKind,
	pub val: Box<Spanned<Expr>>,
}

impl Unary {
	pub fn new(op: OpKind, val: Box<Spanned<Expr>>) -> Unary {
		Unary { op, val }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallExpr {
	pub callee: Box<Spanned<Expr>>,
	pub args: Spanned<Vec<Box<Spanned<Expr>>>>,
}

impl CallExpr {
	pub fn new(callee: Box<Spanned<Expr>>, args: Spanned<Vec<Box<Spanned<Expr>>>>) -> CallExpr {
		CallExpr { callee, args }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinOp {
	pub x: Box<Spanned<Expr>>,
	pub op: OpKind,
	pub y: Box<Spanned<Expr>>,
}

impl BinOp {
	pub fn new(x: Box<Spanned<Expr>>, op: OpKind, y: Box<Spanned<Expr>>) -> BinOp {
		BinOp { x, op, y }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveType {
	I64,
	U64,
	I32,
	U32,
	I16,
	U16,
	I8,
	U8,
	F64,
	F32,
	Bool,
	Void,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
	FnDecl(FnDecl),
	TypeDecl(TypeDecl),
	BlockStmt(BlockStmt),
	VarDecl(VarDecl),
	If(If),
	For(For),
	ExprStmt(Expr),
	Return(Return),
	Mod(Mod),
	Error,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Mod {
	pub_: Spanned<bool>,
	pub name: Spanned<Ident>,
}

impl Mod {
	pub fn new(pub_: Spanned<bool>, name: Spanned<Ident>) -> Mod {
		Mod { pub_, name }
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDecl {
	pub mut_: Spanned<bool>,
	pub type_: Spanned<Expr>,
	pub names: Vec<Spanned<Ident>>,
	pub values: Vec<Spanned<Expr>>,
}

impl VarDecl {
	pub fn new(
		mut_: Spanned<bool>,
		type_: Spanned<Expr>,
		names: Vec<Spanned<Ident>>,
		values: Vec<Spanned<Expr>>,
	) -> VarDecl {
		VarDecl {
			mut_,
			type_,
			names,
			values,
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct If {
	pub condition: Box<Spanned<Expr>>,
	pub then: BlockStmt,
	pub else_: Option<BlockStmt>,
}

impl If {
	pub fn new(condition: Box<Spanned<Expr>>, then: BlockStmt, else_: Option<BlockStmt>) -> If {
		If {
			condition,
			then,
			else_,
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct For {}

impl For {
	pub fn new() -> For {
		For {}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
	pub val: Option<Spanned<Expr>>,
}

impl Return {
	pub fn new(val: Option<Spanned<Expr>>) -> Return {
		Return { val }
	}
}

pub type CharLit = char;
pub type StringLit = SmolStr;
pub type BoolLit = bool;
pub type GenericTypes = Vec<Spanned<Ident>>;
pub type BlockStmt = Vec<Spanned<Stmt>>;
pub type StructType = IndexMap<Spanned<Ident>, Spanned<Field>>;
pub type InterfaceType = HashMap<Spanned<Ident>, Spanned<Method>>;
