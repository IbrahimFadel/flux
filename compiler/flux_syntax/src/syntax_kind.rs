use flux_lexer::TokenKind;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum SyntaxKind {
    Root,

    Visibility,
    FnDecl,
    FnReturnType,
    TypeDeclList,
    TypeDecl,
    TraitDecl,
    TraitAssocTypeDecl,
    TraitMethodDecl,
    ApplyDecl,
    ApplyDeclType,
    ApplyDeclTrait,
    ApplyDeclMethodList,
    ApplyDeclMethod,
    ApplyDeclAssocType,
    UseDecl,
    ModDecl,

    GenericParamList,
    TypeParam,
    WhereClause,
    WherePredicate,
    TypeBoundList,
    TypeBound,
    GenericArgList,
    Path,
    ArgList,
    ParamList,
    Param,

    CastExpr,
    BinExpr,
    IntExpr,
    FloatExpr,
    StringExpr,
    BlockExpr,
    AddressExpr,
    DerefExpr,
    CallExpr,
    IdxExpr,
    TupleExpr,
    ParenExpr,
    StructExpr,
    StructExprFieldList,
    StructExprField,
    PathExpr,
    MemberAccessExpr,

    ExprStmt,
    TerminatorExprStmt,

    TupleType,
    PathType,
    StructDecl,
    StructDeclFieldList,
    StructDeclField,
    EnumDecl,
    EnumDeclVariant,
    ArrayType,
    PtrType,

    Whitespace,
    Comment,
    Mod,
    Use,
    Pub,
    Fn,
    Type,
    Apply,
    To,
    Where,
    Is,
    Mut,
    If,
    Else,
    Struct,
    Trait,
    Let,
    Return,
    Ident,
    IntLit,
    FloatLit,
    StringLit,

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
    For,
    In,
    Enum,
    As,
    Name,
    Poisoned,
    LetStmt,
    EOF,
    Error,
}

impl From<TokenKind> for SyntaxKind {
    fn from(token_kind: TokenKind) -> Self {
        match token_kind {
            TokenKind::Whitespace => SyntaxKind::Whitespace,
            TokenKind::Comment => SyntaxKind::Comment,
            TokenKind::Ident => SyntaxKind::Ident,
            TokenKind::IntLit => SyntaxKind::IntLit,
            TokenKind::FloatLit => SyntaxKind::FloatLit,
            TokenKind::Mod => SyntaxKind::Mod,
            TokenKind::Use => SyntaxKind::Use,
            TokenKind::Pub => SyntaxKind::Pub,
            TokenKind::Fn => SyntaxKind::Fn,
            TokenKind::Type => SyntaxKind::Type,
            TokenKind::Apply => SyntaxKind::Apply,
            TokenKind::To => SyntaxKind::To,
            TokenKind::Where => SyntaxKind::Where,
            TokenKind::Is => SyntaxKind::Is,
            TokenKind::Mut => SyntaxKind::Mut,
            TokenKind::If => SyntaxKind::If,
            TokenKind::Else => SyntaxKind::Else,
            TokenKind::Struct => SyntaxKind::Struct,
            TokenKind::Trait => SyntaxKind::Trait,
            TokenKind::Let => SyntaxKind::Let,
            TokenKind::As => SyntaxKind::As,
            TokenKind::Return => SyntaxKind::Return,
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
            TokenKind::LBrace => SyntaxKind::LBrace,
            TokenKind::RBrace => SyntaxKind::RBrace,
            TokenKind::Eq => SyntaxKind::Eq,
            TokenKind::SemiColon => SyntaxKind::SemiColon,
            TokenKind::Intrinsic => SyntaxKind::Intrinsic,
            TokenKind::Ampersand => SyntaxKind::Ampersand,
            TokenKind::Period => SyntaxKind::Period,
            TokenKind::LSquare => SyntaxKind::LSquare,
            TokenKind::RSquare => SyntaxKind::RSquare,
            TokenKind::For => SyntaxKind::For,
            TokenKind::In => SyntaxKind::In,
            TokenKind::Enum => SyntaxKind::Enum,
            TokenKind::StringLit => SyntaxKind::StringLit,
            TokenKind::EOF => SyntaxKind::EOF,
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
