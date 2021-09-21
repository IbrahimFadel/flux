#include "ir.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

LLVMModuleRef codegen_pkg(TypecheckContext *typecheck_ctx) {
  CodegenContext *ctx = malloc(sizeof(CodegenContext));
  ctx->typecheck_ctx = typecheck_ctx;
  ctx->ctx = LLVMContextCreate();
  ctx->mod = LLVMModuleCreateWithNameInContext(typecheck_ctx->pkg->name, ctx->ctx);
  ctx->builder = LLVMCreateBuilderInContext(ctx->ctx);
  ctx->struct_currently_being_accessed = NULL;
  ctx->interfaces = NULL;
  ctx->structs = NULL;
  unsigned i;
  for (i = 0; i < cvector_size(typecheck_ctx->pkg->private_types); i++) {
    codegen_type_decl(ctx, typecheck_ctx->pkg->private_types[i]);
  }
  for (i = 0; i < cvector_size(typecheck_ctx->pkg->private_functions); i++) {
    codegen_function(ctx, typecheck_ctx->pkg->private_functions[i]);
  }
  for (i = 0; i < cvector_size(typecheck_ctx->pkg->public_functions); i++) {
    codegen_function(ctx, typecheck_ctx->pkg->public_functions[i]);
  }
  return ctx->mod;
}

LLVMValueRef codegen_type_decl(CodegenContext *ctx, TypeDecl *ty) {
  ctx->cur_typedecl_name = ty->name;
  if (ty->value->type == EXPRTYPE_INTERFACE) {
    InterfaceType *t = malloc(sizeof *t);
    t->pub = ty->pub;
    t->name = ty->name;
    t->value = codegen_type_expr(ctx, ty->value);
    cvector_push_back(ctx->interfaces, t);
  } else if (ty->value->type == EXPRTYPE_STRUCT) {
    StructType *t = malloc(sizeof *t);
    t->pub = ty->pub;
    t->name = ty->name;
    t->value = codegen_type_expr(ctx, ty->value);
    cvector_push_back(ctx->structs, t);
    unsigned i;
    for (i = 0; i < cvector_size(ty->value->value.struct_type->properties); i++) {
      cvector_push_back(t->properties, ty->value->value.struct_type->properties[i]);
    }
  } else {
    printf("HUH?? fixme\n");
    exit(1);
  }
  return NULL;
}

LLVMTypeRef codegen_type_expr(CodegenContext *ctx, Expr *expr) {
  switch (expr->type) {
    case EXPRTYPE_PRIMITIVE:
      return codegen_primitive_type_expr(ctx, expr->value.primitive_type);
    case EXPRTYPE_PTR:
      return codegen_ptr_type_expr(ctx, expr->value.pointer_type);
    case EXPRTYPE_VOID:
      return LLVMVoidTypeInContext(ctx->ctx);
    case EXPRTYPE_INTERFACE:
      return codegen_interface_type_expr(ctx, expr->value.interface_type);
    case EXPRTYPE_STRUCT:
      return codegen_struct_type_expr(ctx, expr->value.struct_type);
    case EXPRTYPE_IDENT:
      return codegen_ident_type_expr(ctx, expr->value.ident);
    default:
      printf("unimplemented type expr:  %d\n", expr->type);
      exit(1);
  }
  return NULL;
}

LLVMTypeRef codegen_ptr_type_expr(CodegenContext *ctx, PointerTypeExpr *pointer_type) {
  return LLVMPointerType(codegen_type_expr(ctx, pointer_type->pointer_to_type), 0);
}

LLVMTypeRef codegen_ident_type_expr(CodegenContext *ctx, IdentExpr *ident) {
  unsigned i;
  for (i = 0; i < cvector_size(ctx->interfaces); i++) {
    if (!strcmp(ctx->interfaces[i]->name, ident->value)) return ctx->interfaces[i]->value;
  }
  for (i = 0; i < cvector_size(ctx->structs); i++) {
    if (!strcmp(ctx->structs[i]->name, ident->value)) return ctx->structs[i]->value;
  }
  printf("could not find type with same name as ident expression");
  exit(1);
}

LLVMTypeRef codegen_struct_type_expr(CodegenContext *ctx, StructTypeExpr *s) {
  cvector_vector_type(LLVMTypeRef) prop_types = NULL;
  Property *p_it;
  for (p_it = cvector_begin(s->properties); p_it != cvector_end(s->properties); p_it++) {
    unsigned i;
    for (i = 0; i < cvector_size(p_it->names); i++) {
      cvector_push_back(prop_types, codegen_type_expr(ctx, p_it->type));
    }
  }
  LLVMTypeRef struct_ty = LLVMStructCreateNamed(ctx->ctx, ctx->cur_typedecl_name);
  LLVMStructSetBody(struct_ty, prop_types, cvector_size(prop_types), false);
  return struct_ty;
}

