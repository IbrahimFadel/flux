#include "typecheck.h"

#include <stdio.h>
#include <string.h>

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

void struct_type_insert_implementation(StructTypeExpr *struct_type, TypeDecl *interface, const char *fn_name) {
  /**
   * [
   *   ["Animal", "hello" ],
   *   ["Animal", "othermethod" ],
   *   ["OtherInterface", "foo" ],
   * ]
   */
  const char ***interface_implementations_map = struct_type->interface_method_implementations;
  cvector_vector_type(const char *) pair = NULL;
  cvector_push_back(pair, interface->name);
  cvector_push_back(pair, fn_name);
  cvector_push_back(interface_implementations_map, pair);

  unsigned num_methods_implemented = 0;
  unsigned i;
  for (i = 0; i < cvector_size(interface_implementations_map); i++) {
    const char *stored_interface_name = interface_implementations_map[i][0];
    const char *stored_method_name = interface_implementations_map[i][1];
    printf("HELO: %s/%s\n", stored_interface_name, stored_method_name);
    if (!strcmp(stored_interface_name, interface->name) && !strcmp(stored_method_name, fn_name)) {
      num_methods_implemented++;
    }
  }
  // printf("HELO!!!\n");
  if (num_methods_implemented == cvector_size(interface->value->value.interface_type->methods)) {
    printf("fully implemented!!!\n");
    cvector_push_back(struct_type->interfaces_implemented, interface->name);
  }
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
  printf("typechecking: %s\n", fn->name);

  if (fn->receiver) {
    const char *struct_name = get_type_name(fn->receiver->type);
    unsigned i;
    TypeDecl *ty;
    for (i = 0; i < cvector_size(ctx->pkg->private_types); i++) {
      if (ctx->pkg->private_types[i].name == struct_name) ty = &ctx->pkg->private_types[i];
    }
    for (i = 0; i < cvector_size(ctx->pkg->public_types); i++) {
      if (ctx->pkg->public_types[i].name == struct_name) ty = &ctx->pkg->public_types[i];
    }
    if (ty == NULL) {
      printf("typecheck: receiver references undefined struct\n");
      exit(1);
    }
    cvector_vector_type(TypeDecl *) implements = struct_method_implements_interface(ctx, fn);
    for (i = 0; i < cvector_size(implements); i++) {
      printf("implements interface: %s\n", implements[i]->name);

      // TODO:
      // We're passing a different struct type for each method. need global array of structs
      // better yet, instead of global array of struct_types, just store the map of implementation data in typecheckcontext
      struct_type_insert_implementation(fn->receiver->type->value.struct_type, implements[i], fn->name);
    }
    printf("%s\n", struct_name);
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