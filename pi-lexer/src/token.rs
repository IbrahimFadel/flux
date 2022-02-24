use codespan::Span;

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new() -> Token {
        Token {
            kind: TokenKind::Illegal,
            span: Span::new(0, 0),
        }
    }

    pub fn from(kind: TokenKind, span: Span) -> Token {
        Token { kind, span }
    }
}

#[derive(Debug)]
pub enum TokenKind {
    LineComment,
    BlockComment,
    Whitespace,
    Illegal,

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

    Int,
    Float,

    Ident,
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
