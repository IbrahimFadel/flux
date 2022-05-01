// use pi_syntax::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

// pub trait AstNode {
// 	fn cast(syntax: SyntaxNode) -> Option<Self>
// 	where
// 		Self: Sized;
// 	fn syntax(&self) -> &SyntaxNode;
// }

// #[derive(Debug)]
// pub struct Root(SyntaxNode);

// impl AstNode for Root {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		match syntax.kind() {
// 			SyntaxKind::Root => Some(Self(syntax)),
// 			_ => None,
// 		}
// 	}

// 	fn syntax(&self) -> &SyntaxNode {
// 		&self.0
// 	}
// }

// impl Root {
// 	pub fn stmts(&self) -> impl Iterator<Item = Stmt> {
// 		self.0.children().filter_map(Stmt::cast)
// 	}

// 	pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
// 		self.0.children().filter_map(FnDecl::cast)
// 	}
// }

// #[derive(Debug)]
// pub struct VarDecl(SyntaxNode);

// impl AstNode for VarDecl {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		match syntax.kind() {
// 			SyntaxKind::VarDecl => Some(Self(syntax)),
// 			_ => None,
// 		}
// 	}

// 	fn syntax(&self) -> &SyntaxNode {
// 		&self.0
// 	}
// }

// impl VarDecl {
// 	pub fn name(&self) -> Option<SyntaxToken> {
// 		self
// 			.0
// 			.children_with_tokens()
// 			.filter_map(SyntaxElement::into_token)
// 			.find(|token| token.kind() == SyntaxKind::Ident)
// 	}

// 	pub fn value(&self) -> Option<Expr> {
// 		self.0.children().find_map(Expr::cast)
// 	}
// }

// #[derive(Debug)]
// pub enum Expr {
// 	BinExpr(BinExpr),
// 	IntExpr(IntExpr),
// 	ParenExpr(ParenExpr),
// 	PrefixExpr(PrefixExpr),
// 	IdentExpr(IdentExpr),
// }

// impl AstNode for Expr {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		let result = match syntax.kind() {
// 			SyntaxKind::BinExpr => Self::BinExpr(BinExpr(syntax)),
// 			SyntaxKind::IntLit => Self::IntExpr(IntExpr(syntax)),
// 			SyntaxKind::ParenExpr => Self::ParenExpr(ParenExpr(syntax)),
// 			SyntaxKind::PrefixExpr => Self::PrefixExpr(PrefixExpr(syntax)),
// 			SyntaxKind::Ident => Self::IdentExpr(IdentExpr(syntax)),
// 			_ => return None,
// 		};

// 		Some(result)
// 	}

// 	fn syntax(&self) -> &SyntaxNode {
// 		match self {
// 			Expr::BinExpr(node) => &node.0,
// 			Expr::IntExpr(node) => &node.0,
// 			Expr::ParenExpr(node) => &node.0,
// 			Expr::PrefixExpr(node) => &node.0,
// 			Expr::IdentExpr(node) => &node.0,
// 		}
// 	}
// }

// impl Expr {}

// #[derive(Debug)]
// pub enum Type {
// 	PrimitiveType(PrimitiveType),
// }

// impl Type {
// 	pub fn cast(node: SyntaxNode) -> Option<Self> {
// 		println!("{:?}", node);
// 		let result = match node.kind() {
// 			SyntaxKind::INKw => Self::PrimitiveType(PrimitiveType::I32),
// 			_ => return None,
// 		};

// 		Some(result)
// 	}
// }

// #[derive(Debug)]
// pub enum PrimitiveType {
// 	I32,
// }

// impl PrimitiveType {
// 	// pub fn kind() ->
// }

// #[derive(Debug)]
// pub struct BinExpr(SyntaxNode);

// impl BinExpr {
// 	pub fn lhs(&self) -> Option<Expr> {
// 		self.0.children().find_map(Expr::cast)
// 	}

