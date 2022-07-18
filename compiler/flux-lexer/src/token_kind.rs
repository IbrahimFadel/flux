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

impl TokenKind {
	pub fn is_trivia(self) -> bool {
		matches!(self, Self::Whitespace | Self::Comment)
	}
}
