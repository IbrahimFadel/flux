#ifndef IR_H
#define IR_H

#include <llvm-c/Core.h>

#include "pi.h"
#include "typecheck.h"

typedef struct CodegenContext {
  Package *pkg;
  LLVMContextRef ctx;
  LLVMModuleRef mod;
  LLVMBuilderRef builder;
  LLVMBasicBlockRef cur_bb;
  BlockStmt *cur_block;

  cvector_vector_type(Type) structs;
  cvector_vector_type(Type) interfaces;
} CodegenContext;

LLVMModuleRef codegen_pkg(Package *pkg);
LLVMValueRef block_get_var(BlockStmt *block, const char *name);
const char *fn_name_to_struct_method_name(const char *fn_name, const char *struct_name);

void codegen_function(CodegenContext *ctx, FnDecl *fn);
LLVMTypeRef codegen_type_expr(CodegenContext *ctx, Expr *expr);
LLVMTypeRef codegen_primitive_type_expr(CodegenContext *ctx, PrimitiveTypeExpr *expr);
void coddegen_block_stmt(CodegenContext *ctx, BlockStmt *block);
LLVMValueRef codegen_stmt(CodegenContext *ctx, Stmt *stmt);
LLVMValueRef codegen_expr(CodegenContext *ctx, Expr *expr);
LLVMValueRef codegen_basic_lit_expr(CodegenContext *ctx, BasicLitExpr *lit);
LLVMValueRef codegen_binary_expr(CodegenContext *ctx, BinaryExpr *binop);
LLVMValueRef codegen_binop_arithmetic(CodegenContext *ctx, BinaryExpr *binop);
LLVMValueRef codegen_ident_expr(CodegenContext *ctx, IdentExpr *ident);
LLVMValueRef codegen_int_expr(CodegenContext *ctx, IntExpr *e);
LLVMValueRef codegen_float_expr(CodegenContext *ctx, FloatExpr *e);
LLVMValueRef codegen_var_decl(CodegenContext *ctx, VarDecl *var);
LLVMValueRef codegen_function_params(CodegenContext *ctx, LLVMValueRef fn, cvector_vector_type(Param) params);
LLVMValueRef codegen_type_decl(CodegenContext *ctx, TypeDecl *ty);
LLVMTypeRef codegen_interface_type_expr(CodegenContext *ctx, InterfaceTypeExpr *interface);
LLVMTypeRef codegen_struct_type_expr(CodegenContext *ctx, StructTypeExpr *s);
LLVMTypeRef codegen_ident_type_expr(CodegenContext *ctx, IdentExpr *ident);

#endif