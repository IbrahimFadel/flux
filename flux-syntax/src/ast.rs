use rowan::TextRange;

use crate::syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

pub trait AstNode {
	fn cast(syntax: SyntaxNode) -> Option<Self>
	where
		Self: Sized;
	fn syntax(&self) -> &SyntaxNode;

	fn range(&self) -> TextRange;
}

#[derive(Debug)]
pub struct Root(SyntaxNode);

impl AstNode for Root {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::Root => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl Root {
	pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
		self.0.children().filter_map(FnDecl::cast)
	}

	pub fn types(&self) -> impl Iterator<Item = TypeDecl> {
		self.0.children().filter_map(TypeDecl::cast)
	}
}

#[derive(Debug)]
pub struct TypeDecl(SyntaxNode);

impl AstNode for TypeDecl {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::TypeDecl => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl TypeDecl {
	pub fn public(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::PubKw)
	}

	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn ty(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}
}

#[derive(Debug)]
pub struct FnDecl(SyntaxNode);

impl AstNode for FnDecl {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::FnDecl => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl FnDecl {
	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn params(&self) -> Option<FnParams> {
		self.0.children().find_map(FnParams::cast)
	}

	pub fn return_type(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}

	pub fn body(&self) -> Option<BlockStmt> {
		self.0.children().find_map(BlockStmt::cast)
	}
}

#[derive(Debug)]
pub struct FnParams(SyntaxNode);

impl AstNode for FnParams {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::FnParams => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

#[derive(Debug)]
pub enum Stmt {
	ExprStmt(ExprStmt),
	VarDecl(VarDecl),
	BlockStmt(BlockStmt),
	IfStmt(IfStmt),
}

impl AstNode for Stmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
			SyntaxKind::ExprStmt => Self::ExprStmt(ExprStmt(syntax)),
			SyntaxKind::VarDecl => Self::VarDecl(VarDecl(syntax)),
			SyntaxKind::BlockStmt => Self::BlockStmt(BlockStmt(syntax)),
			SyntaxKind::IfStmt => Self::IfStmt(IfStmt(syntax)),
			_ => return None,
		};

		Some(result)
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Stmt::ExprStmt(node) => &node.0,
			Stmt::VarDecl(node) => &node.0,
			Stmt::BlockStmt(node) => &node.0,
			Stmt::IfStmt(node) => &node.0,
		}
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

#[derive(Debug)]
pub struct ExprStmt(SyntaxNode);

impl AstNode for ExprStmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::ExprStmt => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

#[derive(Debug)]
pub struct VarDecl(SyntaxNode);

impl AstNode for VarDecl {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::VarDecl => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl VarDecl {
	pub fn ty(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}

	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn value(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}
}

#[derive(Debug)]
pub struct BlockStmt(SyntaxNode);

impl AstNode for BlockStmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::BlockStmt => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl BlockStmt {
	pub fn stmts(&self) -> Vec<Stmt> {
		self.0.children().filter_map(Stmt::cast).collect()
	}
}

#[derive(Debug)]
pub struct IfStmt(SyntaxNode);

impl AstNode for IfStmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::IfStmt => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl IfStmt {
	pub fn condition(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}

	pub fn then(&self) -> Option<BlockStmt> {
		self.0.children().find_map(BlockStmt::cast)
	}

	pub fn else_ifs(&self) -> Vec<IfStmt> {
		self.0.children().filter_map(IfStmt::cast).collect()
	}

	pub fn else_(&self) -> Option<BlockStmt> {
		self.0.children().filter_map(BlockStmt::cast).nth(1)
	}
}

#[derive(Debug)]
pub enum Expr {
	BinExpr(BinExpr),
	IntExpr(IntExpr),
	FloatExpr(FloatExpr),
	ParenExpr(ParenExpr),
	PrefixExpr(PrefixExpr),
	IdentExpr(IdentExpr),
}

impl AstNode for Expr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
			SyntaxKind::BinExpr => Self::BinExpr(BinExpr(syntax)),
			SyntaxKind::IntExpr => Self::IntExpr(IntExpr(syntax)),
			SyntaxKind::FloatExpr => Self::FloatExpr(FloatExpr(syntax)),
			SyntaxKind::ParenExpr => Self::ParenExpr(ParenExpr(syntax)),
			SyntaxKind::PrefixExpr => Self::PrefixExpr(PrefixExpr(syntax)),
			SyntaxKind::IdentExpr => Self::IdentExpr(IdentExpr(syntax)),
			_ => return None,
		};

		Some(result)
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Expr::BinExpr(node) => &node.0,
			Expr::IntExpr(node) => &node.0,
			Expr::FloatExpr(node) => &node.0,
			Expr::ParenExpr(node) => &node.0,
			Expr::PrefixExpr(node) => &node.0,
			Expr::IdentExpr(node) => &node.0,
		}
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

