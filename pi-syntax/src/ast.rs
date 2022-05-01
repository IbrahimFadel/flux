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

	pub fn block(&self) -> Option<BlockStmt> {
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
}

#[derive(Debug)]
pub enum Stmt {
	ExprStmt(ExprStmt),
	VarDecl(VarDecl),
	BlockStmt(BlockStmt),
}

impl AstNode for Stmt {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
			SyntaxKind::ExprStmt => Self::ExprStmt(ExprStmt(syntax)),
			SyntaxKind::VarDecl => Self::VarDecl(VarDecl(syntax)),
			_ => return None,
		};

		Some(result)
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Stmt::ExprStmt(node) => &node.0,
			Stmt::VarDecl(node) => &node.0,
			Stmt::BlockStmt(node) => &node.0,
		}
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
}

impl BlockStmt {
	pub fn stmts(&self) -> Vec<Stmt> {
		self.0.children().filter_map(Stmt::cast).collect()
	}
}

#[derive(Debug)]
pub enum Expr {
	BinExpr(BinExpr),
	IntExpr(IntExpr),
	ParenExpr(ParenExpr),
	PrefixExpr(PrefixExpr),
	IdentExpr(IdentExpr),
}

impl AstNode for Expr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		let result = match syntax.kind() {
			SyntaxKind::BinExpr => Self::BinExpr(BinExpr(syntax)),
			SyntaxKind::IntLit => Self::IntExpr(IntExpr(syntax)),
			SyntaxKind::ParenExpr => Self::ParenExpr(ParenExpr(syntax)),
			SyntaxKind::PrefixExpr => Self::PrefixExpr(PrefixExpr(syntax)),
			SyntaxKind::Ident => Self::IdentExpr(IdentExpr(syntax)),
			_ => return None,
		};

		Some(result)
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Expr::BinExpr(node) => &node.0,
			Expr::IntExpr(node) => &node.0,
			Expr::ParenExpr(node) => &node.0,
			Expr::PrefixExpr(node) => &node.0,
			Expr::IdentExpr(node) => &node.0,
		}
	}
}

#[derive(Debug)]
pub enum Type {
	PrimitiveType(PrimitiveType),
}

impl AstNode for Type {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::PrimitiveType => Some(Self::PrimitiveType(PrimitiveType(syntax))),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		match self {
			Type::PrimitiveType(node) => &node.0,
		}
	}
}

#[derive(Debug)]
pub struct PrimitiveType(SyntaxNode);

impl AstNode for PrimitiveType {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::INKw => Some(Self(syntax)),
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
			.find(|token| token.kind() == SyntaxKind::INKw)
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
					SyntaxKind::Plus | SyntaxKind::Minus | SyntaxKind::Star | SyntaxKind::Slash,
				)
			})
	}
}

#[derive(Debug)]
pub struct IntExpr(SyntaxNode);

impl AstNode for IntExpr {
	fn cast(syntax: SyntaxNode) -> Option<Self> {
		match syntax.kind() {
			SyntaxKind::IntLit => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

impl IntExpr {
	pub fn parse(&self) -> u64 {
		self.0.first_token().unwrap().text().parse().unwrap()
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
			SyntaxKind::Ident => Some(Self(syntax)),
			_ => None,
		}
	}

	fn syntax(&self) -> &SyntaxNode {
		&self.0
	}
}

impl IdentExpr {
	pub fn name(&self) -> Option<SyntaxToken> {
		self.0.first_token()
	}
}

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

use crate::syntax_kind::{SyntaxElement, SyntaxKind, SyntaxNode, SyntaxToken};
