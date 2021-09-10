#include "ir.h"

#include <stdio.h>
#include <stdlib.h>

LLVMModuleRef codegen_pkg(Package *pkg) {
  CodegenContext *ctx = malloc(sizeof(CodegenContext));
  ctx->ctx = LLVMContextCreate();
  ctx->mod = LLVMModuleCreateWithNameInContext("test_module", ctx->ctx);
  // unsigned i;
  // for (i = 0; i < pkg->private_functions_len; i++) {
  //   codegen_function(ctx, &pkg->private_functions[i]);
  // }
  return ctx->mod;
}

LLVMTypeRef codegen_type_expr(CodegenContext *ctx, Expr *expr) {
  switch (expr->type) {
    case EXPRTYPE_PRIMITIVE:
      return codegen_primitive_type_expr(ctx, &expr->value.primitive_type);
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

void codegen_function(CodegenContext *ctx, FnDecl *fn) {
  // LLVMTypeRef param_types[fn->params->length];
  // int i;
  // for (i = 0; i < fn->params->length; i++) {
  //   param_types[i] = codegen_type_expr(ctx, fn->params->params[i].type);
  // }
  // LLVMTypeRef ret_type = LLVMFunctionType(codegen_type_expr(ctx, fn->return_type), param_types, fn->params->length, false);
  // LLVMValueRef func = LLVMAddFunction(ctx->mod, fn->name, ret_type);
  // LLVMAppendBasicBlock(func, "entry");
}