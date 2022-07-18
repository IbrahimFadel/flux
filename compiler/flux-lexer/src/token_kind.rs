use std::fmt::Display;

use logos::Logos;
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(
	Debug, Copy, Clone, PartialEq, Logos, Hash, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive,
)]
pub enum TokenKind {
	Root,

	#[regex("[ \n\r\t]+")]
	Whitespace,

	#[regex("//.*")]
	#[regex(r"/\*([^*]|\**[^*/])*\*+/")]
	Comment,
	#[regex("[A-Za-z][A-Za-z0-9_]*")]
	Ident,
	#[regex("0x[0-9a-fA-F]+(_[0-9a-fA-F]+)*")]
	#[regex("0b[0-9]+(_[0-9]+)*")]
	#[regex("[0-9]+(_[0-9]+)*")]
	IntLit,

	#[regex(r"[0-9]+\.[0-9]+(_[0-9]+)*")]
	FloatLit,

	#[token("mod")]
	ModKw,
	#[token("use")]
	UseKw,
	#[token("pub")]
	PubKw,
	#[token("fn")]
	FnKw,
	#[token("type")]
	TypeKw,
	#[token("apply")]
	ApplyKw,
	#[token("to")]
	ToKw,
	#[token("where")]
	WhereKw,
	#[token("mut")]
	MutKw,
	#[token("if")]
	IfKw,
	#[token("else")]
	ElseKw,
	#[token("struct")]
	StructKw,
	#[token("trait")]
	TraitKw,
	#[regex("let")]
	LetKw,
	#[regex("return")]
	ReturnKw,
	#[regex(r"i[1-9]+")]
	INKw,
	#[regex(r"u[1-9]+")]
	UNKw,
	#[token("f64")]
	F64Kw,
	#[token("f32")]
	F32Kw,
	#[token("bool")]
	BoolKw,

	#[token(",")]
	Comma,
	#[token("+")]
	Plus,
	#[token("-")]
	Minus,
	#[token("*")]
	Star,
	#[token("/")]
	Slash,
	#[token("->")]
	Arrow,
	#[token("=>")]
	FatArrow,
	#[token(":")]
	Colon,
	#[token("(")]
	LParen,
	#[token(")")]
	RParen,
	#[token("{")]
	LBrace,
	#[token("}")]
	RBrace,
	#[token("=")]
	Eq,
	#[token("==")]
	CmpEq,
	#[token("!=")]
	CmpNeq,
	#[token("<")]
	CmpLt,
	#[token(">")]
	CmpGt,
	#[token("<=")]
	CmpLte,
	#[token(">=")]
	CmpGte,
	#[token("::")]
	DoubleColon,
	#[token(";")]
	SemiColon,

	#[regex(r"/\*([^*]|\*+[^*/])*\*?")]
	#[error]
	Error,
}

impl Display for TokenKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ApplyKw => write!(f, "apply"),
			Self::Arrow => write!(f, "->"),
			Self::BoolKw => write!(f, "bool"),
			Self::CmpEq => write!(f, "=="),
			Self::CmpGt => write!(f, ">"),
			Self::CmpGte => write!(f, ">="),
			Self::CmpLt => write!(f, "<"),
			Self::CmpLte => write!(f, "<="),
			Self::CmpNeq => write!(f, "!="),
			Self::Colon => write!(f, ":"),
			Self::Comma => write!(f, ","),
			Self::Comment => write!(f, "comment"),
			Self::DoubleColon => write!(f, "::"),
			Self::ElseKw => write!(f, "else"),
			Self::Eq => write!(f, "="),
			Self::Error => write!(f, "error"),
			Self::F32Kw => write!(f, "f32"),
			Self::F64Kw => write!(f, "f64"),
			Self::FatArrow => write!(f, "=>"),
			Self::FloatLit => write!(f, "float"),
			Self::FnKw => write!(f, "fn"),
			Self::INKw => write!(f, "iN"),
			Self::Ident => write!(f, "identifier"),
			Self::IfKw => write!(f, "if"),
			Self::IntLit => write!(f, "int"),
			Self::LBrace => write!(f, "{{"),
			Self::LParen => write!(f, "("),
			Self::LetKw => write!(f, "let"),
			Self::Minus => write!(f, "-"),
			Self::ModKw => write!(f, "mod"),
			Self::MutKw => write!(f, "mut"),
			Self::Plus => write!(f, "+"),
			Self::PubKw => write!(f, "pub"),
			Self::RBrace => write!(f, "}}"),
			Self::RParen => write!(f, ")"),
			Self::ReturnKw => write!(f, "return"),
			Self::Root => write!(f, "root"),
			Self::SemiColon => write!(f, ";"),
			Self::Slash => write!(f, "/"),
			Self::Star => write!(f, "*"),
			Self::StructKw => write!(f, "struct"),
			Self::ToKw => write!(f, "to"),
			Self::TraitKw => write!(f, "trait"),
			Self::TypeKw => write!(f, "type"),
			Self::UNKw => write!(f, "uN"),
			Self::UseKw => write!(f, "use"),
			Self::WhereKw => write!(f, "where"),
			Self::Whitespace => write!(f, "whitespace"),
		}
	}
}

impl TokenKind {
	pub fn is_trivia(self) -> bool {
		matches!(self, Self::Whitespace | Self::Comment)
	}
}
