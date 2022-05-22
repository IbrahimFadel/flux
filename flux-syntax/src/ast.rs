use std::ops::Deref;

use flux_error::Span;
use flux_lexer::TokenKind;
use rowan::TextRange;

use crate::syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};

pub trait AstNode {
	fn cast(syntax: SyntaxNode) -> Option<Self>
	where
		Self: Sized;
	fn syntax(&self) -> &SyntaxNode;

	fn range(&self) -> TextRange;
}

macro_rules! basic_node {
	($name:ident) => {
		#[derive(Debug)]
		pub struct $name(SyntaxNode);

		impl AstNode for $name {
			fn cast(syntax: SyntaxNode) -> Option<Self> {
				match syntax.kind() {
					SyntaxKind::$name => Some(Self(syntax)),
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
	};
}

macro_rules! enum_node {
	($name:ident, $($x:ident),*) => {
			#[derive(Debug)]
			pub enum $name {
				$($x($x)),+
			}

			impl AstNode for $name {
				fn cast(syntax: SyntaxNode) -> Option<Self> {
					let result = match syntax.kind() {
						$(SyntaxKind::$x => Self::$x($x(syntax))),+,
						_ => return None,
					};

					Some(result)
				}

				fn syntax(&self) -> &SyntaxNode {
					match self {
						$($name::$x(node) => &node.0),+
					}
				}

				fn range(&self) -> TextRange {
					self.syntax().text_range()
				}
			}
	};
}

enum_node!(Expr, BinExpr, IntExpr, FloatExpr, ParenExpr, PrefixExpr, IdentExpr, CallExpr);
enum_node!(Stmt, ExprStmt, VarDecl, BlockStmt, IfStmt, ReturnStmt);
enum_node!(Type, PrimitiveType, StructType, InterfaceType, IdentType);

basic_node!(Root);

basic_node!(ModDecl);
basic_node!(UseDecl);
basic_node!(TypeDecl);
basic_node!(FnDecl);
basic_node!(FnParam);
basic_node!(VarDecl);

basic_node!(ReturnStmt);
basic_node!(IfStmt);
basic_node!(ExprStmt);
basic_node!(BlockStmt);

basic_node!(IdentExpr);
basic_node!(ParenExpr);
basic_node!(PrefixExpr);
basic_node!(FloatExpr);
basic_node!(IntExpr);
basic_node!(BinExpr);
basic_node!(CallExpr);

basic_node!(StructField);
basic_node!(InterfaceMethod);

basic_node!(StructType);
basic_node!(InterfaceType);
basic_node!(IdentType);

impl Root {
	pub fn mods(&self) -> impl Iterator<Item = ModDecl> {
		self.0.children().filter_map(ModDecl::cast)
	}

	pub fn uses(&self) -> impl Iterator<Item = UseDecl> {
		self.0.children().filter_map(UseDecl::cast)
	}

	pub fn functions(&self) -> impl Iterator<Item = FnDecl> {
		self.0.children().filter_map(FnDecl::cast)
	}

	pub fn types(&self) -> impl Iterator<Item = TypeDecl> {
		self.0.children().filter_map(TypeDecl::cast)
	}
}

#[derive(Debug, Clone, PartialEq)]
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

impl ModDecl {
	pub fn name(&self) -> Option<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}
}

impl UseDecl {
	pub fn path(&self) -> Vec<SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElement::into_token)
			.filter_map(|token| {
				if token.kind() == SyntaxKind::Ident {
					Some(token)
				} else {
					None
				}
			})
			.collect()
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

impl FnDecl {
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

	pub fn return_type(&self) -> Option<Type> {
		self.0.children().find_map(Type::cast)
	}

	pub fn body(&self) -> Option<BlockStmt> {
		self.0.children().find_map(BlockStmt::cast)
	}
}

impl ReturnStmt {
	pub fn value(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}
}

impl ExprStmt {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
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

impl BlockStmt {
	pub fn stmts(&self) -> Vec<Stmt> {
		self.0.children().filter_map(Stmt::cast).collect()
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

impl CallExpr {
	pub fn callee(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
	}

	pub fn args(&self) -> impl Iterator<Item = Expr> {
		self.0.children().filter_map(Expr::cast).skip(1)
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

impl InterfaceType {
	pub fn methods(&self) -> Vec<InterfaceMethod> {
		self
			.0
			.children()
			.filter_map(InterfaceMethod::cast)
			.collect()
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

impl StructType {
	pub fn fields(&self) -> Vec<StructField> {
		self.0.children().filter_map(StructField::cast).collect()
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

impl IntExpr {
	pub fn tok(&self) -> Option<SyntaxToken> {
		self.0.first_token()
	}
}

impl FloatExpr {
	pub fn tok(&self) -> Option<SyntaxToken> {
		self.0.first_token()
	}
}

impl ParenExpr {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().find_map(Expr::cast)
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

impl IdentExpr {
	pub fn name(&self) -> Option<SyntaxToken> {
		self.0.first_token()
	}
}
