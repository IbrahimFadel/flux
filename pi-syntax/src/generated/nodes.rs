// //! Generated file, do not edit by hand, see `pi-syntax/src/generate.rs`

// use super::tokens::{Ident, IdentExpr, IntExpr};
// use crate::{
// 	syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
// 	S,
// };
// pub trait AstNode {
// 	fn cast(syntax: SyntaxNode) -> Option<Self>
// 	where
// 		Self: Sized;
// 	fn syntax(&self) -> &SyntaxNode;
// }
// #[derive(Debug)]
// pub struct Root {
// 	syntax: SyntaxNode,
// }
// impl AstNode for Root {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		match syntax.kind() {
// 			SyntaxKind::Root => Some(Self { syntax }),
// 			_ => None,
// 		}
// 	}
// 	fn syntax(&self) -> &SyntaxNode {
// 		&self.syntax
// 	}
// }
// impl Root {
// 	pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
// 		self.syntax.children().filter_map(FnDecl::cast)
// 	}
// }
// #[derive(Debug)]
// pub enum Type {
// 	PrimitiveType(PrimitiveType),
// 	Ident(Ident),
// }
// impl AstNode for Type {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		let result = match syntax.kind() {
// 			SyntaxKind::PrimitiveType => Self::PrimitiveType(PrimitiveType { syntax }),
// 			SyntaxKind::Ident => Self::Ident(Ident { syntax }),
// 			_ => return None,
// 		};
// 		Some(result)
// 	}
// 	fn syntax(&self) -> &SyntaxNode {
// 		match self {
// 			Type::PrimitiveType(node) => &node.syntax,
// 			Type::Ident(node) => &node.syntax,
// 		}
// 	}
// }
// #[derive(Debug)]
// pub enum Stmt {
// 	ExprStmt(ExprStmt),
// 	VarDecl(VarDecl),
// }
// impl AstNode for Stmt {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		let result = match syntax.kind() {
// 			SyntaxKind::ExprStmt => Self::ExprStmt(ExprStmt { syntax }),
// 			SyntaxKind::VarDecl => Self::VarDecl(VarDecl { syntax }),
// 			_ => return None,
// 		};
// 		Some(result)
// 	}
// 	fn syntax(&self) -> &SyntaxNode {
// 		match self {
// 			Stmt::ExprStmt(node) => &node.syntax,
// 			Stmt::VarDecl(node) => &node.syntax,
// 		}
// 	}
// }
// #[derive(Debug)]
// pub enum Expr {
// 	IdentExpr(IdentExpr),
// 	BinExpr(BinExpr),
// 	PrefixExpr(PrefixExpr),
// 	IntExpr(IntExpr),
// 	ParenExpr(ParenExpr),
// }
// impl AstNode for Expr {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		let result = match syntax.kind() {
// 			SyntaxKind::IdentExpr => Self::IdentExpr(IdentExpr { syntax }),
// 			SyntaxKind::BinExpr => Self::BinExpr(BinExpr { syntax }),
// 			SyntaxKind::PrefixExpr => Self::PrefixExpr(PrefixExpr { syntax }),
// 			SyntaxKind::IntExpr => Self::IntExpr(IntExpr { syntax }),
// 			SyntaxKind::ParenExpr => Self::ParenExpr(ParenExpr { syntax }),
// 			_ => return None,
// 		};
// 		Some(result)
// 	}
// 	fn syntax(&self) -> &SyntaxNode {
// 		match self {
// 			Expr::IdentExpr(node) => &node.syntax,
// 			Expr::BinExpr(node) => &node.syntax,
// 			Expr::PrefixExpr(node) => &node.syntax,
// 			Expr::IntExpr(node) => &node.syntax,
// 			Expr::ParenExpr(node) => &node.syntax,
// 		}
// 	}
// }
