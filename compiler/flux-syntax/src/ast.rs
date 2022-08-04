use cstree::{SyntaxElementRef, TextRange};

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
			fn cast(node: SyntaxNode) -> Option<Self> {
				match node.kind() {
					SyntaxKind::$name => Some(Self(node)),
					_ => None,
				}
			}

			fn syntax(&self) -> &SyntaxNode {
				&self.0
			}

			fn range(&self) -> TextRange {
				trim_trailing_whitesapce(&self.syntax())
			}
		}
	};
}

fn trim_trailing_whitesapce(node: &SyntaxNode) -> TextRange {
	let end = if let Some(last_child) = node.last_child_or_token() {
		match last_child.as_node() {
			Some(node) => trim_trailing_whitesapce(node).end(),
			None => {
				let tok = last_child.as_token().unwrap();
				if tok.kind() == SyntaxKind::Whitespace {
					tok.text_range().start()
				} else {
					tok.text_range().end()
				}
			}
		}
	} else {
		return node.text_range();
	};
	TextRange::new(node.text_range().start(), end)
}

macro_rules! enum_node {
	($name:ident, $($x:ident),*) => {
			#[derive(Debug)]
			pub enum $name {
				$($x($x)),+
			}

			impl AstNode for $name {
				fn cast(syntax:SyntaxNode) -> Option<Self> {
					match syntax.kind() {
						$(SyntaxKind::$x => Some(Self::$x($x(syntax.clone())))),+,
						_ => return None,
					}
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

enum_node!(
	Expr,
	BinExpr,
	IntExpr,
	FloatExpr,
	ParenExpr,
	PrefixExpr,
	IdentExpr,
	CallExpr,
	PathExpr,
	StructExpr,
	IfExpr,
	BlockExpr,
	TupleExpr,
	IntrinsicExpr,
	AddressExpr,
	IndexMemoryExpr,
	ForExpr
);
enum_node!(Stmt, ExprStmt, VarDecl, ReturnStmt);
enum_node!(
	Type,
	PrimitiveType,
	StructType,
	IdentType,
	TupleType,
	PointerType,
	EnumType
);

basic_node!(Root);

basic_node!(ModDecl);
basic_node!(UseDecl);
basic_node!(TypeDecl);
basic_node!(FnDecl);
basic_node!(FnParam);
basic_node!(VarDecl);
basic_node!(ApplyDecl);
basic_node!(ApplyBlock);

basic_node!(ReturnStmt);
basic_node!(ExprStmt);

basic_node!(IdentExpr);
basic_node!(ParenExpr);
basic_node!(PrefixExpr);
basic_node!(FloatExpr);
basic_node!(IntExpr);
basic_node!(BinExpr);
basic_node!(CallExpr);
basic_node!(PathExpr);
basic_node!(StructExpr);
basic_node!(StructExprField);
basic_node!(IfExpr);
basic_node!(BlockExpr);
basic_node!(TupleExpr);
basic_node!(IntrinsicExpr);
basic_node!(AddressExpr);
basic_node!(IndexMemoryExpr);
basic_node!(ForExpr);

basic_node!(StructTypeField);
basic_node!(TraitMethod);
basic_node!(EnumTypeField);

basic_node!(StructType);
basic_node!(TraitDecl);
basic_node!(IdentType);
basic_node!(TupleType);
basic_node!(PointerType);
basic_node!(EnumType);

basic_node!(WhereClause);
basic_node!(TypeRestriction);
basic_node!(TypeParams);
basic_node!(GenericList);
basic_node!(ApplyDeclTrait);

impl Root {
	pub fn mods(&self) -> impl Iterator<Item = ModDecl> + '_ {
		self.0.children().cloned().filter_map(ModDecl::cast)
	}

	pub fn uses(&self) -> impl Iterator<Item = UseDecl> + '_ {
		self.0.children().cloned().filter_map(UseDecl::cast)
	}

	pub fn functions(&self) -> impl Iterator<Item = FnDecl> + '_ {
		self.0.children().cloned().filter_map(FnDecl::cast)
	}

	pub fn types(&self) -> impl Iterator<Item = TypeDecl> + '_ {
		self.0.children().cloned().filter_map(TypeDecl::cast)
	}

	pub fn applies(&self) -> impl Iterator<Item = ApplyDecl> + '_ {
		self.0.children().cloned().filter_map(ApplyDecl::cast)
	}

	pub fn traits(&self) -> impl Iterator<Item = TraitDecl> + '_ {
		self.0.children().cloned().filter_map(TraitDecl::cast)
	}
}

