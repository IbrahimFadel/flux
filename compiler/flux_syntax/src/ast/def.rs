use super::*;
use crate::{basic_node, enum_node};

basic_node!(Root);

basic_node!(Poisoned);

basic_node!(FnDecl);
basic_node!(FnReturnType);
basic_node!(StructDecl);
basic_node!(StructDeclFieldList);
basic_node!(StructDeclField);
basic_node!(EnumDecl);
basic_node!(EnumDeclVariant);
basic_node!(TraitDecl);
basic_node!(ApplyDecl);
basic_node!(ApplyDeclTrait);
basic_node!(ApplyDeclType);
basic_node!(ApplyDeclAssocType);
basic_node!(UseDecl);
basic_node!(ModDecl);

enum_node!(
    Item: ApplyDecl,
    EnumDecl,
    FnDecl,
    ModDecl,
    StructDecl,
    TraitDecl,
    UseDecl
);

basic_node!(PathExpr);
basic_node!(ParenExpr);
basic_node!(FloatExpr);
basic_node!(IntExpr);
basic_node!(BinExpr);
basic_node!(CallExpr);
basic_node!(StructExpr);
basic_node!(StructExprFieldList);
basic_node!(StructExprField);
basic_node!(BlockExpr);
basic_node!(TupleExpr);
basic_node!(AddressExpr);
basic_node!(IdxExpr);
basic_node!(MemberAccessExpr);

basic_node!(LetStmt);
basic_node!(ExprStmt);
basic_node!(TerminatorExprStmt);

enum_node!(
    Expr: PathExpr,
    ParenExpr,
    FloatExpr,
    IntExpr,
    BinExpr,
    CallExpr,
    StructExpr,
    BlockExpr,
    TupleExpr,
    AddressExpr,
    IdxExpr,
    MemberAccessExpr
);

enum_node!(Type: PathType, ThisPathType, TupleType, ArrayType, PtrType);
basic_node!(PathType);
basic_node!(ThisPathType);
basic_node!(TupleType);
basic_node!(ArrayType);
basic_node!(PtrType);

enum_node!(Stmt: LetStmt, ExprStmt, TerminatorExprStmt);

basic_node!(Path);
basic_node!(Name);
basic_node!(ArgList);
basic_node!(Visibility);
basic_node!(ParamList);
basic_node!(Param);
basic_node!(GenericParamList);
basic_node!(TypeParam);
basic_node!(WhereClause);
basic_node!(WherePredicate);
basic_node!(TypeBoundList);
basic_node!(TypeBound);
basic_node!(GenericArgList);
basic_node!(TraitAssocTypeDecl);
basic_node!(TraitMethodDecl);

// basic_node!(Visibility);
// basic_node!(FnDecl);
// basic_node!(ParamList);
// basic_node!(Param);
// basic_node!(TypeDeclList);
// basic_node!(TypeDecl);
// basic_node!(ApplyDecl);
// basic_node!(TraitDecl);

// basic_node!(PathExpr);
// basic_node!(ParenExpr);
// basic_node!(FloatExpr);
// basic_node!(IntExpr);
// basic_node!(BinExpr);
// basic_node!(CallExpr);
// basic_node!(StructExpr);
// basic_node!(StructExprFieldList);
// basic_node!(StructExprField);
// basic_node!(BlockExpr);
// basic_node!(TupleExpr);
// basic_node!(AddressExpr);
// basic_node!(IdxExpr);

// basic_node!(PathType);
// basic_node!(TupleType);
// basic_node!(ArrayType);
// basic_node!(PtrType);

// enum_node!(
//     Expr: PathExpr,
//     ParenExpr,
//     FloatExpr,
//     IntExpr,
//     BinExpr,
//     CallExpr,
//     StructExpr,
//     BlockExpr,
//     TupleExpr,
//     AddressExpr,
//     IdxExpr
// );

// enum_node!(Type: PathType, TupleType, ArrayType, PtrType);

// basic_node!(Path);
