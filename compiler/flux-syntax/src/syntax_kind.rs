use flux_lexer::TokenKind;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum SyntaxKind {
	Root,
	BinExpr,
	PrefixExpr,
	ParenExpr,
	VarDecl,
	FnDecl,
	GenericList,
	FnParams,
	FnParam,
	ThisParam,
	TypeExpr,
	PrimitiveType,
	BlockExpr,
	ExprStmt,
	IdentExpr,
	IntExpr,
	FloatExpr,
	IfExpr,
	TypeDecl,
	StructType,
	StructTypeField,
	TraitDecl,
	TraitMethod,
	IdentType,
	CallExpr,
	ReturnStmt,
	ModDecl,
	UseDecl,
	PathExpr,
	StructExpr,
	StructExprField,
	TupleExpr,
	TupleType,
	ApplyDecl,
	ApplyBlock,
	WhereClause,
	TypeRestriction,
	TypeParams,
	PointerType,
	IntrinsicExpr,
	AddressExpr,
	IndexMemoryExpr,
	ForExpr,
	ApplyDeclTrait,
	EnumType,
	EnumTypeField,

	Whitespace,
	Comment,
	ModKw,
	UseKw,
	PubKw,
	FnKw,
	TypeKw,
	ApplyKw,
	ToKw,
	WhereKw,
	IsKw,
	MutKw,
	IfKw,
	ElseKw,
	StructKw,
	TraitKw,
	LetKw,
	ReturnKw,
	INKw,
	UNKw,
	F64Kw,
	F32Kw,
	BoolKw,
	Ident,

	Comma,
	CmpEq,
	CmpNeq,
	CmpLt,
	CmpGt,
	CmpLte,
	CmpGte,
	Colon,
	DoubleColon,
	Plus,
	Minus,
	Star,
	Slash,
	Arrow,
	FatArrow,
	LParen,
	RParen,
	LBrace,
	RBrace,
	Eq,
	SemiColon,
	Intrinsic,
	Ampersand,
	Period,
	LSquare,
	RSquare,
	ForKw,
	InKw,
	EnumKw,
	Error,
}

impl From<TokenKind> for SyntaxKind {
	fn from(token_kind: TokenKind) -> Self {
		match token_kind {
			TokenKind::Root => SyntaxKind::Root,
			TokenKind::Whitespace => SyntaxKind::Whitespace,
			TokenKind::Comment => SyntaxKind::Comment,
			TokenKind::Ident => SyntaxKind::Ident,
			TokenKind::IntLit => SyntaxKind::IntExpr,
			TokenKind::FloatLit => SyntaxKind::FloatExpr,
			TokenKind::ModKw => SyntaxKind::ModKw,
			TokenKind::UseKw => SyntaxKind::UseKw,
			TokenKind::PubKw => SyntaxKind::PubKw,
			TokenKind::FnKw => SyntaxKind::FnKw,
			TokenKind::TypeKw => SyntaxKind::TypeKw,
			TokenKind::ApplyKw => SyntaxKind::ApplyDecl,
			TokenKind::ToKw => SyntaxKind::ToKw,
			TokenKind::WhereKw => SyntaxKind::WhereKw,
			TokenKind::IsKw => SyntaxKind::IsKw,
			TokenKind::MutKw => SyntaxKind::MutKw,
			TokenKind::IfKw => SyntaxKind::IfKw,
			TokenKind::ElseKw => SyntaxKind::ElseKw,
			TokenKind::StructKw => SyntaxKind::StructKw,
			TokenKind::TraitKw => SyntaxKind::TraitKw,
			TokenKind::LetKw => SyntaxKind::LetKw,
			TokenKind::ReturnKw => SyntaxKind::ReturnKw,
			TokenKind::INKw => SyntaxKind::INKw,
			TokenKind::UNKw => SyntaxKind::UNKw,
			TokenKind::F64Kw => SyntaxKind::F64Kw,
			TokenKind::F32Kw => SyntaxKind::F32Kw,
			TokenKind::BoolKw => SyntaxKind::BoolKw,
			TokenKind::Comma => SyntaxKind::Comma,
			TokenKind::CmpEq => SyntaxKind::CmpEq,
			TokenKind::CmpNeq => SyntaxKind::CmpNeq,
			TokenKind::CmpLt => SyntaxKind::CmpLt,
			TokenKind::CmpGt => SyntaxKind::CmpGt,
			TokenKind::CmpLte => SyntaxKind::CmpLte,
			TokenKind::CmpGte => SyntaxKind::CmpGte,
			TokenKind::Colon => SyntaxKind::Colon,
			TokenKind::DoubleColon => SyntaxKind::DoubleColon,
			TokenKind::Plus => SyntaxKind::Plus,
			TokenKind::Minus => SyntaxKind::Minus,
			TokenKind::Star => SyntaxKind::Star,
			TokenKind::Slash => SyntaxKind::Slash,
			TokenKind::Arrow => SyntaxKind::Arrow,
			TokenKind::FatArrow => SyntaxKind::FatArrow,
			TokenKind::LParen => SyntaxKind::LParen,
			TokenKind::RParen => SyntaxKind::RParen,
			TokenKind::LBrace => SyntaxKind::RBrace,
			TokenKind::RBrace => SyntaxKind::RBrace,
			TokenKind::Eq => SyntaxKind::Eq,
			TokenKind::SemiColon => SyntaxKind::SemiColon,
			TokenKind::Intrinsic => SyntaxKind::Intrinsic,
			TokenKind::Ampersand => SyntaxKind::Ampersand,
			TokenKind::Period => SyntaxKind::Period,
			TokenKind::LSquare => SyntaxKind::LSquare,
			TokenKind::RSquare => SyntaxKind::RSquare,
			TokenKind::ForKw => SyntaxKind::ForKw,
			TokenKind::InKw => SyntaxKind::InKw,
			TokenKind::EnumKw => SyntaxKind::EnumKw,
			TokenKind::Error => SyntaxKind::Error,
		}
	}
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum FluxLanguage {}

impl cstree::Language for FluxLanguage {
	type Kind = SyntaxKind;

	fn kind_from_raw(raw: cstree::SyntaxKind) -> Self::Kind {
		unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
	}

	fn kind_to_raw(kind: Self::Kind) -> cstree::SyntaxKind {
		kind.into()
	}
}

impl From<SyntaxKind> for cstree::SyntaxKind {
	fn from(kind: SyntaxKind) -> Self {
		Self(kind as u16)
	}
}

pub type SyntaxNode = cstree::SyntaxNode<FluxLanguage>;
pub type SyntaxToken = cstree::SyntaxToken<FluxLanguage>;
pub type SyntaxElement = cstree::SyntaxElement<FluxLanguage>;
