#ifndef AST_H
#define AST_H

#include <c-vector/cvector.h>
#include <llvm-c/Core.h>
#include <sds/sds.h>
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
  EXPRTYPE_FUNCTION_CALL,
  EXPRTYPE_NIL,
  EXPRTYPE_IDX_MEM_ACCESS,
  EXPRTYPE_PROP_ACCESS,
  EXPRTYPE_SIZEOF,
  EXPRTYPE_TYPE_CAST,
} ExprType;

typedef enum StmtType {
  STMTTYPE_VARDECL,
  STMTTYPE_RETURN,
  STMTTYPE_BLOCK,
  STMTTYPE_EXPR,
  STMTTYPE_IF,
} StmtType;

struct Expr;
struct VarDecl;
struct Stmt;
struct Param;
struct IfStmt;
struct IdentExpr;

typedef struct PropAccessExpr {
  struct Expr *x;
  struct IdentExpr *prop;
  bool ptr_access;
} PropAccessExpr;

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

typedef struct Variable {
  bool mut;
  const char *name;
  LLVMValueRef ptr;
} Variable;

typedef struct BlockStmt {
  cvector_vector_type(struct Stmt) stmts;
  cvector_vector_type(Variable) variables;
  struct BlockStmt *parent;
} BlockStmt;

typedef struct Method {
  bool pub;
  const char *name;
  cvector_vector_type(struct Param *) params;
  struct Expr *return_type;
} Method;

typedef struct InterfaceTypeExpr {
  cvector_vector_type(Method *) methods;
} InterfaceTypeExpr;

typedef struct Property {
  bool pub;
  bool mut;
  struct Expr *type;
  cvector_vector_type(const char *) names;
} Property;

typedef struct StructTypeExpr {
  cvector_vector_type(Property) properties;
} StructTypeExpr;

typedef struct FnCall {
  struct Expr *callee;
  cvector_vector_type(struct Expr *) args;
} FnCall;

typedef struct IndexedMemAccess {
  struct Expr *memory;
  struct Expr *index;
} IndexedMemAccess;

typedef struct TypeCastExpr {
  struct Expr *type;
  struct Expr *x;
} TypeCastExpr;

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
    struct FnCall *fn_call;
    struct Expr *nil_type;
    struct IndexedMemAccess *idx_mem_access;
    struct PropAccessExpr *prop_access;
    struct Expr *sizeof_operation;
    struct TypeCastExpr *typecast;
  } value;
} Expr;

typedef struct Stmt {
  StmtType type;
  union {
    struct VarDecl *var_decl;
    struct ReturnStmt *ret;
    struct BlockStmt *block;
    struct Expr *expr;
    struct IfStmt *if_stmt;
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
  cvector_vector_type(Param *) params;
  Expr *return_type;
  BlockStmt *body;
} FnDecl;

typedef struct VarDecl {
  bool pub;
  bool mut;
  Expr *type;
  cvector_vector_type(const char *) names;
  cvector_vector_type(Expr *) values;
} VarDecl;

typedef struct IfStmt {
  Expr *condition;
  BlockStmt *then_block;
  BlockStmt *else_block;
} IfStmt;

typedef struct TypeDecl {
  bool pub;
  const char *name;
  Expr *value;
} TypeDecl;

void fndecl_destroy(FnDecl *fn);
void typedecl_destroy(TypeDecl *ty);
void param_destroy(Param *param);
void blockstmt_destroy(BlockStmt *block);
void stmt_destroy(Stmt *stmt);
void expr_destroy(Expr *expr);
void vardecl_destroy(VarDecl *var);
void retstmt_destroy(ReturnStmt *ret);
void interfacetype_destroy(InterfaceTypeExpr *interface);
void method_destroy(Method *method);
void basic_lit_destroy(BasicLitExpr *lit);
void primitive_type_destroy(PrimitiveTypeExpr *prim);

sds fn_tostring(FnDecl *fn);
sds param_tostring(Param *param);
sds type_expr_tostring(Expr *e);
sds type_expr_primitive_tostring(PrimitiveTypeExpr *prim);
sds type_expr_ptr_tostring(PointerTypeExpr *ptr);
sds stmt_tostring(Stmt *stmt);
sds expr_tostring(Expr *expr);
sds ident_expr_tostring(IdentExpr *ident);

#endif