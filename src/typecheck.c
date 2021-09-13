#include "typecheck.h"

#include <stdio.h>

unsigned primitive_type_get_num_bits(TokenType ty) {
  switch (ty) {
    case TOKTYPE_I64:
    case TOKTYPE_u64:
    case TOKTYPE_F64:
      return 64;
    case TOKTYPE_I32:
    case TOKTYPE_u32:
    case TOKTYPE_F32:
      return 32;
    case TOKTYPE_I16:
    case TOKTYPE_u16:
      return 16;
    case TOKTYPE_I8:
    case TOKTYPE_u8:
      return 8;
    default:
      printf("could not get primitive type bits\n");
      exit(1);
  }
  return -1;
}

bool primitive_type_get_signed(TokenType ty) {
  switch (ty) {
    case TOKTYPE_I64:
    case TOKTYPE_I32:
    case TOKTYPE_I16:
    case TOKTYPE_I8:
    case TOKTYPE_F64:
    case TOKTYPE_F32:
      return true;
    case TOKTYPE_u64:
    case TOKTYPE_u32:
    case TOKTYPE_u16:
    case TOKTYPE_u8:
      return false;
    default:
      printf("could not determine if primitive type is signed\n");
      exit(1);
  }
  return true;
}

void typecheck_function(TypecheckContext *ctx, FnDecl *fn) {
  printf("typechecking: %s\n", fn->name);

  Stmt *stmt;
  for (stmt = cvector_begin(fn->body->stmts); stmt != cvector_end(fn->body->stmts); stmt++) {
    if (stmt->type == STMTTYPE_RETURN) {
      ctx->expecting_type = fn->return_type;
      typecheck_return(ctx, stmt->value.ret);
    }
  }
}

void typecheck_return(TypecheckContext *ctx, ReturnStmt *ret) {
  if (ret->v->type == EXPRTYPE_BASIC_LIT) {
    if (ctx->expecting_type->type != EXPRTYPE_PRIMITIVE) {
      printf("typecheck: expected return types to match");
      exit(1);
    }
    TokenType primitive_ty = ctx->expecting_type->value.primitive_type->type;
    if (primitive_ty <= TOKTYPE_TYPES_BEGIN || primitive_ty >= TOKTYPE_TYPES_END) {
      printf("typecheck: expected return types to match");
      exit(1);
    }

    switch (ret->v->value.basic_lit->type) {
      case TOKTYPE_INT:
        ret->v->value.basic_lit->value.int_lit->bits = primitive_type_get_num_bits(primitive_ty);
        ret->v->value.basic_lit->value.int_lit->is_signed = primitive_type_get_signed(primitive_ty);
        break;
      case TOKTYPE_FLOAT:
        ret->v->value.basic_lit->value.float_lit->bits = primitive_type_get_num_bits(primitive_ty);
        break;
      default:
        printf("typecheck: unimplemented basic lit\n");
        exit(1);
    }
  }
}