#ifndef TYPECHECK_H
#define TYPECHECK_H

#include "ast.h"
#include "pi.h"

typedef struct TypecheckContext {
  Package *pkg;
  Expr *expecting_type;
} TypecheckContext;

unsigned primitive_type_get_num_bits(TokenType ty);
bool primitive_type_get_signed(TokenType ty);
const char *get_type_name(Expr *e);
cvector_vector_type(TypeDecl *) struct_method_implements_interface(TypecheckContext *ctx, FnDecl *fn);
bool fn_implements_interface_method(FnDecl *fn, Method *method);
void struct_type_insert_implementation(StructTypeExpr *struct_type, TypeDecl *interface, const char *fn_name);

void typecheck_pkg(TypecheckContext *ctx, Package *pkg);
void typecheck_function(TypecheckContext *ctx, FnDecl *fn);
void typecheck_return(TypecheckContext *ctx, ReturnStmt *ret);

#endif