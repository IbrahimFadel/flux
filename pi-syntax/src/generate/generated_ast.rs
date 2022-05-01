// automatically generated. do not edit manually

use crate::{
	syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
	S,
};

pub trait AstNode {
	fn cast(syntax: SyntaxNode) -> Option<Self>
	where
		Self: Sized;
	fn syntax(&self) -> &SyntaxNode;
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
}

impl Root {
	pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
		self.0.children().filter_map(FnDecl::cast)
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
}

impl FnParams {
}

#[derive(Debug)]
pub struct ThisParam(SyntaxNode);

impl AstNode for ThisParam {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::ThisParam => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

impl ThisParam {
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
}

impl FnParam {
	pub fn ty(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}
	pub fn name(&self) -> Option<IdentExpr> {
		self.0.children().find_map(IdentExpr::cast)
	}
}

#[derive(Debug)]
pub enum Type {
	PrimitiveType(PrimitiveType),
	IdentExpr(IdentExpr),
}

impl AstNode for Type {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
			SyntaxKind::PrimitiveType => Self::PrimitiveType(PrimitiveType(syntax)),
			SyntaxKind::IdentExpr => Self::IdentExpr(IdentExpr(syntax)),
			_ => return None,
		};
		return Some(result);
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Type::PrimitiveType(node) => &node.0,
			Type::IdentExpr(node) => &node.0,
		}
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
}

impl IdentExpr {
	pub fn text(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == S!(ident))
	}
}

#[derive(Debug)]
pub struct PrimitiveType(SyntaxNode);

impl AstNode for PrimitiveType {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::PrimitiveType => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

impl PrimitiveType {
	pub fn ty(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == S!(int_number))
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
}

impl BlockStmt {
	pub fn stmts(&self) -> Vec<Stmt> {
		self.0.children().filter_map(Stmt::cast).collect()
	}
}

#[derive(Debug)]
pub enum Stmt {
	ExprStmt(ExprStmt),
	VarDecl(VarDecl),
}

impl AstNode for Stmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
			SyntaxKind::ExprStmt => Self::ExprStmt(ExprStmt(syntax)),
			SyntaxKind::VarDecl => Self::VarDecl(VarDecl(syntax)),
			_ => return None,
		};
		return Some(result);
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Stmt::ExprStmt(node) => &node.0,
			Stmt::VarDecl(node) => &node.0,
		}
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
}

impl FnDecl {
	pub fn name(&self) -> Option<IdentExpr> {
		self.0.children().find_map(IdentExpr::cast)
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
pub enum Expr {
	IdentExpr(IdentExpr),
	BinExpr(BinExpr),
	PrefixExpr(PrefixExpr),
	IntExpr(IntExpr),
	ParenExpr(ParenExpr),
}

impl AstNode for Expr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
			SyntaxKind::IdentExpr => Self::IdentExpr(IdentExpr(syntax)),
			SyntaxKind::BinExpr => Self::BinExpr(BinExpr(syntax)),
			SyntaxKind::PrefixExpr => Self::PrefixExpr(PrefixExpr(syntax)),
			SyntaxKind::IntExpr => Self::IntExpr(IntExpr(syntax)),
			SyntaxKind::ParenExpr => Self::ParenExpr(ParenExpr(syntax)),
			_ => return None,
		};
		return Some(result);
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Expr::IdentExpr(node) => &node.0,
			Expr::BinExpr(node) => &node.0,
			Expr::PrefixExpr(node) => &node.0,
			Expr::IntExpr(node) => &node.0,
			Expr::ParenExpr(node) => &node.0,
		}
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
}

impl BinExpr {
	pub fn lhs(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}
	pub fn op(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| {
				matches!(
					token.kind(),
					S!(+) | S!(-) | S!(*) | S!(/),
				)
			})
	}
	pub fn rhs(&self) -> Option<Expr> {
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
}

impl PrefixExpr {
	pub fn op(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == S!(-))
	}
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
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
}

impl IntExpr {
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
}

impl ParenExpr {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
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
}

impl ExprStmt {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
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
}

impl VarDecl {
	pub fn ty(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}
	pub fn name(&self) -> Option<IdentExpr> {
		self.0.children().find_map(IdentExpr::cast)
	}
	pub fn value(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}
}