// 	pub fn rhs(&self) -> Option<Expr> {
// 		self.0.children().filter_map(Expr::cast).nth(1)
// 	}

// 	pub fn op(&self) -> Option<SyntaxToken> {
// 		self
// 			.0
// 			.children_with_tokens()
// 			.filter_map(SyntaxElement::into_token)
// 			.find(|token| {
// 				matches!(
// 					token.kind(),
// 					SyntaxKind::Plus | SyntaxKind::Minus | SyntaxKind::Star | SyntaxKind::Slash,
// 				)
// 			})
// 	}
// }

// #[derive(Debug)]
// pub struct IntExpr(SyntaxNode);

// impl IntExpr {
// 	pub fn parse(&self) -> u64 {
// 		self.0.first_token().unwrap().text().parse().unwrap()
// 	}
// }

// #[derive(Debug)]
// pub struct ParenExpr(SyntaxNode);

// impl ParenExpr {
// 	pub fn expr(&self) -> Option<Expr> {
// 		self.0.children().find_map(Expr::cast)
// 	}
// }

// #[derive(Debug)]
// pub struct PrefixExpr(SyntaxNode);

// impl PrefixExpr {
// 	pub fn expr(&self) -> Option<Expr> {
// 		self.0.children().find_map(Expr::cast)
// 	}

// 	pub fn op(&self) -> Option<SyntaxToken> {
// 		self
// 			.0
// 			.children_with_tokens()
// 			.filter_map(SyntaxElement::into_token)
// 			.find(|token| token.kind() == SyntaxKind::Minus)
// 	}
// }

// #[derive(Debug)]
// pub struct IdentExpr(SyntaxNode);

// impl IdentExpr {
// 	pub fn name(&self) -> Option<SyntaxToken> {
// 		self.0.first_token()
// 	}
// }

// #[derive(Debug)]
// pub enum Stmt {
// 	VarDecl(VarDecl),
// 	Expr(Expr),
// }

// impl Stmt {
// 	pub fn cast(node: SyntaxNode) -> Option<Self> {
// 		let result = match node.kind() {
// 			SyntaxKind::VarDecl => Self::VarDecl(VarDecl(node)),
// 			_ => Self::Expr(Expr::cast(node)?),
// 		};

// 		Some(result)
// 	}
// }

// #[derive(Debug)]
// pub struct FnParams(SyntaxNode);

// impl FnParams {
// 	pub fn cast(node: SyntaxNode) -> Option<Self> {
// 		match node.kind() {
// 			SyntaxKind::FnParams => Some(Self(node)),
// 			_ => None,
// 		}
// 	}
// }

// #[derive(Debug)]
// pub struct Block(SyntaxNode);

// impl Block {
// 	pub fn cast(node: SyntaxNode) -> Option<Self> {
// 		match node.kind() {
// 			SyntaxKind::Block => Some(Self(node)),
// 			_ => None,
// 		}
// 	}

// 	// pub fn stmts(&self) -> Option<Vec<Stmt>> {
// 	// self.0.children().
// 	// }
// }

// #[derive(Debug)]
// pub struct FnDecl(SyntaxNode);

// impl FnDecl {
// 	pub fn cast(node: SyntaxNode) -> Option<Self> {
// 		let result = match node.kind() {
// 			SyntaxKind::FnDecl => Self(node),
// 			_ => panic!(),
// 		};

// 		Some(result)
// 	}

// 	pub fn name(&self) -> Option<SyntaxToken> {
// 		self
// 			.0
// 			.children_with_tokens()
// 			.filter_map(SyntaxElement::into_token)
// 			.find(|token| token.kind() == SyntaxKind::Ident)
// 	}

// 	pub fn params(&self) -> Option<FnParams> {
// 		self.0.children().find_map(FnParams::cast)
// 	}

// 	pub fn return_type(&self) -> Option<Expr> {
// 		self.0.children().find_map(Expr::cast)
// 	}

