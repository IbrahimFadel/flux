//! Generated file, do not edit by hand, see `pi-syntax/src/generate.rs`

use super::tokens::{IdentExpr, IntExpr};
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
pub struct Root {
	syntax: SyntaxNode,
}
impl AstNode for Root {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::Root => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl Root {
	pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
		self.syntax.children().filter_map(FnDecl::cast)
	}
}
#[derive(Debug)]
pub struct FnParams {
	syntax: SyntaxNode,
}
impl AstNode for FnParams {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::FnParams => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl FnParams {}
#[derive(Debug)]
pub struct ThisParam {
	syntax: SyntaxNode,
}
impl AstNode for ThisParam {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::ThisParam => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl ThisParam {}
#[derive(Debug)]
pub struct FnParam {
	syntax: SyntaxNode,
}
impl AstNode for FnParam {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::FnParam => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl FnParam {
	pub fn ty(&self) -> Option<Type> {
		self.syntax.children().find_map(Type::cast)
	}
	pub fn name(&self) -> Option<IdentExpr> {
		self.syntax.children().find_map(IdentExpr::cast)
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
			SyntaxKind::PrimitiveType => Self::PrimitiveType(PrimitiveType { syntax }),
			SyntaxKind::IdentExpr => Self::IdentExpr(IdentExpr { syntax }),
			_ => return None,
		};
		Some(result)
	}
	fn syntax(&self) -> &SyntaxNode {
		match self {
			Type::PrimitiveType(node) => &node.syntax,
			Type::IdentExpr(node) => &node.syntax,
		}
	}
}
#[derive(Debug)]
pub struct PrimitiveType {
	syntax: SyntaxNode,
}
impl AstNode for PrimitiveType {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::PrimitiveType => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl PrimitiveType {
	pub fn ty(&self) -> Option<SyntaxToken> {
		self
			.syntax
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == S!("int_number"))
	}
}
#[derive(Debug)]
pub struct BlockStmt {
	syntax: SyntaxNode,
}
impl AstNode for BlockStmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::BlockStmt => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl BlockStmt {
	pub fn stmts(&self) -> Vec<Stmt> {
		{
			self.syntax.children().filter_map(Stmt::cast).collect()
		}
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
			SyntaxKind::ExprStmt => Self::ExprStmt(ExprStmt { syntax }),
			SyntaxKind::VarDecl => Self::VarDecl(VarDecl { syntax }),
			_ => return None,
		};
		Some(result)
	}
	fn syntax(&self) -> &SyntaxNode {
		match self {
			Stmt::ExprStmt(node) => &node.syntax,
			Stmt::VarDecl(node) => &node.syntax,
		}
	}
}
#[derive(Debug)]
pub struct FnDecl {
	syntax: SyntaxNode,
}
impl AstNode for FnDecl {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::FnDecl => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl FnDecl {
	pub fn name(&self) -> Option<IdentExpr> {
		self.syntax.children().find_map(IdentExpr::cast)
	}
	pub fn params(&self) -> Option<FnParams> {
		self.syntax.children().find_map(FnParams::cast)
	}
	pub fn return_type(&self) -> Option<Type> {
		{
			self.syntax.children().find_map(Type::cast)
		}
	}
	pub fn body(&self) -> Option<BlockStmt> {
		self.syntax.children().find_map(BlockStmt::cast)
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
			SyntaxKind::IdentExpr => Self::IdentExpr(IdentExpr { syntax }),
			SyntaxKind::BinExpr => Self::BinExpr(BinExpr { syntax }),
			SyntaxKind::PrefixExpr => Self::PrefixExpr(PrefixExpr { syntax }),
			SyntaxKind::IntExpr => Self::IntExpr(IntExpr { syntax }),
			SyntaxKind::ParenExpr => Self::ParenExpr(ParenExpr { syntax }),
			_ => return None,
		};
		Some(result)
	}
	fn syntax(&self) -> &SyntaxNode {
		match self {
			Expr::IdentExpr(node) => &node.syntax,
			Expr::BinExpr(node) => &node.syntax,
			Expr::PrefixExpr(node) => &node.syntax,
			Expr::IntExpr(node) => &node.syntax,
			Expr::ParenExpr(node) => &node.syntax,
		}
	}
}
#[derive(Debug)]
pub struct BinExpr {
	syntax: SyntaxNode,
}
impl AstNode for BinExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::BinExpr => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl BinExpr {
	pub fn lhs(&self) -> Option<Expr> {
		self.syntax.children().find_map(Expr::cast)
	}
	pub fn op(&self) -> Option<SyntaxToken> {
		self
			.syntax
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| matches!(token.kind(), S!("+") | S!("-") | S!("*") | S!("/"),))
	}
	pub fn rhs(&self) -> Option<Expr> {
		self.syntax.children().find_map(Expr::cast)
	}
}
#[derive(Debug)]
pub struct PrefixExpr {
	syntax: SyntaxNode,
}
impl AstNode for PrefixExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::PrefixExpr => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl PrefixExpr {
	pub fn op(&self) -> Option<SyntaxToken> {
		self
			.syntax
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == S!("-"))
	}
	pub fn expr(&self) -> Option<Expr> {
		self.syntax.children().find_map(Expr::cast)
	}
}
#[derive(Debug)]
pub struct ParenExpr {
	syntax: SyntaxNode,
}
impl AstNode for ParenExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::ParenExpr => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl ParenExpr {
	pub fn expr(&self) -> Option<Expr> {
		self.syntax.children().find_map(Expr::cast)
	}
}
#[derive(Debug)]
pub struct ExprStmt {
	syntax: SyntaxNode,
}
impl AstNode for ExprStmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::ExprStmt => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl ExprStmt {
	pub fn expr(&self) -> Option<Expr> {
		self.syntax.children().find_map(Expr::cast)
	}
}
#[derive(Debug)]
pub struct VarDecl {
	syntax: SyntaxNode,
}
impl AstNode for VarDecl {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::VarDecl => Some(Self { syntax }),
			_ => None,
		}
	}
	fn syntax(&self) -> &SyntaxNode {
		&self.syntax
	}
}
impl VarDecl {
	pub fn ty(&self) -> Option<Type> {
		self.syntax.children().find_map(Type::cast)
	}
	pub fn name(&self) -> Option<IdentExpr> {
		self.syntax.children().find_map(IdentExpr::cast)
	}
	pub fn value(&self) -> Option<Expr> {
		self.syntax.children().find_map(Expr::cast)
	}
}
