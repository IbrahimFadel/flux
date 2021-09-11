#ifndef IR_H
#define IR_H

#include <llvm-c/Core.h>

#include "pi.h"

typedef struct CodegenContext {
  LLVMContextRef ctx;
  LLVMModuleRef mod;
  LLVMBuilderRef builder;
  LLVMBasicBlockRef cur_bb;
} CodegenContext;

LLVMModuleRef codegen_pkg(Package *pkg);

void codegen_function(CodegenContext *ctx, FnDecl *fn);
LLVMTypeRef codegen_type_expr(CodegenContext *ctx, Expr *expr);
LLVMTypeRef codegen_primitive_type_expr(CodegenContext *ctx, PrimitiveTypeExpr *expr);
void coddegen_block_stmt(CodegenContext *ctx, Stmt *block);
LLVMValueRef codegen_stmt(CodegenContext *ctx, Stmt *stmt);
LLVMValueRef codegen_expr(CodegenContext *ctx, Expr *expr);
LLVMValueRef codegen_basic_lit_expr(CodegenContext *ctx, BasicLitExpr *lit);
LLVMValueRef codegen_binary_expr(CodegenContext *ctx, BinaryExpr *binop);
LLVMValueRef codegen_binop_arithmetic(CodegenContext *ctx, BinaryExpr *binop);
LLVMValueRef codegen_ident_expr(CodegenContext *ctx, IdentExpr *ident);

#endif