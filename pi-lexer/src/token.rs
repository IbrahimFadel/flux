use std::ops::Range;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
	pub kind: TokenKind,
	pub span: Range<usize>,
}

impl Token {
	pub fn new() -> Token {
		Token {
			kind: TokenKind::Illegal,
			span: 0usize..0usize,
		}
	}

	pub fn from(kind: TokenKind, span: Range<usize>) -> Token {
		Token { kind, span }
	}
}

#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum TokenKind {
	LineComment,
	BlockComment,
	Whitespace,
	Illegal,
	EOF,

	Semicolon,
	DoubleColon,
	LParen,
	RParen,
	LBrace,
	RBrace,
	LBracket,
	RBracket,

	Plus,
	Minus,
	Asterisk,
	Ampersand,
	Slash,
	Comma,
	Period,
	Eq,
	Arrow,

	CmpNE,
	CmpEQ,
	CmpOr,
	CmpAnd,
	CmpLT,
	CmpLTE,
	CmpGT,
	CmpGTE,

	Pub,
	Fn,
	Return,
	Mut,
	Type,
	Interface,
	Struct,
	Nil,
	If,
	Else,
	Mod,
	Apply,
	To,

	TypesBegin,

	I64,
	U64,
	I32,
	U32,
	I16,
	U16,
	I8,
	U8,
	F64,
	F32,
	Bool,

	TypesEnd,

	BasicLitBegin,

	Int,
	Float,
	StringLit,
	CharLit,

	BasicLitEnd,

	Ident,
}

impl std::fmt::Display for TokenKind {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self)
	}
}

pub fn lookup_keyword(v: &str) -> TokenKind {
	match v {
		"pub" => TokenKind::Pub,
		"fn" => TokenKind::Fn,
		"return" => TokenKind::Return,
		"mut" => TokenKind::Mut,
		"type" => TokenKind::Type,
		"interface" => TokenKind::Interface,
		"struct" => TokenKind::Struct,
		"nil" => TokenKind::Nil,
		"if" => TokenKind::If,
		"else" => TokenKind::Else,
		"mod" => TokenKind::Mod,
		"apply" => TokenKind::Apply,
		"to" => TokenKind::To,

		"i64" => TokenKind::I64,
		"u64" => TokenKind::U64,
		"i32" => TokenKind::I32,
		"u32" => TokenKind::U32,
		"i16" => TokenKind::I16,
		"u16" => TokenKind::U16,
		"i8" => TokenKind::I8,
		"u8" => TokenKind::U8,
		"f64" => TokenKind::F64,
		"f32" => TokenKind::F32,
		"bool" => TokenKind::Bool,

		_ => TokenKind::Ident,
	}
}
