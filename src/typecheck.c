#include "typecheck.h"

#include <stdio.h>
#include <string.h>

TypecheckContext *typecheck_ctx_create(Package *pkg) {
  TypecheckContext *ctx = malloc(sizeof *ctx);
  ctx->pkg = pkg;
  ctx->interface_method_implementations_map = NULL;
  ctx->struct_implements_interfaces_map = NULL;
  return ctx;
}

void typecheck_ctx_insert_interface_implementation(TypecheckContext *ctx, const char *struct_name, TypeDecl *interface, const char *method_name) {
  char **interface_method_tuple = malloc(3);

  unsigned struct_name_len = strlen(struct_name);
  unsigned interface_name_len = strlen(interface->name);
  unsigned method_name_len = strlen(method_name);

  interface_method_tuple[0] = malloc(struct_name_len + 1);
  strcpy(interface_method_tuple[0], struct_name);
  interface_method_tuple[1] = malloc(interface_name_len + 1);
  strcpy(interface_method_tuple[1], interface->name);
  interface_method_tuple[2] = malloc(method_name_len + 1);
  strcpy(interface_method_tuple[2], method_name);
  cvector_push_back(ctx->interface_method_implementations_map, interface_method_tuple);

  unsigned num_methods_implemented = 0;
  unsigned i;
  for (i = 0; i < cvector_size(ctx->interface_method_implementations_map); i++) {
    if (!strcmp(ctx->interface_method_implementations_map[i][0], struct_name)) {
      if (!strcmp(ctx->interface_method_implementations_map[i][1], interface->name)) {
        num_methods_implemented++;
      }
    }
  }

  if (num_methods_implemented == cvector_size(interface->value->value.interface_type->methods)) {
    char **full_implementation_pair = malloc(2);
    full_implementation_pair[0] = malloc(struct_name_len + 1);
    strcpy(full_implementation_pair[0], struct_name);
    full_implementation_pair[1] = malloc(interface_name_len + 1);
    strcpy(full_implementation_pair[1], interface->name);
    cvector_push_back(ctx->struct_implements_interfaces_map, full_implementation_pair);
  }
}

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

const char *get_type_name(Expr *e) {
  if (e->type == EXPRTYPE_IDENT) return e->value.ident->value;
  if (e->type != EXPRTYPE_PTR) {
    printf("could not get type name: it wasn't a ident or ptr to ident\n");
    exit(1);
  }
  int i = 0;
  Expr expr = *e;
  while (expr.type != EXPRTYPE_IDENT) {
    i++;
    expr = *expr.value.pointer_type->pointer_to_type;
    if (i > 1) {
      printf("typecheck: function receiver can take struct as value or pointer\n");
      exit(1);
    }
  }
  return expr.value.ident->value;
}

bool fn_implements_interface_method(FnDecl *fn, Method *method) {
  Param *method_param;
  Param *fn_param;
  if (method->pub != fn->pub) return false;
  if (method->return_type->type != fn->return_type->type) return false;
  for (method_param = cvector_begin(method->params); method_param != cvector_end(method->params); method_param++) {
    for (fn_param = cvector_begin(fn->params); fn_param != cvector_end(fn->params); fn_param++) {
      if (method_param->mut != fn_param->mut) return false;
      if (method_param->type->type != fn_param->type->type) return false;
    }
  }
  return true;
}

/**
 * Returns a list of interfaces that have methods that this function implements
 */
cvector_vector_type(TypeDecl *) struct_method_implements_interface(TypecheckContext *ctx, FnDecl *fn) {
  cvector_vector_type(TypeDecl *) interfaces = NULL;

  unsigned i;
  for (i = 0; i < cvector_size(ctx->pkg->private_types); i++) {
    if (ctx->pkg->private_types[i].value->type != EXPRTYPE_INTERFACE) continue;
    InterfaceTypeExpr *interface = ctx->pkg->private_types[i].value->value.interface_type;

    Method *m;
    for (m = cvector_begin(interface->methods); m != cvector_end(interface->methods); m++) {
      if (fn_implements_interface_method(fn, m)) cvector_push_back(interfaces, &ctx->pkg->private_types[i]);
    }
  }

  return interfaces;
}

void typecheck_pkg(TypecheckContext *ctx, Package *pkg) {
  ctx->pkg = pkg;

  unsigned i;
  for (i = 0; i < cvector_size(pkg->private_functions); i++) {
    typecheck_function(ctx, &pkg->private_functions[i]);
  }
  for (i = 0; i < cvector_size(pkg->public_functions); i++) {
    typecheck_function(ctx, &pkg->public_functions[i]);
  }
}

void typecheck_function(TypecheckContext *ctx, FnDecl *fn) {
  if (fn->receiver) {
    const char *struct_name = get_type_name(fn->receiver->type);
    unsigned i;
    TypeDecl *ty;
    for (i = 0; i < cvector_size(ctx->pkg->private_types); i++) {
      if (ctx->pkg->private_types[i].name == struct_name) ty = &ctx->pkg->private_types[i];
    }
    if (ty == NULL) {
      for (i = 0; i < cvector_size(ctx->pkg->public_types); i++) {
        if (ctx->pkg->public_types[i].name == struct_name) ty = &ctx->pkg->public_types[i];
      }
    }
    if (ty == NULL) {
      printf("typecheck: receiver references undefined struct\n");
      exit(1);
    }
    cvector_vector_type(TypeDecl *) implements = struct_method_implements_interface(ctx, fn);
    for (i = 0; i < cvector_size(implements); i++) {
      typecheck_ctx_insert_interface_implementation(ctx, struct_name, implements[i], fn->name);
    }
  }

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