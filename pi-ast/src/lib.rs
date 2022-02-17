use smol_str::SmolStr;

#[derive(Debug)]
pub struct AST {
    pub functions: Vec<FnDecl>,
}

impl AST {
    pub fn new(functions: Vec<FnDecl>) -> AST {
        AST { functions }
    }
}

#[derive(Debug, PartialEq)]

pub enum Decl {
    FnDecl(FnDecl),
    TypeDecl(TypeDecl),
}

#[derive(Debug, PartialEq)]
pub struct FnDecl {
    pub name: SmolStr,
    pub generics: GenericTypes,
    pub params: Vec<FnParam>,
    pub ret_ty: Expr,
    pub block: BlockStmt,
}

#[derive(Debug, PartialEq)]
pub struct FnParam {
    pub mut_: bool,
    pub type_: Expr,
    pub name: SmolStr,
}

impl FnParam {
    pub fn new(mut_: bool, type_: Expr, name: SmolStr) -> FnParam {
        FnParam { mut_, type_, name }
    }
}

impl FnDecl {
    pub fn new(
        name: SmolStr,
        generics: GenericTypes,
        params: Vec<FnParam>,
        ret_ty: Expr,
        block: BlockStmt,
    ) -> FnDecl {
        FnDecl {
            name,
            generics,
            params,
            ret_ty,
            block,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TypeDecl {}

#[derive(Debug, PartialEq, Clone)]
pub enum OpKind {
    Plus,
    Minus,
    Asterisk,
    Slash,
    CmpEq,
    CmpNEq,
    And,
    Or,
    Illegal,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Ident(Ident),
    BinOp(BinOp),
    IntLit(IntLit),
    FloatLit(FloatLit),
    PrimitiveType(PrimitiveType),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinOp {
    x: Box<Expr>,
    op: OpKind,
    y: Box<Expr>,
}

impl BinOp {
    pub fn new(x: Box<Expr>, op: OpKind, y: Box<Expr>) -> BinOp {
        BinOp { x, op, y }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveKind {
    I64,
    U64,
    I32,
    U32,
    I16,
    U6,
    I8,
    U8,
    F64,
    F32,
    Bool,
    Void,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrimitiveType {
    pub kind: PrimitiveKind,
}

impl PrimitiveType {
    pub fn new(kind: PrimitiveKind) -> PrimitiveType {
        PrimitiveType { kind }
    }
}

#[derive(Debug, PartialEq)]
pub enum Stmt {
    BlockStmt(BlockStmt),
    VarDecl(VarDecl),
    If(If),
    For(For),
}

#[derive(Debug, PartialEq)]
pub struct VarDecl {
    pub type_: Expr,
    pub names: Vec<Ident>,
    pub values: Option<Vec<Expr>>,
}

impl VarDecl {
    pub fn new(type_: Expr, names: Vec<Ident>, values: Option<Vec<Expr>>) -> VarDecl {
        VarDecl {
            type_,
            names,
            values,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct If {
    pub condition: Expr,
    pub then: BlockStmt,
}

impl If {
    pub fn new(condition: Expr, then: BlockStmt) -> If {
        If { condition, then }
    }
}

#[derive(Debug, PartialEq)]
pub struct For {}

impl For {
    pub fn new() -> For {
        For {}
    }
}

pub type Ident = SmolStr;
pub type IntLit = i128;
pub type FloatLit = f64;
pub type GenericTypes = Vec<SmolStr>;
pub type BlockStmt = Vec<Stmt>;
