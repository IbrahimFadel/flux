use std::fmt::Display;

use logos::Logos;

#[derive(Debug, Copy, Clone, PartialEq, Logos, Hash, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    #[regex("[ \n\r\t]+")]
    Whitespace,

    #[regex(r"@flux.intrinsics.[a-zA-Z]+[a-zA-Z0-9_]*")]
    Intrinsic,

    #[regex("//.*")]
    #[token("/*", |lex| {
        let len = lex.remainder().find("*/")?;
        lex.bump(len + 2); // include len of `*/`
        Some(())
    })]
    Comment,
    #[regex("[A-Za-z][A-Za-z0-9_]*")]
    Ident,
    #[regex("0x[0-9a-fA-F]+(_[0-9a-fA-F]+)*")]
    #[regex("0b[0-9]+(_[0-9]+)*")]
    #[regex("[0-9]+(_[0-9]+)*")]
    IntLit,

    #[regex(r#""(\\[\\"]|[^"])*""#)]
    StringLit,

    #[regex(r"[0-9]+\.[0-9]+(_[0-9]+)*")]
    FloatLit,

    #[token("This")]
    This,
    #[token("mod")]
    Mod,
    #[token("use")]
    Use,
    #[token("pub")]
    Pub,
    #[token("fn")]
    Fn,
    #[token("type")]
    Type,
    #[token("apply")]
    Apply,
    #[token("to")]
    To,
    #[token("where")]
    Where,
    #[token("is")]
    Is,
    #[token("mut")]
    Mut,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("struct")]
    Struct,
    #[token("trait")]
    Trait,
    #[token("let")]
    Let,
    #[token("return")]
    Return,
    #[token("for")]
    For,
    #[token("in")]
    In,
    #[token("enum")]
    Enum,
    #[token("as")]
    As,

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
    #[token("[")]
    LSquare,
    #[token("]")]
    RSquare,
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
    #[token("&&")]
    CmpAnd,
    #[token("||")]
    CmpOr,
    #[token("::")]
    DoubleColon,
    #[token(";")]
    SemiColon,
    #[token("&")]
    Ampersand,
    #[token(".")]
    Period,

    EOF,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ampersand => write!(f, "&"),
            Self::Apply => write!(f, "apply"),
            Self::Arrow => write!(f, "->"),
            Self::As => write!(f, "as"),
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
            Self::Else => write!(f, "else"),
            Self::Enum => write!(f, "enum"),
            Self::Eq => write!(f, "="),
            Self::FatArrow => write!(f, "=>"),
            Self::FloatLit => write!(f, "float"),
            Self::Fn => write!(f, "fn"),
            Self::For => write!(f, "for"),
            Self::In => write!(f, "in"),
            Self::Ident => write!(f, "identifier"),
            Self::If => write!(f, "if"),
            Self::IntLit => write!(f, "int"),
            Self::Intrinsic => write!(f, "intrinsic"),
            Self::Is => write!(f, "is"),
            Self::LBrace => write!(f, "{{"),
            Self::LParen => write!(f, "("),
            Self::LSquare => write!(f, "["),
            Self::Let => write!(f, "let"),
            Self::Minus => write!(f, "-"),
            Self::Mod => write!(f, "mod"),
            Self::Mut => write!(f, "mut"),
            Self::Period => write!(f, "."),
            Self::Plus => write!(f, "+"),
            Self::Pub => write!(f, "pub"),
            Self::RBrace => write!(f, "}}"),
            Self::RParen => write!(f, ")"),
            Self::RSquare => write!(f, "]"),
            Self::Return => write!(f, "return"),
            Self::SemiColon => write!(f, ";"),
            Self::Slash => write!(f, "/"),
            Self::Star => write!(f, "*"),
            Self::StringLit => write!(f, "string literal"),
            Self::Struct => write!(f, "struct"),
            Self::To => write!(f, "to"),
            Self::Trait => write!(f, "trait"),
            Self::Type => write!(f, "type"),
            Self::Use => write!(f, "use"),
            Self::Where => write!(f, "where"),
            Self::Whitespace => write!(f, "whitespace"),
            Self::This => write!(f, "This"),
            Self::CmpAnd => write!(f, "&&"),
            Self::CmpOr => write!(f, "||"),
            Self::EOF => write!(f, "EOF"),
        }
    }
}

impl TokenKind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }
}