impl ModDecl {
	pub fn first_token(&self) -> Option<&SyntaxToken> {
		self.0.first_token()
	}

	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}
}

impl UseDecl {
	pub fn first_token(&self) -> Option<&SyntaxToken> {
		self.0.first_token()
	}

	pub fn path(&self) -> Option<PathExpr> {
		self.0.children().cloned().find_map(PathExpr::cast)
	}
}

impl TypeDecl {
	pub fn first_token(&self) -> Option<&SyntaxToken> {
		self.0.first_token()
	}

	pub fn public(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::PubKw)
	}

	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn generics(&self) -> Option<GenericList> {
		self.0.children().cloned().find_map(GenericList::cast)
	}

	pub fn ty(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}

	pub fn where_clause(&self) -> Option<WhereClause> {
		self.0.children().cloned().find_map(WhereClause::cast)
	}
}

impl FnDecl {
	pub fn first_token(&self) -> Option<&SyntaxToken> {
		self.0.first_token()
	}

	pub fn public(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::PubKw)
	}

	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn lparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::LParen)
	}

	pub fn params(&self) -> impl Iterator<Item = FnParam> + '_ {
		self.0.children().cloned().filter_map(FnParam::cast)
	}

	pub fn rparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::RParen)
	}

	pub fn return_type(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}

	pub fn body(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}
}

impl ApplyDecl {
	pub fn generics(&self) -> Option<GenericList> {
		self.0.children().cloned().find_map(GenericList::cast)
	}

	pub fn where_clause(&self) -> Option<WhereClause> {
		self.0.children().cloned().find_map(WhereClause::cast)
	}

	pub fn trait_(&self) -> Option<ApplyDeclTrait> {
		self.0.children().cloned().find_map(ApplyDeclTrait::cast)
	}

	pub fn ty(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}

	pub fn block(&self) -> Option<ApplyBlock> {
		self.0.children().cloned().find_map(ApplyBlock::cast)
	}
}

impl ApplyBlock {
	pub fn methods(&self) -> impl Iterator<Item = FnDecl> + '_ {
		self.0.children().cloned().filter_map(FnDecl::cast)
	}
}

impl ReturnStmt {
	pub fn value(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}
}

impl ExprStmt {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}

	pub fn semicolon(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::SemiColon)
	}
}

impl VarDecl {
	pub fn ty(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}

	pub fn let_tok(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::LetKw)
	}

	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn equal_tok(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Eq)
	}

	pub fn value(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}
}

impl BlockExpr {
	pub fn stmts(&self) -> impl Iterator<Item = Stmt> + '_ {
		self.0.children().cloned().filter_map(Stmt::cast)
	}
}

impl TupleExpr {
	pub fn values(&self) -> impl Iterator<Item = Expr> + '_ {
		self.0.children().cloned().filter_map(Expr::cast)
	}
}

impl IfExpr {
	pub fn condition(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}

	pub fn then(&self) -> Option<BlockExpr> {
		self.0.children().cloned().find_map(BlockExpr::cast)
	}

	pub fn else_(&self) -> Option<Expr> {
		self.0.children().cloned().filter_map(Expr::cast).nth(2)
	}
}

impl CallExpr {
	pub fn callee(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}

	pub fn lparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::LParen)
	}

	pub fn args(&self) -> impl Iterator<Item = Expr> + '_ {
		self.0.children().cloned().filter_map(Expr::cast).skip(1)
	}

	pub fn rparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::RParen)
	}
}

impl IdentType {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn type_params(&self) -> Option<TypeParams> {
		self.0.children().cloned().find_map(TypeParams::cast)
	}
}

impl TraitDecl {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn generics(&self) -> Option<GenericList> {
		self.0.children().cloned().find_map(GenericList::cast)
	}

	pub fn where_clause(&self) -> Option<WhereClause> {
		self.0.children().cloned().find_map(WhereClause::cast)
	}

	pub fn methods(&self) -> impl Iterator<Item = TraitMethod> + '_ {
		self.0.children().cloned().filter_map(TraitMethod::cast)
	}
}

impl TraitMethod {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn lparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::LParen)
	}

	pub fn params(&self) -> impl Iterator<Item = FnParam> + '_ {
		self.0.children().cloned().filter_map(FnParam::cast)
	}

	pub fn rparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::RParen)
	}

	pub fn return_ty(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}
}

impl FnParam {
	pub fn mutable(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::MutKw)
	}

	pub fn ty(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}

	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}
}

impl StructType {
	pub fn fields(&self) -> impl Iterator<Item = StructTypeField> + '_ {
		self.0.children().cloned().filter_map(StructTypeField::cast)
	}
}

