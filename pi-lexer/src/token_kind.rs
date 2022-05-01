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
	#[regex(r"(?s)/\*(.*)\*/")]
	Comment,
	// #[regex("/\*")]
	// BlockComment,
	#[regex("[A-Za-z][A-Za-z0-9_]*")]
	Ident,
	#[regex("[0-9]")]
	IntLit,

	#[token("pub")]
	PubKw,
	#[token("fn")]
	FnKw,
	#[token("mut")]
	MutKw,
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
	#[token("<")]
	CmpLt,
	#[token(">")]
	CmpGt,
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
	#[token(";")]
	SemiColon,

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
	[pub] => { $crate::TokenKind::PubKw };
	[fn] => { $crate::TokenKind::FnKw };
	[mut] => { $crate::TokenKind::MutKw };
	[iN] => { $crate::TokenKind::INKw };
	[uN] => { $crate::TokenKind::UNKw };
	[f64] => { $crate::TokenKind::F64Kw };
	[f32] => { $crate::TokenKind::F32Kw };
	[bool] => { $crate::TokenKind::BoolKw };
	[+] => { $crate::TokenKind::Plus };
	[-] => { $crate::TokenKind::Minus };
	[*] => { $crate::TokenKind::Star };
	[/] => { $crate::TokenKind::Slash };
	[arrow] => { $crate::TokenKind::Arrow };
	[comma] => { $crate::TokenKind::Comma };
	[cmplt] => { $crate::TokenKind::CmpLt };
	[cmpgt] => { $crate::TokenKind::CmpGt };
	[lparen] => { $crate::TokenKind::LParen };
	[rparen] => { $crate::TokenKind::RParen };
	[lbrace] => { $crate::TokenKind::LBrace };
	[rbrace] => { $crate::TokenKind::RBrace };
	[eq] => {$crate::TokenKind::Eq };
	[semicolon] => {$crate::TokenKind::SemiColon };
	[comment] => {$crate::TokenKind::Comment };
	[error] => { $crate::TokenKind::Error };
}

impl Display for TokenKind {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				T![root] => "Root",
				T![ws] => "WhiteSpace",
				T![ident] => "Ident",
				T![intlit] => "IntLit",
				T![pub] => "Pub",
				T![fn] => "Fn",
				T![mut] => "Mut",
				T![iN] => "iN",
				T![uN] => "uN",
				T![f64] => "F64",
				T![f32] => "F32",
				T![bool] => "bool",
				T![+] => "+",
				T![-] => "-",
				T![*] => "*",
				T![/] => "/",
				T![arrow] => "->",
				T![comma] => ",",
				T![cmplt] => "<",
				T![cmpgt] => ">",
				T![lparen] => "(",
				T![rparen] => ")",
				T![lbrace] => "{",
				T![rbrace] => "}",
				T![eq] => "=",
				T![semicolon] => ";",
				T![comment] => "Comment",
				T![error] => "Error",
			}
		)
	}
}
