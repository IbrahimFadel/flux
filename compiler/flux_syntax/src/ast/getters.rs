use super::*;
use crate::getters;

// stupidly unnecessary macro syntax, but... pretty? prettier i guess
getters! {
    Root {
        items -> nodes(Item);
        fn_decls -> nodes(FnDecl);
        struct_decls -> nodes(StructDecl);
        enum_decls -> nodes(EnumDecl);
        trait_decls -> nodes(TraitDecl);
        apply_decls -> nodes(ApplyDecl);
        use_decls -> nodes(UseDecl);
        mod_decls -> nodes(ModDecl);
    }
    Visibility {
        public -> tok(Pub);
    }
    FnDecl {
        fn_kw -> tok(Fn);
        name -> node(Name);
        generic_param_list -> node(GenericParamList);
        where_clause -> node(WhereClause);
        visibility -> node(Visibility);
        param_list -> node(ParamList);
        return_type -> node(FnReturnType);
        body -> node(Expr);
    }
    FnReturnType {
        ty -> node(Type);
    }
    Name {
        ident -> tok(Ident);
    }
    ParamList {
        params -> nodes(Param);
    }
    Param {
        ty -> node(Type);
        name -> node(Name);
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
        generic_arg_list -> node(GenericArgList);
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
        lparen -> tok(LParen);
        rparen -> tok(RParen);
    }
    ArgList {
        args -> nodes(Expr);
    }
    StructExpr {
        path -> node(Path);
        field_list -> node(StructExprFieldList);
    }
    StructExprFieldList {
        fields -> nodes(StructExprField);
    }
    StructExprField {
        name -> node(Name);
        val -> node(Expr);
    }

    LetStmt {
        name -> node(Name);
        ty -> node(Type);
        value -> node(Expr);
    }
    ExprStmt {
        expr -> node(Expr);
    }
    TerminatorExprStmt {
        expr -> node(Expr);
    }
    StructDecl {
        name -> node(Name);
        visibility -> node(Visibility);
        generic_param_list -> node(GenericParamList);
        where_clause -> node(WhereClause);
        field_list -> node(StructDeclFieldList);
    }
    StructDeclFieldList {
        fields -> nodes(StructDeclField);
    }
    StructDeclField {
        name -> node(Name);
        ty -> node(Type);
    }
    GenericParamList {
        type_params -> nodes(TypeParam);
    }
    TypeParam {
        name -> node(Name);
    }
    WhereClause {
        predicates -> nodes(WherePredicate);
    }
    WherePredicate {
        name -> node(Name);
        type_bound_list -> node(TypeBoundList);
    }
    TypeBoundList {
        type_bounds -> nodes(TypeBound);
    }
    TypeBound {
        trait_path -> node(Path);
    }
    GenericArgList {
        args -> nodes(Type);
    }
    TraitDecl {
        visibility -> node(Visibility);
        name -> node(Name);
        generic_param_list -> node(GenericParamList);
        where_clause -> node(WhereClause);
        associated_types -> nodes(TraitAssocTypeDecl);
        method_decls -> nodes(TraitMethodDecl);
        lbrace -> tok(LBrace);
        rbrace -> tok(RBrace);
    }
    TraitAssocTypeDecl {
        name -> node(Name);
    }
    TraitMethodDecl {
        name -> node(Name);
        generic_param_list -> node(GenericParamList);
        param_list -> node(ParamList);
        return_ty -> node(FnReturnType);
        where_clause -> node(WhereClause);
    }
    EnumDecl {
        name -> node(Name);
        generic_param_list -> node(GenericParamList);
        where_clause -> node(WhereClause);
        variants -> nodes(EnumDeclVariant);
    }
    EnumDeclVariant {
        name -> tok(Ident);
        ty -> node(Type);
    }
    ApplyDecl {
        visibility -> node(Visibility);
        apply_kw -> tok(Apply);
        generic_param_list -> node(GenericParamList);
        trt -> node(ApplyDeclTrait);
        to_kw -> tok(To);
        to_ty -> node(ApplyDeclType);
        where_clause -> node(WhereClause);
        associated_types -> nodes(ApplyDeclAssocType);
        methods -> nodes(FnDecl);
        lbrace -> tok(LBrace);
        rbrace -> tok(RBrace);
    }
    ApplyDeclTrait {
        path -> node(Path);
    }
    ApplyDeclType {
        ty -> node(Type);
    }
    ApplyDeclAssocType {
        name -> node(Name);
        eq -> tok(Eq);
        ty -> node(Type);
    }
    UseDecl {
        visibility -> node(Visibility);
        path -> node(Path);
        alias -> node(Name);
    }
    ModDecl {
        visibility -> node(Visibility);
        name -> node(Name);
    }
}