#[derive(Debug)]
pub enum Type {
	PrimitiveType(PrimitiveType),
	StructType(StructType),
	InterfaceType(InterfaceType),
	IdentType(IdentType),
}

impl AstNode for Type {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::PrimitiveType => Some(Self::PrimitiveType(PrimitiveType(syntax))),
			SyntaxKind::StructType => Some(Self::StructType(StructType(syntax))),
			SyntaxKind::InterfaceType => Some(Self::InterfaceType(InterfaceType(syntax))),
			SyntaxKind::IdentType => Some(Self::IdentType(IdentType(syntax))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Type::PrimitiveType(node) => &node.0,
			Type::StructType(node) => &node.0,
			Type::InterfaceType(node) => &node.0,
			Type::IdentType(node) => &node.0,
		}
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

#[derive(Debug)]
pub struct IdentType(SyntaxNode);

impl AstNode for IdentType {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::IdentType => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl IdentType {
	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}
}

#[derive(Debug)]
pub struct InterfaceType(SyntaxNode);

impl AstNode for InterfaceType {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::InterfaceType => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl InterfaceType {
	pub fn methods(&self) -> Vec<InterfaceMethod> {
		self
			.0
			.children()
			.filter_map(InterfaceMethod::cast)
			.collect()
	}
}

#[derive(Debug)]
pub struct InterfaceMethod(SyntaxNode);

impl AstNode for InterfaceMethod {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::InterfaceMethod => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl InterfaceMethod {
	pub fn public(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::PubKw)
	}

	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn params(&self) -> Vec<FnParam> {
		self.0.children().filter_map(FnParam::cast).collect()
	}

	pub fn return_ty(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}
}

#[derive(Debug)]
pub struct FnParam(SyntaxNode);

impl AstNode for FnParam {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::FnParam => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl FnParam {
	pub fn mutable(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::MutKw)
	}

	pub fn ty(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}

	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}
}

#[derive(Debug)]
pub struct StructType(SyntaxNode);

impl AstNode for StructType {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::StructType => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl StructType {
	pub fn fields(&self) -> Vec<StructField> {
		self.0.children().filter_map(StructField::cast).collect()
	}
}

#[derive(Debug)]
pub struct StructField(SyntaxNode);

impl AstNode for StructField {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::StructTypeField => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl StructField {
	pub fn public(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::PubKw)
	}

	pub fn mutable(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::MutKw)
	}

	pub fn type_(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}

	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}
}

#[derive(Debug)]
pub struct PrimitiveType(SyntaxNode);

impl AstNode for PrimitiveType {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::INKw => Some(Self(syntax)),
			SyntaxKind::UNKw => Some(Self(syntax)),
			SyntaxKind::F64Kw => Some(Self(syntax)),
			SyntaxKind::F32Kw => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl PrimitiveType {
	pub fn ty(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| {
				matches!(
					token.kind(),
					SyntaxKind::INKw | SyntaxKind::UNKw | SyntaxKind::F64Kw | SyntaxKind::F32Kw
				)
			})
	}
}

#[derive(Debug)]
pub struct BinExpr(SyntaxNode);

impl AstNode for BinExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::BinExpr => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl BinExpr {
	pub fn lhs(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}

	pub fn rhs(&self) -> Option<Expr> {
		self.0.children().filter_map(Expr::cast).nth(1)
	}

	pub fn op(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| {
				matches!(
					token.kind(),
					SyntaxKind::Plus
						| SyntaxKind::Minus
						| SyntaxKind::Star
						| SyntaxKind::Slash
						| SyntaxKind::CmpEq,
				)
			})
	}
}

#[derive(Debug)]
pub struct IntExpr(SyntaxNode);

impl AstNode for IntExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::IntExpr => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl IntExpr {
	pub fn tok(&self) -> Option<SyntaxToken> {
		self.0.first_token()
	}
}

#[derive(Debug)]
pub struct FloatExpr(SyntaxNode);

impl AstNode for FloatExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::FloatExpr => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl FloatExpr {
	pub fn tok(&self) -> Option<SyntaxToken> {
		self.0.first_token()
	}
}

#[derive(Debug)]
pub struct ParenExpr(SyntaxNode);

impl AstNode for ParenExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::ParenExpr => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl ParenExpr {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}
}

#[derive(Debug)]
pub struct PrefixExpr(SyntaxNode);

impl AstNode for PrefixExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::PrefixExpr => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl PrefixExpr {
	pub fn op(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Minus)
	}

	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}
}

#[derive(Debug)]
pub struct IdentExpr(SyntaxNode);

impl AstNode for IdentExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::IdentExpr => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}

	fn range(&self) -> TextRange {
		self.syntax().text_range()
	}
}

impl IdentExpr {
	pub fn name(&self) -> Option<SyntaxToken> {
		self.0.first_token()
	}
}
