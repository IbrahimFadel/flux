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

#[macro_export]
macro_rules! T {
	[root] => { $crate::TokenKind::Root };
	[ws] => { $crate::TokenKind::Whitespace };
	[ident] => { $crate::TokenKind::Ident };
	[intlit] => { $crate::TokenKind::IntLit };
	[floatlit] => { $crate::TokenKind::FloatLit };
	[mod] => { $crate::TokenKind::ModKw };
	[use] => { $crate::TokenKind::UseKw };
	[pub] => { $crate::TokenKind::PubKw };
	[fn] => { $crate::TokenKind::FnKw };
	[type] => { $crate::TokenKind::TypeKw };
	[apply] => { $crate::TokenKind::ApplyKw };
	[to] => { $crate::TokenKind::ToKw };
	[mut] => { $crate::TokenKind::MutKw };
	[if] => { $crate::TokenKind::IfKw };
	[else] => { $crate::TokenKind::ElseKw };
	[struct] => { $crate::TokenKind::StructKw };
	[trait] => { $crate::TokenKind::TraitKw };
	[let] => { $crate::TokenKind::LetKw };
	[return] => { $crate::TokenKind::ReturnKw };
	[iN] => { $crate::TokenKind::INKw };
	[uN] => { $crate::TokenKind::UNKw };
	[f64] => { $crate::TokenKind::F64Kw };
	[f32] => { $crate::TokenKind::F32Kw };
	[bool] => { $crate::TokenKind::BoolKw };
	[+] => { $crate::TokenKind::Plus };
	[-] => { $crate::TokenKind::Minus };
	[*] => { $crate::TokenKind::Star };
	[/] => { $crate::TokenKind::Slash };
	[->] => { $crate::TokenKind::Arrow };
	[=>] => { $crate::TokenKind::FatArrow };
	[==] => { $crate::TokenKind::CmpEq };
	[!=] => { $crate::TokenKind::CmpNeq };
	[<] => { $crate::TokenKind::CmpLt };
	[>] => { $crate::TokenKind::CmpGt };
	[<=] => { $crate::TokenKind::CmpLte };
	[>=] => { $crate::TokenKind::CmpGte };
	[:] => { $crate::TokenKind::Colon };
	[::] => { $crate::TokenKind::DoubleColon };
	[comma] => { $crate::TokenKind::Comma };
	[lparen] => { $crate::TokenKind::LParen };
	[rparen] => { $crate::TokenKind::RParen };
	[lbrace] => { $crate::TokenKind::LBrace };
	[rbrace] => { $crate::TokenKind::RBrace };
	[eq] => {$crate::TokenKind::Eq };
	[cmpeq] => {$crate::TokenKind::CmpEq };
	[semicolon] => {$crate::TokenKind::SemiColon };
	[comment] => {$crate::TokenKind::Comment };
	[error] => { $crate::TokenKind::Error };
}

// impl Display for TokenKind {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		write!(
// 			f,
// 			"{}",
// 			match self {
// 				T![root] => "Root",
// 				T![ws] => "WhiteSpace",
// 				T![ident] => "Ident",
// 				T![intlit] => "IntLit",
// 				T![pub] => "Pub",
// 				T![fn] => "Fn",
// 				T![type] => "Type",
// 				T![mut] => "Mut",
// 				T![if] => "If",
// 				T![else] => "Else",
// 				T![iN] => "iN",
// 				T![uN] => "uN",
// 				T![f64] => "F64",
// 				T![f32] => "F32",
// 				T![bool] => "bool",
// 				T![+] => "+",
// 				T![-] => "-",
// 				T![*] => "*",
// 				T![/] => "/",
// 				T![arrow] => "->",
// 				T![comma] => ",",
// 				T![cmplt] => "<",
// 				T![cmpgt] => ">",
// 				T![lparen] => "(",
// 				T![rparen] => ")",
// 				T![lbrace] => "{",
// 				T![rbrace] => "}",
// 				T![eq] => "=",
// 				T![cmpeq] => "==",
// 				T![semicolon] => ";",
// 				T![comment] => "Comment",
// 				T![error] => "Error",
// 			}
// 		)
// 	}
// }