LLVMTypeRef codegen_interface_type_expr(CodegenContext *ctx, InterfaceTypeExpr *interface) {
  LLVMTypeRef vtable_ty = LLVMStructCreateNamed(ctx->ctx, interface_name_to_interface_vtable_name(ctx->cur_typedecl_name));
  LLVMTypeRef interface_ty = LLVMStructCreateNamed(ctx->ctx, ctx->cur_typedecl_name);
  LLVMStructSetBody(interface_ty, &vtable_ty, 1, false);
  return interface_ty;
}

LLVMTypeRef codegen_primitive_type_expr(CodegenContext *ctx, PrimitiveTypeExpr *expr) {
  switch (expr->type) {
    case TOKTYPE_I64:
      return LLVMInt64TypeInContext(ctx->ctx);
    case TOKTYPE_I32:
      return LLVMInt32TypeInContext(ctx->ctx);
    case TOKTYPE_I16:
      return LLVMInt16TypeInContext(ctx->ctx);
    case TOKTYPE_I8:
      return LLVMInt8TypeInContext(ctx->ctx);
    case TOKTYPE_U64:
      return LLVMInt64TypeInContext(ctx->ctx);
    case TOKTYPE_U32:
      return LLVMInt32TypeInContext(ctx->ctx);
    case TOKTYPE_U16:
      return LLVMInt16TypeInContext(ctx->ctx);
    case TOKTYPE_U8:
      return LLVMInt8TypeInContext(ctx->ctx);
    case TOKTYPE_F64:
      return LLVMDoubleTypeInContext(ctx->ctx);
    case TOKTYPE_F32:
      return LLVMFloatTypeInContext(ctx->ctx);
    default:
      printf("unimplemented primitive type expr: %d\n", expr->type);
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
    case STMTTYPE_EXPR:
      return codegen_expr(ctx, stmt->value.expr);
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
  if (num_names > num_vals && var->values != NULL) {
    init_all_to_one_val = true;
    val_to_init_vars = codegen_expr(ctx, var->values[0]);
  }
  for (i = 0; i < num_names; i++) {
    LLVMValueRef ptr = LLVMBuildAlloca(ctx->builder, codegen_type_expr(ctx, var->type), "");
    if (init_all_to_one_val && var->values != NULL) {
      LLVMBuildStore(ctx->builder, val_to_init_vars, ptr);
    } else if (!init_all_to_one_val && var->values != NULL) {
      LLVMBuildStore(ctx->builder, codegen_expr(ctx, var->values[i]), ptr);
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
    case EXPRTYPE_FUNCTION_CALL:
      return codegen_function_call(ctx, expr->value.fn_call);
    default:
      printf("unimplemented expr\n");
      exit(1);
  }
  return NULL;
}

LLVMValueRef codegen_function_call(CodegenContext *ctx, FnCall *call) {
  LLVMValueRef callee;
  switch (call->callee->type) {
    case EXPRTYPE_BINARY:
      callee = codegen_binary_expr(ctx, call->callee->value.binop);
      break;
    default:
      printf("unimplemented function call callee type\n");
      exit(1);
  }

  cvector_vector_type(LLVMValueRef) args = NULL;
  unsigned num_args = 0;
  if (ctx->struct_currently_being_accessed) {
    // Is this GetOperand consistent? monitor this
    cvector_push_back(args, LLVMGetOperand(ctx->struct_currently_being_accessed, 0));
    num_args++;
  }
  unsigned i;
  for (i = 0; i < cvector_size(call->args); i++) {
    cvector_push_back(args, codegen_expr(ctx, call->args[i]));
    num_args++;
  }
  ctx->struct_currently_being_accessed = NULL;

  return LLVMBuildCall2(ctx->builder, LLVMGetReturnType(LLVMTypeOf(callee)), callee, args, num_args, "");
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
    case TOKTYPE_PERIOD:
      return codegen_binop_struct_access(ctx, binop);
    default:
      break;
  }
  return NULL;
}

LLVMValueRef codegen_binop_struct_access(CodegenContext *ctx, BinaryExpr *binop) {
  LLVMValueRef lhs = codegen_expr(ctx, binop->x);

  ctx->struct_currently_being_accessed = lhs;

  if (binop->y->type != EXPRTYPE_IDENT) {
    printf("%d\n", binop->y->type);
    printf("expected identifier in struct property access\n");
    exit(1);
  }
  const char *prop_name = binop->y->value.ident->value;

  unsigned i;
  for (i = 0; i < cvector_size(ctx->structs); i++) {
    LLVMTypeRef struct_ty = ctx->structs[i]->value;
    const char *n = LLVMGetStructName(struct_ty);
    LLVMTypeRef ty = LLVMTypeOf(lhs);
    const char *actual_struct_name = LLVMGetStructName(ty);

    if (!strcmp(n, actual_struct_name)) {
      unsigned j;
      for (j = 0; j < cvector_size(ctx->structs[i]->method_names); j++) {
        const char *method_name = fn_name_to_struct_method_name(prop_name, actual_struct_name);
        if (!strcmp(ctx->structs[i]->method_names[j], method_name)) {
          return LLVMGetNamedFunction(ctx->mod, method_name);
        }
      }
    }
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

LLVMValueRef codegen_function_params(CodegenContext *ctx, LLVMValueRef fn, cvector_vector_type(Param *) params) {
  unsigned i;
  for (i = 0; i < cvector_size(params); i++) {
    LLVMValueRef fn_param_val = LLVMGetParam(fn, i);
    LLVMValueRef ptr = LLVMBuildAlloca(ctx->builder, codegen_type_expr(ctx, params[i]->type), "");
    LLVMBuildStore(ctx->builder, fn_param_val, ptr);

    Variable *v = malloc(sizeof *v);
    v->mut = params[i]->mut;
    v->name = params[i]->name;
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
    param_types[i] = codegen_type_expr(ctx, fn->params[i]->type);
  }
  LLVMTypeRef fn_type = LLVMFunctionType(codegen_type_expr(ctx, fn->return_type), param_types, param_len, false);

  const char *fn_name = fn->name;
  if (fn->receiver != NULL) {
    const char *struct_name = get_type_name(fn->receiver->type);
    fn_name = fn_name_to_struct_method_name(fn->name, struct_name);
    bool added_to_vtable = false;
    for (i = 0; i < cvector_size(ctx->typecheck_ctx->struct_implements_interfaces_map); i++) {
      const char *stored_struct_name = ctx->typecheck_ctx->struct_implements_interfaces_map[i][0];
      const char *stored_interface_name = ctx->typecheck_ctx->struct_implements_interfaces_map[i][1];
      if (!strcmp(stored_struct_name, struct_name)) {
        add_method_to_interface_vtable(ctx, fn_name, fn_type, stored_interface_name);
        added_to_vtable = true;
      }
    }
    if (!added_to_vtable) {
      for (i = 0; i < cvector_size(ctx->structs); i++) {
        if (!strcmp(ctx->structs[i]->name, struct_name)) {
          cvector_push_back(ctx->structs[i]->method_names, fn_name);
        }
      }
    }
  }

  LLVMValueRef func = LLVMAddFunction(ctx->mod, fn_name, fn_type);
  ctx->cur_bb = LLVMAppendBasicBlock(func, "entry");
  ctx->cur_block = fn->body;
  LLVMPositionBuilderAtEnd(ctx->builder, ctx->cur_bb);

  codegen_function_params(ctx, func, fn->params);
  coddegen_block_stmt(ctx, fn->body);
}

const char *fn_name_to_struct_method_name(const char *fn_name, const char *struct_name) {
  char *method_name = malloc(strlen(fn_name) + 1 + strlen(struct_name) + 1);  // add 1 for '\0'
  strcpy(method_name, struct_name);
  strcat(method_name, "_");
  strcat(method_name, fn_name);
  return method_name;
}

const char *interface_name_to_interface_vtable_name(const char *interface_name) {
  // {interface_name}_VTable
  char *vtable_name = malloc(strlen(interface_name) + 1 + 6 + 1);  // add 1 for '\0'
  strcpy(vtable_name, interface_name);
  strcat(vtable_name, "_VTable");
  return vtable_name;
}

/**
 * This confusing mess grabs an interface, looks at the FIRST element for the vtable type
 * Then it copies the vtable type's elements, and makes space for a new one
 * And places a pointer to the struct method in the vtable
 */
void add_method_to_interface_vtable(CodegenContext *ctx, const char *fn_name, LLVMTypeRef fn_type, const char *interface_name) {
  unsigned i;
  for (i = 0; i < cvector_size(ctx->interfaces); i++) {
    InterfaceType *interface = ctx->interfaces[i];
    if (!strcmp(interface->name, interface_name)) {
      LLVMTypeRef vtable_type = LLVMStructGetTypeAtIndex(interface->value, 0);
      unsigned num_vtable_els = LLVMCountStructElementTypes(vtable_type);
      LLVMTypeRef *vtable_el_types = malloc((sizeof *vtable_el_types) * (num_vtable_els + 1));
      LLVMGetStructElementTypes(vtable_type, vtable_el_types);
      vtable_el_types[num_vtable_els] = LLVMPointerType(fn_type, 0);
      LLVMStructSetBody(vtable_type, vtable_el_types, num_vtable_els + 1, false);
    }
  }
}