// 	pub fn block(&self) -> Option<Block> {
// 		self.0.children().find_map(Block::cast)
// 	}
// }

// // ----------------------------------------------------------------------------

// // use indexmap::IndexMap;
// // use pi_error::Span;
// // use serde::{Deserialize, Serialize};
// // use smol_str::SmolStr;
// // use std::{
// // 	collections::HashMap,
// // 	fmt,
// // 	hash::Hash,
// // 	ops::{Deref, DerefMut},
// // };

// // #[derive(Debug, Clone, Eq, Serialize, Deserialize)]
// // pub struct Spanned<T> {
// // 	pub node: T,
// // 	pub span: Span,
// // }

// // impl<T> Spanned<T> {
// // 	pub fn new(node: T, span: Span) -> Self {
// // 		Self { node, span }
// // 	}
// // }

// // impl<T> Deref for Spanned<T> {
// // 	type Target = T;

// // 	fn deref(&self) -> &Self::Target {
// // 		&self.node
// // 	}
// // }

// // impl<T> DerefMut for Spanned<T> {
// // 	fn deref_mut(&mut self) -> &mut Self::Target {
// // 		&mut self.node
// // 	}
// // }

// // impl<T: Hash> Hash for Spanned<T> {
// // 	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
// // 		self.node.hash(state);
// // 	}
// // }

// // impl<T: std::cmp::PartialEq> PartialEq for Spanned<T> {
// // 	fn eq(&self, other: &Self) -> bool {
// // 		self.node == other.node
// // 	}
// // }

// // #[derive(Debug, Serialize, Deserialize)]
// // pub struct AST {
// // 	pub name: String,
// // 	pub mods: Vec<Spanned<Mod>>,
// // 	pub functions: Vec<Spanned<FnDecl>>,
// // 	pub types: Vec<Spanned<TypeDecl>>,
// // 	pub apply_blocks: Vec<Spanned<ApplyBlock>>,
// // }

// // impl AST {
// // 	pub fn new(
// // 		name: String,
// // 		mods: Vec<Spanned<Mod>>,
// // 		functions: Vec<Spanned<FnDecl>>,
// // 		types: Vec<Spanned<TypeDecl>>,
// // 		apply_blocks: Vec<Spanned<ApplyBlock>>,
// // 	) -> AST {
// // 		AST {
// // 			name,
// // 			mods,
// // 			functions,
// // 			types,
// // 			apply_blocks,
// // 		}
// // 	}
// // }

// // impl fmt::Display for AST {
// // 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// // 		write!(f, "{:#?}", self)
// // 	}
// // }

// // #[derive(Debug, Clone, Serialize, Deserialize)]
// // pub struct ApplyBlock {
// // 	pub interface_name: Option<Spanned<Ident>>,
// // 	pub struct_name: Spanned<Ident>,
// // 	pub methods: Vec<Spanned<FnDecl>>,
// // }

// // impl ApplyBlock {
// // 	pub fn new(
// // 		interface_name: Option<Spanned<Ident>>,
// // 		struct_name: Spanned<Ident>,
// // 		methods: Vec<Spanned<FnDecl>>,
// // 	) -> ApplyBlock {
// // 		ApplyBlock {
// // 			interface_name,
// // 			struct_name,
// // 			methods,
// // 		}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct FnDecl {
// // 	pub pub_: Spanned<bool>,
// // 	pub name: Spanned<Ident>,
// // 	pub generics: Spanned<GenericTypes>,
// // 	pub params: Spanned<Vec<Spanned<FnParam>>>,
// // 	pub ret_ty: Spanned<Expr>,
// // 	pub block: BlockStmt,
// // }

