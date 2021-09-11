#include "ir.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

LLVMModuleRef codegen_pkg(Package *pkg) {
  CodegenContext *ctx = malloc(sizeof(CodegenContext));
  ctx->ctx = LLVMContextCreate();
  ctx->mod = LLVMModuleCreateWithNameInContext(pkg->name, ctx->ctx);
  ctx->builder = LLVMCreateBuilderInContext(ctx->ctx);
  unsigned i;
  for (i = 0; i < cvector_size(pkg->private_functions); i++) {
    codegen_function(ctx, &pkg->private_functions[i]);
  }
  for (i = 0; i < cvector_size(pkg->public_functions); i++) {
    codegen_function(ctx, &pkg->public_functions[i]);
  }
  return ctx->mod;
}

LLVMTypeRef codegen_type_expr(CodegenContext *ctx, Expr *expr) {
  switch (expr->type) {
    case EXPRTYPE_PRIMITIVE:
      return codegen_primitive_type_expr(ctx, expr->value.primitive_type);
    default:
      break;
  }
  printf("unimpemented type expr:  %d\n", expr->type);
  exit(1);
}

LLVMTypeRef codegen_primitive_type_expr(CodegenContext *ctx, PrimitiveTypeExpr *expr) {
  switch (expr->type) {
    case TOKTYPE_i64:
      return LLVMInt64TypeInContext(ctx->ctx);
    case TOKTYPE_i32:
      return LLVMInt32TypeInContext(ctx->ctx);
    case TOKTYPE_i16:
      return LLVMInt16TypeInContext(ctx->ctx);
    case TOKTYPE_i8:
      return LLVMInt8TypeInContext(ctx->ctx);
    case TOKTYPE_u64:
      return LLVMInt64TypeInContext(ctx->ctx);
    case TOKTYPE_u32:
      return LLVMInt32TypeInContext(ctx->ctx);
    case TOKTYPE_u16:
      return LLVMInt16TypeInContext(ctx->ctx);
    case TOKTYPE_u8:
      return LLVMInt8TypeInContext(ctx->ctx);
    default:
      printf("unimpemented primitive type expr: %d\n", expr->type);
      exit(1);
  }
}

void coddegen_block_stmt(CodegenContext *ctx, BlockStmt *block) {
  unsigned i;
  for (i = 0; i < cvector_size(block->stmts); i++) {
    codegen_stmt(ctx, &block->stmts[i]);
  }
}

LLVMValueRef codegen_stmt(CodegenContext *ctx, Stmt *stmt) {
  switch (stmt->type) {
    case STMTTYPE_VARDECL:
      return codegen_var_decl(ctx, stmt->value.var_decl);
    case STMTTYPE_RETURN:
      return LLVMBuildRet(ctx->builder, codegen_expr(ctx, stmt->value.ret->v));
    default:
      printf("unimplemented stmt\n");
      exit(1);
      break;
  }
  return NULL;
}

LLVMValueRef codegen_var_decl(CodegenContext *ctx, VarDecl *var) {
  unsigned i;
  unsigned num_names = cvector_size(var->names);
  unsigned num_vals = cvector_size(var->values);
  bool init_all_to_one_val = false;
  LLVMValueRef val_to_init_vars = NULL;
  if (num_names > num_vals) {
    init_all_to_one_val = true;
    val_to_init_vars = codegen_expr(ctx, var->values[0]);
  }
  for (i = 0; i < num_names; i++) {
    LLVMValueRef ptr = LLVMBuildAlloca(ctx->builder, codegen_type_expr(ctx, var->type), "");
    if (init_all_to_one_val) {
      LLVMBuildStore(ctx->builder, val_to_init_vars, ptr);
    }
    Variable *v = malloc(sizeof *v);
    v->mut = var->mut;
    v->name = var->names[i];
    v->ptr = ptr;
    cvector_push_back(ctx->cur_block->variables, *v);
  }
  return NULL;
}

LLVMValueRef codegen_expr(CodegenContext *ctx, Expr *expr) {
  switch (expr->type) {
    case EXPRTYPE_BASIC_LIT:
      return codegen_basic_lit_expr(ctx, expr->value.basic_lit);
    case EXPRTYPE_BINARY:
      return codegen_binary_expr(ctx, expr->value.binop);
    case EXPRTYPE_IDENT:
      return codegen_ident_expr(ctx, expr->value.ident);
    default:
      printf("unimplemented expr\n");
      exit(1);
  }
  return NULL;
}

LLVMValueRef codegen_ident_expr(CodegenContext *ctx, IdentExpr *ident) {
  LLVMValueRef ptr = block_get_var(ctx->cur_block, ident->value);
  if (ptr == NULL) {
    printf("unknown variable referenced\n");
    exit(1);
  }
  return LLVMBuildLoad(ctx->builder, ptr, "");
}

