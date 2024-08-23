use std::fmt::Display;

use flux_diagnostics::ice;
use flux_id::id;
use flux_typesystem::{Type, Typed, WithType};
use flux_util::{Path, Spanned, Word};

use super::StructExprFieldList;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Address(id::Expr),
    // Block(Block),
    BinOp(BinOp),
    Cast(Cast),
    // Enum(EnumExpr),
    // Call(Call),
    // Float(f64),
    Int(u64),
    Tuple(Vec<id::Expr>),
    Path(Path<Word, id::Ty>),
    // Let(Let),
    Struct(StructExpr),
    MemberAccess(MemberAccess),
    If(If),
    Intrinsic,
    // Str(Str),
    Poisoned,
}

impl WithType for Expr {}

impl Expr {
    pub(crate) const fn unit() -> Self {
        Self::Tuple(vec![])
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BinOp {
    pub lhs: id::Expr,
    pub rhs: id::Expr,
    pub op: Spanned<Op>,
}

impl BinOp {
    pub fn new(lhs: id::Expr, rhs: id::Expr, op: Spanned<Op>) -> Self {
        Self { lhs, rhs, op }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Op {
    Eq,
    Add,
    Sub,
    Mul,
    Div,
    CmpAnd,
    CmpEq,
    CmpGt,
    CmpGte,
    CmpLt,
    CmpLte,
    CmpNeq,
    CmpOr,
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Op::*;
        let s = match self {
            Eq => "=",
            Add => "+",
            Sub => "-",
            Mul => "*",
            Div => "/",
            CmpAnd => "&&",
            CmpEq => "==",
            CmpGt => ">",
            CmpGte => ">=",
            CmpLt => "<",
            CmpLte => "<=",
            CmpNeq => "!=",
            CmpOr => "||",
        };
        write!(f, "{s}")
    }
}

impl Op {
    pub fn all() -> [Self; 13] {
        use Op::*;
        [
            Eq, Add, Sub, Mul, Div, CmpAnd, CmpEq, CmpGt, CmpGte, CmpLt, CmpLte, CmpNeq, CmpOr,
        ]
    }

    pub fn binops() -> [Self; 12] {
        use Op::*;
        [
            Add, Sub, Mul, Div, CmpAnd, CmpEq, CmpGt, CmpGte, CmpLt, CmpLte, CmpNeq, CmpOr,
        ]
    }

    pub fn as_trait_name(&self) -> &'static str {
        match self {
            Op::Eq => ice("no trait name associated with `=`"),
            Op::Add => "Add",
            Op::Sub => "Sub",
            Op::Mul => "Mul",
            Op::Div => "Div",
            Op::CmpAnd => "CmpAnd",
            Op::CmpEq => "CmpEq",
            Op::CmpGt => "CmpGt",
            Op::CmpGte => "CmpGte",
            Op::CmpLt => "CmpLt",
            Op::CmpLte => "CmpLte",
            Op::CmpNeq => "CmpNeq",
            Op::CmpOr => "CmpOr",
        }
    }

    pub fn as_trait_method_name(&self) -> &'static str {
        match self {
            Op::Eq => ice("no trait name associated with `=`"),
            Op::Add => "add",
            Op::Sub => "sub",
            Op::Mul => "mul",
            Op::Div => "div",
            Op::CmpAnd => "cmp_and",
            Op::CmpEq => "cmp_eq",
            Op::CmpGt => "cmp_gt",
            Op::CmpGte => "cmp_gte",
            Op::CmpLt => "cmp_lt",
            Op::CmpLte => "cmp_lte",
            Op::CmpNeq => "cmp_neq",
            Op::CmpOr => "cmp_or",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cast {
    pub val: id::Expr,
    pub to_ty: id::Ty,
}

impl Cast {
    pub fn new(val: id::Expr, to_ty: id::Ty) -> Self {
        Self { val, to_ty }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructExpr {
    pub path: Spanned<Path<Word, Type>>,
    pub fields: StructExprFieldList,
}

impl StructExpr {
    pub fn new(path: Spanned<Path<Word, Type>>, fields: StructExprFieldList) -> Self {
        Self { path, fields }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StructExprField {
    pub name: Spanned<Word>,
    pub val: Spanned<id::Expr>,
}

impl StructExprField {
    pub fn new(name: Spanned<Word>, val: Spanned<id::Expr>) -> Self {
        Self { name, val }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MemberAccess {
    pub lhs: id::Expr,
    pub field: Spanned<Word>,
}

impl MemberAccess {
    pub fn new(lhs: id::Expr, field: Spanned<Word>) -> Self {
        Self { lhs, field }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct If {
    exprs: Vec<id::Expr>,
}

impl If {
    pub fn new(
        condition: id::Expr,
        then: id::Expr,
        else_ifs: impl Iterator<Item = (id::Expr, id::Expr)>,
        r#else: Option<id::Expr>,
    ) -> Self {
        Self {
            exprs: [condition, then]
                .into_iter()
                .chain(else_ifs.flat_map(<[_; 2]>::from))
                .chain(r#else)
                .collect(),
        }
    }

    pub fn blocks(&self) -> impl Iterator<Item = &id::Expr> {
        self.exprs.iter().step_by(2)
    }

    #[inline]
    pub fn has_else(&self) -> bool {
        self.exprs.len() % 2 != 0
    }

    pub fn condition(&self) -> &id::Expr {
        &self
            .exprs
            .get(0)
            .unwrap_or_else(|| ice("if expression missing condition expression"))
    }

    pub fn then(&self) -> &id::Expr {
        &self
            .exprs
            .get(1)
            .unwrap_or_else(|| ice("if expression missing then block expression"))
    }

    pub fn else_ifs(&self) -> Option<&[id::Expr]> {
        if self.has_else() {
            self.exprs.get(2..self.exprs.len() - 1)
        } else {
            self.exprs.get(2..)
        }
    }

    pub fn else_block(&self) -> Option<&id::Expr> {
        if self.has_else() {
            self.exprs.last()
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Intrinsic {
    Panic,
    CmpEqU8,
    AddU8,
}
