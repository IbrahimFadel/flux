#include "ast.h"

#include <stdio.h>
#include <string.h>

sds fn_tostring(FnDecl *fn) {
  sds repr = sdsnew("Fn Decl: {\n");
  repr = sdscatfmt(repr, "Pub: %s\n", (fn->pub ? "true" : "false"));
  repr = sdscatfmt(repr, "Name: %s\n", fn->name);
  repr = sdscat(repr, "Params: {\n");
  unsigned i;
  for (i = 0; i < cvector_size(fn->params); i++) {
    repr = sdscat(repr, param_tostring(fn->params[i]));
  }
  repr = sdscat(repr, "}\n");
  repr = sdscat(repr, "Then: {\n");
  for (i = 0; i < cvector_size(fn->body->stmts); i++) {
    repr = sdscat(repr, stmt_tostring(&fn->body->stmts[i]));
  }
  repr = sdscat(repr, "}\n");
  repr = sdscat(repr, "}\n");
  return repr;
}

sds stmt_tostring(Stmt *stmt) {
  switch (stmt->type) {
    case STMTTYPE_EXPR:
      return expr_tostring(stmt->value.expr);
    default: {
      return sdsnew("ERROR STMT TYPE\n");
    }
  }
}

sds expr_tostring(Expr *expr) {
  switch (expr->type) {
    case EXPRTYPE_IDENT:
      return ident_expr_tostring(expr->value.ident);
    default:
      return sdsnew("ERROR EXPR TYPE\n");
  }
}

sds ident_expr_tostring(IdentExpr *ident) {
  sds repr = sdsnew("Ident Expr: {\n");
  repr = sdscatfmt(repr, "Value: %s\n}\n", ident->value);
  return repr;
}

sds param_tostring(Param *param) {
  sds repr = sdsnew("Param: {\n");
  repr = sdscatfmt(repr, "Mut: %s\n", (param->mut ? "true" : "false"));
  repr = sdscat(repr, "Type: {\n");
  repr = sdscat(repr, type_expr_tostring(param->type));
  repr = sdscat(repr, "}\n");
  repr = sdscat(repr, "}\n");
  return repr;
}

sds type_expr_tostring(Expr *e) {
  switch (e->type) {
    case EXPRTYPE_PRIMITIVE:
      return type_expr_primitive_tostring(e->value.primitive_type);
    case EXPRTYPE_PTR:
      return type_expr_ptr_tostring(e->value.pointer_type);
    default:
      return "ERROR TYPE\n";
  }
}

sds type_expr_ptr_tostring(PointerTypeExpr *ptr) {
  sds repr = sdsnew("Pointer Type Expr: {\n");
  repr = sdscatfmt(repr, "Pointer To: {\n%s}\n}\n", type_expr_tostring(ptr->pointer_to_type));
  return repr;
}

sds type_expr_primitive_tostring(PrimitiveTypeExpr *prim) {
  sds repr = sdsnew("Primitive Type Expr: {\n");
  repr = sdscatfmt(repr, "Token: %u\n", (unsigned)prim->type);
  repr = sdscat(repr, "}\n");
  return repr;
}

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