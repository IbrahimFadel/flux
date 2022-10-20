use super::*;
use crate::getters;

// stupidly unnecessary macro syntax, but... pretty? prettier i guess
getters! {
    Root {
        fn_decls -> nodes(FnDecl);
        struct_decls -> nodes(StructDecl);
        enum_decls -> nodes(EnumDecl);
        trait_decls -> nodes(TraitDecl);
        apply_decls -> nodes(ApplyDecl);
    }
    Visibility {
        public -> tok(Pub);
    }
    FnDecl {
        fn_kw -> tok(Fn);
        name -> node(Name);
        visibility -> node(Visibility);
        param_list -> node(ParamList);
        return_type -> node(Type);
        body -> node(Expr);
    }
    Name {
        ident -> tok(Ident);
    }
    ParamList {
        params -> nodes(Param);
    }
    Param {
        ty -> node(Type);
        name -> tok(Ident);
    }
    PathType {
        path -> node(Path);
    }
    TupleType {
        types -> nodes(Type);
    }
    ArrayType {
        ty -> node(Type);
        n -> node(IntExpr);
    }
    PtrType {
        ty -> node(Type);
    }
    Path {
        segments -> toks(Ident);
    }

    IntExpr {
        v -> tok(IntLit);
    }
    FloatExpr {
        v -> tok(FloatLit);
    }
    PathExpr {
        segments -> toks(Ident);
    }
    BlockExpr {
        stmts -> nodes(Stmt);
    }
    CallExpr {
        path -> node(PathExpr);
        args -> node(ArgList);
    }
    ArgList {
        args -> nodes(Expr);
    }

    LetStmt {
        name -> node(Name);
        ty -> node(Type);
        value -> node(Expr);
    }
    ExprStmt {
        expr -> node(Expr);
        semicolon -> tok(SemiColon);
    }
}
