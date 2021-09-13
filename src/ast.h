#ifndef AST_H
#define AST_H

#include <cvec.h>
#include <llvm-c/Core.h>
#include <stdbool.h>

#include "token.h"

typedef enum ExprType {
  EXPRTYPE_PRIMITIVE,
  EXPRTYPE_PTR,
  EXPRTYPE_VOID,
  EXPRTYPE_IDENT,
  EXPRTYPE_BASIC_LIT,
  EXPRTYPE_BINARY,
  EXPRTYPE_INTERFACE,
  EXPRTYPE_STRUCT,
} ExprType;

typedef enum StmtType {
  STMTTYPE_VARDECL,
  STMTTYPE_RETURN,
  STMTTYPE_BLOCK,
} StmtType;

struct Expr;
struct VarDecl;
struct Stmt;
struct Param;

typedef struct IntExpr {
  unsigned bits;
  bool is_signed;
  int value;
} IntExpr;

typedef struct FloatExpr {
  unsigned bits;
  double value;
} FloatExpr;

typedef struct PrimitiveTypeExpr {
  TokenType type;
} PrimitiveTypeExpr;

typedef struct PointerTypeExpr {
  struct Expr *pointer_to_type;
} PointerTypeExpr;

typedef struct IdentExpr {
  const char *value;
} IdentExpr;

typedef struct BasicLitExpr {
  TokenType type;
  union {
    struct IntExpr *int_lit;
    struct FloatExpr *float_lit;
    const char *str_lit;
    char char_lit;
  } value;
} BasicLitExpr;

typedef struct BinaryExpr {
  struct Expr *x;
  TokenType op;
  struct Expr *y;
} BinaryExpr;

typedef struct ReturnStmt {
  struct Expr *v;
} ReturnStmt;

typedef struct Type {
  bool pub;
  const char *name;
  LLVMTypeRef value;
} Type;

typedef struct Variable {
  bool mut;
  const char *name;
  LLVMValueRef ptr;
} Variable;

typedef struct BlockStmt {
  cvector_vector_type(struct Stmt) stmts;
  cvector_vector_type(Variable) variables;
} BlockStmt;

typedef struct Method {
  bool pub;
  const char *name;
  cvector_vector_type(struct Param) params;
  struct Expr *return_type;
} Method;

typedef struct InterfaceTypeExpr {
  cvector_vector_type(Method) methods;
} InterfaceTypeExpr;

typedef struct Property {
  bool pub;
  bool mut;
  struct Expr *type;
  cvector_vector_type(const char *) names;
} Property;

typedef struct StructTypeExpr {
  cvector_vector_type(Property) properties;
  cvector_vector_type(cvector_vector_type(const char *)) interface_method_implementations;
  cvector_vector_type(const char *) interfaces_implemented;
} StructTypeExpr;

typedef struct Expr {
  ExprType type;
  union {
    struct PrimitiveTypeExpr *primitive_type;
    struct PointerTypeExpr *pointer_type;
    struct InterfaceTypeExpr *interface_type;
    struct StructTypeExpr *struct_type;
    struct IdentExpr *ident;
    struct BasicLitExpr *basic_lit;
    struct BinaryExpr *binop;
  } value;
} Expr;

typedef struct Stmt {
  StmtType type;
  union {
    struct VarDecl *var_decl;
    struct ReturnStmt *ret;
    struct BlockStmt *block;
  } value;
} Stmt;

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
  FnReceiver *receiver;
  cvector_vector_type(Param) params;
  Expr *return_type;
  BlockStmt *body;
  cvector_vector_type(Type) implements;
} FnDecl;

typedef struct VarDecl {
  bool pub;
  bool mut;
  Expr *type;
  cvector_vector_type(const char *) names;
  cvector_vector_type(Expr *) values;
} VarDecl;

typedef struct TypeDecl {
  bool pub;
  const char *name;
  Expr *value;
} TypeDecl;

#endif