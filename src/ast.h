#ifndef AST_H
#define AST_H

#include <cvec.h>
#include <stdbool.h>

#include "token.h"

typedef enum ExprType {
  EXPRTYPE_PRIMITIVE,
  EXPRTYPE_PTR,
} ExprType;

typedef enum StmtType {
  STMTTYPE_VARDECL,
} StmtType;

struct Expr;
struct VarDecl;

typedef struct PrimitiveTypeExpr {
  TokenType type;
} PrimitiveTypeExpr;

typedef struct PointerTypeExpr {
  struct Expr *pointer_to_type;
} PointerTypeExpr;

typedef struct _VoidTypeExpr VoidTypeExpr;

typedef struct Expr {
  ExprType type;
  union {
    struct PrimitiveTypeExpr *primitive_type;
    struct PointerTypeExpr *pointer_type;
  } value;
} Expr;

typedef struct Stmt {
  StmtType type;
  union {
    struct VarDecl *var_decl;
  } value;
} Stmt;

typedef struct BlockStmt {
} BlockStmt;

typedef struct FnReceiver {
  Expr *type;
  const char *name;
} FnReceiver;

typedef struct Param {
  bool mut;
  Expr *type;
  const char *name;
} Param;

typedef struct FnDecl {
  bool pub;
  const char *name;
  cvector_vector_type(Param) params;
  Expr *return_type;
} FnDecl;

typedef struct VarDecl {
  bool pub;
  bool mut;
  Expr *type;
  cvector_vector_type(const char *) names;
  cvector_vector_type(Expr *) values;
} VarDecl;

typedef struct TypeDecl {
  const char *name;
} TypeDecl;

#endif