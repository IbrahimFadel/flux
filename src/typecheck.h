#ifndef TYPECHECK_H
#define TYPECHECK_H

#include "ast.h"

typedef struct TypecheckContext {
  Expr *expecting_type;
} TypecheckContext;

unsigned primitive_type_get_num_bits(TokenType ty);
bool primitive_type_get_signed(TokenType ty);

void typecheck_function(TypecheckContext *ctx, FnDecl *fn);
void typecheck_return(TypecheckContext *ctx, ReturnStmt *ret);

#endif