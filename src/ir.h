#ifndef IR_H
#define IR_H

#include <llvm-c/Core.h>

#include "pi.h"

typedef struct CodegenContext {
  LLVMContextRef ctx;
  LLVMModuleRef mod;
} CodegenContext;

LLVMModuleRef codegen_pkg(Package *pkg);

void codegen_function(CodegenContext *ctx, FnDecl *fn);
LLVMTypeRef codegen_type_expr(CodegenContext *ctx, Expr *expr);
LLVMTypeRef codegen_primitive_type_expr(CodegenContext *ctx, PrimitiveTypeExpr *expr);

#endif