// // impl FnDecl {
// // 	pub fn new(
// // 		pub_: Spanned<bool>,
// // 		name: Spanned<Ident>,
// // 		generics: Spanned<GenericTypes>,
// // 		params: Spanned<Vec<Spanned<FnParam>>>,
// // 		ret_ty: Spanned<Expr>,
// // 		block: BlockStmt,
// // 	) -> FnDecl {
// // 		FnDecl {
// // 			pub_,
// // 			name,
// // 			generics,
// // 			params,
// // 			ret_ty,
// // 			block,
// // 		}
// // 	}
// // }

// // impl fmt::Display for FnDecl {
// // 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// // 		write!(f, "{:#?}", self)
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct FnParam {
// // 	pub mut_: Spanned<bool>,
// // 	pub type_: Spanned<Expr>,
// // 	pub name: Spanned<Ident>,
// // }

// // impl FnParam {
// // 	pub fn new(mut_: Spanned<bool>, type_: Spanned<Expr>, name: Spanned<Ident>) -> FnParam {
// // 		FnParam { mut_, type_, name }
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct TypeDecl {
// // 	pub_: Spanned<bool>,
// // 	pub name: Spanned<Ident>,
// // 	pub type_: Spanned<Expr>,
// // }

// // impl TypeDecl {
// // 	pub fn new(pub_: Spanned<bool>, name: Spanned<Ident>, type_: Spanned<Expr>) -> TypeDecl {
// // 		TypeDecl { pub_, name, type_ }
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct Field {
// // 	pub_: Spanned<bool>,
// // 	pub type_: Spanned<Expr>,
// // 	pub val: Option<Spanned<Expr>>,
// // }

// // impl Field {
// // 	pub fn new(pub_: Spanned<bool>, type_: Spanned<Expr>, val: Option<Spanned<Expr>>) -> Field {
// // 		Field { pub_, type_, val }
// // 	}
// // }

// // #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// // pub struct Method {
// // 	pub pub_: Spanned<bool>,
// // 	pub name: Spanned<Ident>,
// // 	pub params: Spanned<Vec<Spanned<FnParam>>>,
// // 	pub ret_ty: Spanned<Expr>,
// // }

// // impl Method {
// // 	pub fn new(
// // 		pub_: Spanned<bool>,
// // 		name: Spanned<Ident>,
// // 		params: Spanned<Vec<Spanned<FnParam>>>,
// // 		ret_ty: Spanned<Expr>,
// // 	) -> Method {
// // 		Method {
// // 			pub_,
// // 			name,
// // 			params,
// // 			ret_ty,
// // 		}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
// // pub enum OpKind {
// // 	Plus,
// // 	Minus,
// // 	Asterisk,
// // 	Slash,
// // 	CmpLT,
// // 	CmpLTE,
// // 	CmpGT,
// // 	CmpGTE,
// // 	CmpEQ,
// // 	CmpNE,
// // 	CmpAnd,
// // 	CmpOr,
// // 	Doublecolon,
// // 	Period,
// // 	Eq,
// // 	Ampersand,
// // 	Illegal,
// // }

// // impl fmt::Display for OpKind {
// // 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// // 		match self {
// // 			OpKind::Asterisk => write!(f, "*"),
// // 			OpKind::Slash => write!(f, "/"),
// // 			OpKind::Plus => write!(f, "+"),
// // 			OpKind::Minus => write!(f, "-"),
// // 			OpKind::CmpEQ => write!(f, "=="),
// // 			_ => write!(f, "{:?}", self),
// // 		}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub enum Expr {
// // 	Ident(Ident),
// // 	BinOp(BinOp),
// // 	IntLit(IntLit),
// // 	FloatLit(FloatLit),
// // 	CharLit(CharLit),
// // 	StringLit(StringLit),
// // 	BoolLit(BoolLit),
// // 	PrimitiveType(PrimitiveType),
// // 	PtrType(PtrType),
// // 	StructType(StructType),
// // 	InterfaceType(InterfaceType),
// // 	EnumType(EnumType),
// // 	CallExpr(CallExpr),
// // 	Paren(Box<Expr>),
// // 	Unary(Unary),
// // 	StructExpr(StructExpr),
// // 	EnumExpr(EnumExpr),
// // 	Void,
// // 	Error,
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct EnumExpr {
// // 	pub enum_name: Spanned<Ident>,
// // 	pub tag_name: Spanned<Ident>,
// // 	pub val: Box<Spanned<Expr>>,
// // }

// // impl EnumExpr {
// // 	pub fn new(
// // 		enum_name: Spanned<Ident>,
// // 		tag_name: Spanned<Ident>,
// // 		val: Box<Spanned<Expr>>,
// // 	) -> EnumExpr {
// // 		EnumExpr {
// // 			enum_name,
// // 			tag_name,
// // 			val,
// // 		}
// // 	}
// // }

// // // impl Into<Option<BinOp>> for Expr {
// // // 	fn into(self) -> Option<BinOp> {
// // // 		match self {
// // // 			Expr::BinOp(b) => Some(b),
// // // 			_ => None,
// // // 		}
// // // 	}
// // // }

// // // impl Into<Option<Ident>> for Expr {
// // // 	fn into(self) -> Option<Ident> {
// // // 		match self {
// // // 			Expr::Ident(x) => Some(x),
// // // 			_ => None,
// // // 		}
// // // 	}
// // // }

// // // impl Expr {
// // // 	pub fn binop(&self) -> &BinOp {
// // // 		if let Expr::BinOp(b) = self {
// // // 			b
// // // 		} else {
// // // 			panic!("expected binop")
// // // 		}
// // // 	}
// // // }

// // impl fmt::Display for Expr {
// // 	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
// // 		match self {
// // 			Expr::PrimitiveType(prim) => write!(f, "{:?}", prim),
// // 			Expr::Ident(ident) => write!(f, "{}", ident.to_string()),
// // 			_ => write!(f, "{:?}", self),
// // 		}
// // 	}
// // }

// // pub type EnumType = IndexMap<Spanned<Ident>, Spanned<Expr>>;

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct StructExpr {
// // 	pub name: Ident,
// // 	pub fields: Spanned<IndexMap<Spanned<Ident>, Option<Box<Spanned<Expr>>>>>,
// // }

// // impl StructExpr {
// // 	pub fn new(
// // 		name: Ident,
// // 		fields: Spanned<IndexMap<Spanned<Ident>, Option<Box<Spanned<Expr>>>>>,
// // 	) -> StructExpr {
// // 		StructExpr { name, fields }
// // 	}
// // }

// // pub type Ident = SmolStr;

// // pub type PtrType = Box<Spanned<Expr>>;

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct IntLit {
// // 	pub negative: Spanned<bool>,
// // 	pub signed: bool,
// // 	pub bits: u8,
// // 	pub val: Spanned<u64>,
// // }

// // impl IntLit {
// // 	pub fn new(negative: Spanned<bool>, signed: bool, bits: u8, val: Spanned<u64>) -> IntLit {
// // 		IntLit {
// // 			negative,
// // 			signed,
// // 			bits,
// // 			val,
// // 		}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct FloatLit {
// // 	pub negative: Spanned<bool>,
// // 	pub bits: u8,
// // 	pub val: Spanned<f64>,
// // }

// // impl FloatLit {
// // 	pub fn new(negative: Spanned<bool>, bits: u8, val: Spanned<f64>) -> FloatLit {
// // 		FloatLit {
// // 			negative,
// // 			bits,
// // 			val,
// // 		}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct Unary {
// // 	pub op: OpKind,
// // 	pub val: Box<Spanned<Expr>>,
// // }

// // impl Unary {
// // 	pub fn new(op: OpKind, val: Box<Spanned<Expr>>) -> Unary {
// // 		Unary { op, val }
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct CallExpr {
// // 	pub callee: Box<Spanned<Expr>>,
// // 	pub args: Spanned<Vec<Box<Spanned<Expr>>>>,
// // }

// // impl CallExpr {
// // 	pub fn new(callee: Box<Spanned<Expr>>, args: Spanned<Vec<Box<Spanned<Expr>>>>) -> CallExpr {
// // 		CallExpr { callee, args }
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct BinOp {
// // 	pub x: Box<Spanned<Expr>>,
// // 	pub op: OpKind,
// // 	pub y: Box<Spanned<Expr>>,
// // }

// // impl BinOp {
// // 	pub fn new(x: Box<Spanned<Expr>>, op: OpKind, y: Box<Spanned<Expr>>) -> BinOp {
// // 		BinOp { x, op, y }
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub enum PrimitiveType {
// // 	I64,
// // 	U64,
// // 	I32,
// // 	U32,
// // 	I16,
// // 	U16,
// // 	I8,
// // 	U8,
// // 	F64,
// // 	F32,
// // 	Bool,
// // 	Void,
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub enum Stmt {
// // 	FnDecl(FnDecl),
// // 	TypeDecl(TypeDecl),
// // 	BlockStmt(BlockStmt),
// // 	VarDecl(VarDecl),
// // 	If(If),
// // 	For(For),
// // 	ExprStmt(Expr),
// // 	Return(Return),
// // 	Mod(Mod),
// // 	Error,
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct Mod {
// // 	pub_: Spanned<bool>,
// // 	pub name: Spanned<Ident>,
// // }

// // impl Mod {
// // 	pub fn new(pub_: Spanned<bool>, name: Spanned<Ident>) -> Mod {
// // 		Mod { pub_, name }
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct VarDecl {
// // 	pub mut_: Spanned<bool>,
// // 	pub type_: Spanned<Expr>,
// // 	pub names: Vec<Spanned<Ident>>,
// // 	pub values: Vec<Spanned<Expr>>,
// // }

// // impl VarDecl {
// // 	pub fn new(
// // 		mut_: Spanned<bool>,
// // 		type_: Spanned<Expr>,
// // 		names: Vec<Spanned<Ident>>,
// // 		values: Vec<Spanned<Expr>>,
// // 	) -> VarDecl {
// // 		VarDecl {
// // 			mut_,
// // 			type_,
// // 			names,
// // 			values,
// // 		}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct If {
// // 	pub condition: Box<Spanned<Expr>>,
// // 	pub then: BlockStmt,
// // 	pub else_: Option<BlockStmt>,
// // }

// // impl If {
// // 	pub fn new(condition: Box<Spanned<Expr>>, then: BlockStmt, else_: Option<BlockStmt>) -> If {
// // 		If {
// // 			condition,
// // 			then,
// // 			else_,
// // 		}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct For {}

// // impl For {
// // 	pub fn new() -> For {
// // 		For {}
// // 	}
// // }

// // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // pub struct Return {
// // 	pub val: Option<Spanned<Expr>>,
// // }

// // impl Return {
// // 	pub fn new(val: Option<Spanned<Expr>>) -> Return {
// // 		Return { val }
// // 	}
// // }

// // pub type CharLit = char;
// // pub type StringLit = SmolStr;
// // pub type BoolLit = bool;
// // pub type GenericTypes = Vec<Spanned<Ident>>;
// // pub type BlockStmt = Vec<Spanned<Stmt>>;
// // pub type StructType = IndexMap<Spanned<Ident>, Spanned<Field>>;
// // pub type InterfaceType = HashMap<Spanned<Ident>, Spanned<Method>>;

// // // #[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
// // // pub struct InterfaceType(pub HashMap<Spanned<Ident>, Spanned<Method>>);

// // // impl Serialize for InterfaceType {
// // // 	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
// // // 	where
// // // 		S: Serializer,
// // // 	{
// // // 		let mut map = serializer.serialize_map(Some(self.len()))?;
// // // 		for (k, v) in self {
// // // 			map.serialize_entry(k, v)?;
// // // 		}
// // // 		map.end();
// // // 	}
// // // }
