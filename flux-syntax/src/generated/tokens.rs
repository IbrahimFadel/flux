// //! Generated file, do not edit by hand, see `pi-syntax/src/generate.rs`

// use super::ast::AstNode;
// use crate::{
// 	syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken},
// 	S,
// };
// #[derive(Debug)]
// pub struct Ident {
// 	pub(crate) syntax: SyntaxNode,
// }
// impl AstNode for Ident {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		match syntax.kind() {
// 			SyntaxKind::Ident => Some(Self { syntax }),
// 			_ => None,
// 		}
// 	}
// 	fn syntax(&self) -> &SyntaxNode {
// 		&self.syntax
// 	}
// }
// impl Ident {
// 	pub fn text(&self) -> Option<SyntaxToken> {
// 		self
// 			.syntax
// 			.children_with_tokens()
// 			.filter_map(SyntaxElement::into_token)
// 			.find(|token| token.kind() == S!("ident"))
// 	}
// }
// #[derive(Debug)]
// pub struct IdentExpr {
// 	pub(crate) syntax: SyntaxNode,
// }
// impl AstNode for IdentExpr {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		match syntax.kind() {
// 			SyntaxKind::IdentExpr => Some(Self { syntax }),
// 			_ => None,
// 		}
// 	}
// 	fn syntax(&self) -> &SyntaxNode {
// 		&self.syntax
// 	}
// }
// impl IdentExpr {
// 	pub fn text(&self) -> Option<SyntaxToken> {
// 		self
// 			.syntax
// 			.children_with_tokens()
// 			.filter_map(SyntaxElement::into_token)
// 			.find(|token| token.kind() == S!("ident"))
// 	}
// }
// #[derive(Debug)]
// pub struct IntExpr {
// 	pub(crate) syntax: SyntaxNode,
// }
// impl AstNode for IntExpr {
// 	fn cast(syntax: SyntaxNode) -> Option<Self> {
// 		match syntax.kind() {
// 			SyntaxKind::IntExpr => Some(Self { syntax }),
// 			_ => None,
// 		}
// 	}
// 	fn syntax(&self) -> &SyntaxNode {
// 		&self.syntax
// 	}
// }
// impl IntExpr {
// 	pub fn int(&self) -> Option<SyntaxToken> {
// 		self
// 			.syntax
// 			.children_with_tokens()
// 			.filter_map(SyntaxElement::into_token)
// 			.find(|token| token.kind() == S!("int_number"))
// 	}
// }
