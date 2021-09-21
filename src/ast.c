#include "ast.h"

#include <stdio.h>

void fndecl_destroy(FnDecl *fn) {
  unsigned i;
  for (i = 0; i < cvector_size(fn->params); i++) {
    param_destroy(fn->params[i]);
  }
  free(fn->receiver);
  blockstmt_destroy(fn->body);
  free(fn->return_type);
  free(fn);
}

void typedecl_destroy(TypeDecl *ty) {
  expr_destroy(ty->value);
  free(ty);
}

void param_destroy(Param *param) {
  free(param->type);
  free(param);
}

void blockstmt_destroy(BlockStmt *block) {
  unsigned i;
  for (i = 0; i < cvector_size(block->stmts); i++) {
    stmt_destroy(&block->stmts[i]);
  }
  cvector_free(block->stmts);
  // cvector_free(block->variables);
}

void stmt_destroy(Stmt *stmt) {
  switch (stmt->type) {
    case STMTTYPE_RETURN:
      retstmt_destroy(stmt->value.ret);
      break;
    case STMTTYPE_VARDECL:
      vardecl_destroy(stmt->value.var_decl);
      break;
    default:
      printf("could not destroy stmt of unknown type\n");
      break;
  }
}

void expr_destroy(Expr *expr) {
  switch (expr->type) {
    case EXPRTYPE_BASIC_LIT: {
      basic_lit_destroy(expr->value.basic_lit);
      break;
    }
    case EXPRTYPE_BINARY:
      expr_destroy(expr->value.binop->x);
      expr_destroy(expr->value.binop->y);
      break;
    case EXPRTYPE_PRIMITIVE:
      primitive_type_destroy(expr->value.primitive_type);
      break;
    case EXPRTYPE_INTERFACE:
    default:
      break;
  }
  free(expr);
}

void primitive_type_destroy(PrimitiveTypeExpr *prim) {
  free(prim);
}

void basic_lit_destroy(BasicLitExpr *lit) {
  switch (lit->type) {
    case TOKTYPE_INT:
      free(lit->value.int_lit);
      break;
    case TOKTYPE_FLOAT:
      free(lit->value.float_lit);
    default:
      printf("could not free basic lit of unknown type\n");
      break;
  }
  free(lit);
}

void retstmt_destroy(ReturnStmt *ret) {
  expr_destroy(ret->v);
  free(ret);
}

void vardecl_destroy(VarDecl *var) {
  unsigned i;
  expr_destroy(var->type);
  cvector_free(var->names);
  for (i = 0; i < cvector_size(var->values); i++) {
    expr_destroy(var->values[i]);
  }
  cvector_free(var->values);
  free(var);
}

void interfacetype_destroy(InterfaceTypeExpr *interface) {
  unsigned i;
  for (i = 0; i < cvector_size(interface->methods); i++) {
    method_destroy(interface->methods[i]);
  }
  free(interface);
}

void method_destroy(Method *method) {
  unsigned i;
  for (i = 0; i < cvector_size(method->params); i++) {
    param_destroy(method->params[i]);
  }
  expr_destroy(method->return_type);
  free(method);
}