LLVMValueRef block_get_var(BlockStmt *block, const char *name) {
  Variable *it;
  for (it = cvector_begin(block->variables); it != cvector_end(block->variables); it++) {
    if (!strcmp(it->name, name)) return it->ptr;
  }
  return NULL;
}

LLVMValueRef codegen_binary_expr(CodegenContext *ctx, BinaryExpr *binop) {
  switch (binop->op) {
    case TOKTYPE_PLUS:
    case TOKTYPE_MINUS:
    case TOKTYPE_ASTERISK:
    case TOKTYPE_SLASH:
      return codegen_binop_arithmetic(ctx, binop);
    default:
      break;
  }
  return NULL;
}

// TODO: don't hardcode signed and floats
LLVMValueRef codegen_binop_arithmetic(CodegenContext *ctx, BinaryExpr *binop) {
  LLVMValueRef lhs = codegen_expr(ctx, binop->x);
  LLVMValueRef rhs = codegen_expr(ctx, binop->y);

  switch (binop->op) {
    case TOKTYPE_PLUS:
      return LLVMBuildAdd(ctx->builder, lhs, rhs, "");
    case TOKTYPE_MINUS:
      return LLVMBuildSub(ctx->builder, lhs, rhs, "");
    case TOKTYPE_ASTERISK:
      return LLVMBuildMul(ctx->builder, lhs, rhs, "");
    case TOKTYPE_SLASH:
      return LLVMBuildSDiv(ctx->builder, lhs, rhs, "");
    default:
      break;
  }

  return NULL;
}

// TODO: pi-lang needs a type-checking stage
// TODO: don't hardcode false for signed
LLVMValueRef codegen_basic_lit_expr(CodegenContext *ctx, BasicLitExpr *lit) {
  switch (lit->type) {
    case TOKTYPE_INT:
      return codegen_int_expr(ctx, lit->value.int_lit);
    case TOKTYPE_FLOAT:
      return codegen_float_expr(ctx, lit->value.float_lit);
    default:
      break;
  }
  return NULL;
}

LLVMValueRef codegen_int_expr(CodegenContext *ctx, IntExpr *e) {
  switch (e->bits) {
    case 64:
      return LLVMConstInt(LLVMInt64TypeInContext(ctx->ctx), e->value, e->is_signed);
    case 32:
      return LLVMConstInt(LLVMInt32TypeInContext(ctx->ctx), e->value, e->is_signed);
    default:
      printf("unimplemented integer bit count\n");
      exit(1);
  }
  return NULL;
}

LLVMValueRef codegen_float_expr(CodegenContext *ctx, FloatExpr *e) {
  switch (e->bits) {
    case 64:
      return LLVMConstReal(LLVMDoubleTypeInContext(ctx->ctx), e->value);
    case 32:
      return LLVMConstReal(LLVMFloatTypeInContext(ctx->ctx), e->value);
    default:
      printf("unimplemented float bit count\n");
      exit(1);
  }
  return NULL;
}

LLVMValueRef codegen_function_params(CodegenContext *ctx, LLVMValueRef fn, cvector_vector_type(Param) params) {
  unsigned i;
  for (i = 0; i < cvector_size(params); i++) {
    LLVMValueRef fn_param_val = LLVMGetParam(fn, i);
    LLVMValueRef ptr = LLVMBuildAlloca(ctx->builder, codegen_type_expr(ctx, params[i].type), "");
    LLVMBuildStore(ctx->builder, fn_param_val, ptr);

    Variable *v = malloc(sizeof *v);
    v->mut = params[i].mut;
    v->name = params[i].name;
    v->ptr = ptr;
    cvector_push_back(ctx->cur_block->variables, *v);
  }
  return NULL;
}

void codegen_function(CodegenContext *ctx, FnDecl *fn) {
  unsigned param_len = cvector_size(fn->params);
  LLVMTypeRef param_types[param_len];
  unsigned i;
  for (i = 0; i < param_len; i++) {
    param_types[i] = codegen_type_expr(ctx, fn->params[i].type);
  }
  LLVMTypeRef ret_type = LLVMFunctionType(codegen_type_expr(ctx, fn->return_type), param_types, param_len, false);
  LLVMValueRef func = LLVMAddFunction(ctx->mod, fn->name, ret_type);
  ctx->cur_bb = LLVMAppendBasicBlock(func, "entry");
  ctx->cur_block = fn->body;
  LLVMPositionBuilderAtEnd(ctx->builder, ctx->cur_bb);

  codegen_function_params(ctx, func, fn->params);
  coddegen_block_stmt(ctx, fn->body);
}