impl StructTypeField {
	pub fn public(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::PubKw)
	}

	pub fn mutable(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::MutKw)
	}

	pub fn type_(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}

	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
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
	pub fn ty(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
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
		self.0.children().cloned().find_map(Expr::cast)
	}

	pub fn rhs(&self) -> Option<Expr> {
		self.0.children().cloned().filter_map(Expr::cast).nth(1)
	}

	pub fn op(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| {
				matches!(
					token.kind(),
					SyntaxKind::Plus
						| SyntaxKind::Minus
						| SyntaxKind::Star
						| SyntaxKind::Slash
						| SyntaxKind::CmpEq
						| SyntaxKind::DoubleColon
						| SyntaxKind::Period
						| SyntaxKind::CmpNeq
						| SyntaxKind::CmpGt
						| SyntaxKind::Eq,
				)
			})
	}
}

impl IntExpr {
	pub fn tok(&self) -> Option<&SyntaxToken> {
		self.0.first_token()
	}
}

impl FloatExpr {
	pub fn tok(&self) -> Option<&SyntaxToken> {
		self.0.first_token()
	}
}

impl ParenExpr {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}
}

impl PrefixExpr {
	pub fn op(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Minus)
	}

	pub fn expr(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}
}

impl IdentExpr {
	pub fn path(&self) -> Option<PathExpr> {
		self.0.children().cloned().find_map(PathExpr::cast)
	}
}

impl PathExpr {
	pub fn names(&self) -> impl Iterator<Item = &SyntaxToken> + '_ {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.filter_map(|token| {
				if token.kind() == SyntaxKind::Ident {
					Some(token)
				} else {
					None
				}
			})
	}
}

impl StructExpr {
	pub fn name(&self) -> Option<PathExpr> {
		self.0.children().cloned().find_map(PathExpr::cast)
	}

	pub fn lparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::LParen)
	}

	pub fn fields(&self) -> impl Iterator<Item = StructExprField> + '_ {
		self.0.children().cloned().filter_map(StructExprField::cast)
	}

	pub fn rparen(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::RParen)
	}
}

impl StructExprField {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn value(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}
}

impl TupleType {
	pub fn types(&self) -> impl Iterator<Item = Type> + '_ {
		self.0.children().cloned().filter_map(Type::cast)
	}
}

impl PointerType {
	pub fn to(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}
}

impl WhereClause {
	pub fn type_restrictions(&self) -> impl Iterator<Item = TypeRestriction> + '_ {
		self.0.children().cloned().filter_map(TypeRestriction::cast)
	}
}

impl TypeRestriction {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn trait_(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.filter(|token| token.kind() == SyntaxKind::Ident)
			.nth(1)
	}

	pub fn trait_type_params(&self) -> Option<TypeParams> {
		self.0.children().cloned().find_map(TypeParams::cast)
	}
}

impl TypeParams {
	pub fn params(&self) -> impl Iterator<Item = Type> + '_ {
		self.0.children().cloned().filter_map(Type::cast)
	}
}

impl GenericList {
	pub fn names(&self) -> impl Iterator<Item = &SyntaxToken> + '_ {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.filter(|token| token.kind() == SyntaxKind::Ident)
	}
}

impl IntrinsicExpr {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Intrinsic)
	}
}

impl AddressExpr {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}
}

impl IndexMemoryExpr {
	pub fn expr(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}

	pub fn idx(&self) -> Option<Expr> {
		self.0.children().cloned().filter_map(Expr::cast).nth(1)
	}
}

impl ForExpr {
	pub fn item(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn iterator(&self) -> Option<Expr> {
		self.0.children().cloned().find_map(Expr::cast)
	}

	pub fn block(&self) -> Option<BlockExpr> {
		self.0.children().cloned().find_map(BlockExpr::cast)
	}
}

impl ApplyDeclTrait {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn type_params(&self) -> Option<TypeParams> {
		self.0.children().cloned().find_map(TypeParams::cast)
	}
}

impl EnumType {
	pub fn fields(&self) -> impl Iterator<Item = EnumTypeField> + '_ {
		self.0.children().cloned().filter_map(EnumTypeField::cast)
	}
}

impl EnumTypeField {
	pub fn name(&self) -> Option<&SyntaxToken> {
		self
			.0
			.children_with_tokens()
			.filter_map(SyntaxElementRef::into_token)
			.find(|token| token.kind() == SyntaxKind::Ident)
	}

	pub fn ty(&self) -> Option<Type> {
		self.0.children().cloned().find_map(Type::cast)
	}
}
