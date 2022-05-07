//! Generated file, do not edit by hand, see `pi-syntax/src/generate.rs`

use super::tokens::{Ident, IdentExpr, IntExpr};
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
pub fn get_lparen(&self) -> Option<SyntaxToken> {
	self
		.syntax
		.children_with_tokens()
		.filter_map(SyntaxElement::into_token)
		.find(|token| token.kind() == S!("("))
}
pub fn get_ty(&self) -> Option<SyntaxToken> {
	self
		.syntax
		.children_with_tokens()
		.filter_map(SyntaxElement::into_token)
		.find(|token| token.kind() == S!("int_number"))
}
pub fn get_op(&self) -> Option<SyntaxToken> {
	self
		.syntax
		.children_with_tokens()
		.filter_map(SyntaxElement::into_token)
		.find(|token| token.kind() == S!("-"))
}
