#ifndef TYPECHECK_H
#define TYPECHECK_H

#include "ast.h"
#include "pi.h"

typedef struct TypecheckContext {
  Package *pkg;
  Expr *expecting_type;

  //TODO: better naming, or document (both probably)
  cvector_vector_type(cvector_vector_type(char *)) interface_method_implementations_map;
  cvector_vector_type(cvector_vector_type(char *)) struct_implements_interfaces_map;
} TypecheckContext;

TypecheckContext *typecheck_ctx_create(Package *pkg);
void typecheck_ctx_insert_interface_implementation(TypecheckContext *ctx, const char *struct_name, TypeDecl *interface, const char *method_name);

unsigned primitive_type_get_num_bits(TokenType ty);
bool primitive_type_get_signed(TokenType ty);
const char *get_type_name(Expr *e);
cvector_vector_type(TypeDecl *) struct_method_implements_interface(TypecheckContext *ctx, FnDecl *fn);
bool fn_implements_interface_method(FnDecl *fn, Method *method);

void typecheck_pkg(TypecheckContext *ctx, Package *pkg);
void typecheck_function(TypecheckContext *ctx, FnDecl *fn);
void typecheck_return(TypecheckContext *ctx, ReturnStmt *ret);

#endif