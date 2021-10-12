#include "typecheck.h"

#include <stdio.h>
#include <string.h>

TypecheckContext *typecheck_ctx_create(Package *pkg) {
  TypecheckContext *ctx = malloc(sizeof *ctx);
  ctx->pkg = pkg;
  ctx->symbol_table = NULL;
  ctx->interface_method_implementations_map = NULL;
  ctx->struct_implements_interfaces_map = NULL;
  return ctx;
}

void typecheck_ctx_destroy(TypecheckContext *ctx) {
  package_destroy(ctx->pkg);
  cvector_free(ctx->interface_method_implementations_map);
  cvector_free(ctx->struct_implements_interfaces_map);
  free(ctx);
}

void typecheck_ctx_insert_interface_implementation(TypecheckContext *ctx, const char *struct_name, TypeDecl *interface, const char *method_name) {
  char **interface_method_tuple = malloc((sizeof *interface_method_tuple) * 3);

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
    char **full_implementation_pair = malloc((sizeof *full_implementation_pair) * 2);
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
    case TOKTYPE_U64:
    case TOKTYPE_F64:
      return 64;
    case TOKTYPE_I32:
    case TOKTYPE_U32:
    case TOKTYPE_F32:
      return 32;
    case TOKTYPE_I16:
    case TOKTYPE_U16:
      return 16;
    case TOKTYPE_I8:
    case TOKTYPE_U8:
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
    case TOKTYPE_U64:
    case TOKTYPE_U32:
    case TOKTYPE_U16:
    case TOKTYPE_U8:
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
  Param **method_param;
  Param **fn_param;
  if (method->pub != fn->pub) return false;
  if (method->return_type->type != fn->return_type->type) return false;
  for (method_param = cvector_begin(method->params); method_param != cvector_end(method->params); method_param++) {
    for (fn_param = cvector_begin(fn->params); fn_param != cvector_end(fn->params); fn_param++) {
      if ((*method_param)->mut != (*fn_param)->mut) return false;
      if ((*method_param)->type->type != (*fn_param)->type->type) return false;
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
    if (ctx->pkg->private_types[i]->value->type != EXPRTYPE_INTERFACE) continue;
    InterfaceTypeExpr *interface = ctx->pkg->private_types[i]->value->value.interface_type;

    Method **m;
    for (m = cvector_begin(interface->methods); m != cvector_end(interface->methods); m++) {
      if (fn_implements_interface_method(fn, *m)) cvector_push_back(interfaces, ctx->pkg->private_types[i]);
    }
  }

  return interfaces;
}

void typecheck_pkg(TypecheckContext *ctx, Package *pkg) {
  ctx->pkg = pkg;

  unsigned i;
  for (i = 0; i < cvector_size(pkg->private_functions); i++) {
    typecheck_function(ctx, pkg->private_functions[i]);
  }
  for (i = 0; i < cvector_size(pkg->public_functions); i++) {
    typecheck_function(ctx, pkg->public_functions[i]);
  }
}

void coerce_basic_lit_to_type(BasicLitExpr *lit, TokenType ty) {
  switch (lit->type) {
    case TOKTYPE_INT:
      if (ty == TOKTYPE_F64 || ty == TOKTYPE_F32) {
        float v = lit->value.int_lit->value;
        free(lit->value.int_lit);
        lit->type = TOKTYPE_FLOAT;
        lit->value.float_lit = malloc(sizeof *lit->value.float_lit);
        lit->value.float_lit->bits = primitive_type_get_num_bits(ty);
        lit->value.float_lit->value = v;
      }
      lit->value.int_lit->bits = primitive_type_get_num_bits(ty);
      lit->value.int_lit->is_signed = primitive_type_get_signed(ty);
      break;
    case TOKTYPE_FLOAT:
      lit->value.float_lit->bits = primitive_type_get_num_bits(ty);
      break;
    default:
      printf("typecheck: unimplemented basic lit\n");
      exit(1);
  }
}

void typecheck_function(TypecheckContext *ctx, FnDecl *fn) {
  if (fn->receiver) {
    const char *struct_name = get_type_name(fn->receiver->type);
    unsigned i;
    TypeDecl *ty;
    for (i = 0; i < cvector_size(ctx->pkg->private_types); i++) {
      if (ctx->pkg->private_types[i]->name == struct_name) ty = ctx->pkg->private_types[i];
    }
    if (ty == NULL) {
      for (i = 0; i < cvector_size(ctx->pkg->public_types); i++) {
        if (ctx->pkg->public_types[i]->name == struct_name) ty = ctx->pkg->public_types[i];
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

    // need function receiver as a var decl
    Symbol *recv_symbol = malloc(sizeof *recv_symbol);
    recv_symbol->name = fn->receiver->name;
    recv_symbol->type = fn->receiver->type;
    cvector_push_back(ctx->symbol_table, recv_symbol);
  }

  ctx->expecting_type = fn->return_type;
  Stmt *stmt;
  for (stmt = cvector_begin(fn->body->stmts); stmt != cvector_end(fn->body->stmts); stmt++) {
    if (stmt->type == STMTTYPE_RETURN) {
      typecheck_return(ctx, stmt->value.ret);
    } else if (stmt->type == STMTTYPE_VARDECL) {
      typecheck_var_decl(ctx, stmt->value.var_decl);
    } else if (stmt->type == STMTTYPE_EXPR) {
      typecheck_expr(ctx, stmt->value.expr);
    }
  }
  //TODO: free data
  ctx->symbol_table = NULL;
}

void typecheck_var_decl(TypecheckContext *ctx, VarDecl *var) {
  unsigned i;
  if (var->type->type == EXPRTYPE_IDENT) {
    for (i = 0; i < cvector_size(var->names); i++) {
      Symbol *s = malloc(sizeof *s);
      s->name = var->names[i];
      s->type = var->type;
      cvector_push_back(ctx->symbol_table, s);
    }
  }

  ctx->expecting_type = var->type;
  for (i = 0; i < cvector_size(var->values); i++) {
    typecheck_expr(ctx, var->values[i]);
  }
}

void typecheck_expr(TypecheckContext *ctx, Expr *expr) {
  switch (expr->type) {
    case EXPRTYPE_FUNCTION_CALL:
      typecheck_function_call(ctx, expr->value.fn_call);
      break;
    case EXPRTYPE_BINARY:
      typecheck_binop(ctx, expr->value.binop);
      break;
    case EXPRTYPE_BASIC_LIT:
      typecheck_basic_lit(ctx, expr->value.basic_lit);
      break;
    default:
      break;
  }
}

void typecheck_binop(TypecheckContext *ctx, BinaryExpr *binop) {
  if (binop->op == TOKTYPE_EQ) {
    ctx->expecting_type = binop->x;

    if (binop->x->type == EXPRTYPE_BINARY) {
      BinaryExpr *lhs = binop->x->value.binop;
      if (lhs->y->type != EXPRTYPE_IDENT) {
        printf("typecheck: rhs of '->' must be an identifier\n");
        exit(1);
      }
      if (lhs->op == TOKTYPE_ARROW) {
        Expr *ptr_ty = get_struct_prop_type(ctx, binop->x->value.binop);
        if (ptr_ty->type != EXPRTYPE_PTR) {
          printf("typecheck: lhs of '->' must be a pointer\n");
          exit(1);
        }
        Expr *val_ty = ptr_ty->value.pointer_type->pointer_to_type;
        if (val_ty->type != EXPRTYPE_IDENT) {
          printf("typecheck: lhs of '->' must be a pointer to a struct\n");
          exit(1);
        }
        const char *struct_name = val_ty->value.ident->value;
        unsigned i;
        for (i = 0; i < cvector_size(ctx->pkg->public_types); i++) {
          TypeDecl *type_decl = ctx->pkg->public_types[i];
          if (!strcmp(type_decl->name, struct_name)) {
            if (type_decl->value->type == EXPRTYPE_STRUCT) {
              cvector_vector_type(Property) props = type_decl->value->value.struct_type->properties;
              unsigned j;
              for (j = 0; j < cvector_size(props); j++) {
                const char **name = NULL;
                for (name = cvector_begin(props[j].names); name != cvector_end(props[j].names); name++) {
                  if (!strcmp(*name, lhs->y->value.ident->value)) {
                    Expr *rhs_type = props[j].type;
                    ctx->expecting_type = rhs_type;
                  }
                }
              }
            }
          }
        }

      } else if (lhs->op == TOKTYPE_PERIOD) {
        printf("typecheck: TODO\n");
        exit(1);
      } else {
        printf("typecheck: expected lhs of assignment to be variable\n");
        exit(1);
      }
    }
  }

  typecheck_expr(ctx, binop->x);
  typecheck_expr(ctx, binop->y);
}

Expr *get_struct_prop_type(TypecheckContext *ctx, BinaryExpr *binop) {
  switch (binop->x->type) {
    case EXPRTYPE_BINARY:
      printf("kill me\n");
      exit(1);
    case EXPRTYPE_IDENT: {
      unsigned i;
      for (i = 0; i < cvector_size(ctx->symbol_table); i++) {
        if (!strcmp(ctx->symbol_table[i]->name, binop->x->value.ident->value)) return ctx->symbol_table[i]->type;
      }
      break;
    }
    default:
      printf("kill me\n");
      exit(1);
      break;
  }

  return NULL;
}

void typecheck_basic_lit(TypecheckContext *ctx, BasicLitExpr *lit) {
  if (ctx->expecting_type->type != EXPRTYPE_PRIMITIVE) {
    printf("typecheck: expected primitive type");
    exit(1);
  }
  TokenType needed_type = ctx->expecting_type->value.primitive_type->type;
  if (needed_type <= TOKTYPE_TYPES_BEGIN || needed_type >= TOKTYPE_TYPES_END) {
    printf("typecheck: expected primitive type");
    exit(1);
  }

  coerce_basic_lit_to_type(lit, needed_type);
}

void typecheck_function_call(TypecheckContext *ctx, FnCall *call) {
  FnDecl *callee_fn_decl = get_fn_decl_by_callee_expr(ctx, call->callee);
  unsigned call_num_args = cvector_size(call->args);
  unsigned fn_decl_num_args = cvector_size(callee_fn_decl->params);
  if (callee_fn_decl->receiver) fn_decl_num_args--;
  if (call_num_args != fn_decl_num_args) {
    printf("incorrect number of arguments supplied to function call\n");
    exit(1);
  }
  unsigned i;
  for (i = 0; i < call_num_args; i++) {
    unsigned j;
    for (j = 0; j < fn_decl_num_args; j++) {
      Param *p = callee_fn_decl->params[j];
      if (callee_fn_decl->receiver) {
        p = callee_fn_decl->params[j + 1];
      }
      if (p->type->type == EXPRTYPE_PRIMITIVE && call->args[i]->type == EXPRTYPE_BASIC_LIT) {
        TokenType needed_type = p->type->value.primitive_type->type;
        BasicLitExpr *lit = call->args[i]->value.basic_lit;
        coerce_basic_lit_to_type(lit, needed_type);
      }
    }
  }
}

FnDecl *get_fn_decl_by_callee_expr(TypecheckContext *ctx, Expr *callee) {
  switch (callee->type) {
    case EXPRTYPE_IDENT: {
      const char *fn_name = callee->value.ident->value;
      unsigned i;
      for (i = 0; i < cvector_size(ctx->pkg->private_functions); i++) {
        if (!strcmp(ctx->pkg->private_functions[i]->name, fn_name)) return ctx->pkg->private_functions[i];
      }
      for (i = 0; i < cvector_size(ctx->pkg->public_functions); i++) {
        if (!strcmp(ctx->pkg->public_functions[i]->name, fn_name)) return ctx->pkg->public_functions[i];
      }
      printf("unknown function referenced\n");
      exit(1);
    }
    case EXPRTYPE_BINARY: {
      if (callee->value.binop->y->type != EXPRTYPE_IDENT) {
        printf("expected rhs of callee function binop expression to be identifier\n");
        exit(1);
      }
      const char *fn_name = callee->value.binop->y->value.ident->value;
      unsigned i;
      for (i = 0; i < cvector_size(ctx->pkg->private_functions); i++) {
        if (!strcmp(ctx->pkg->private_functions[i]->name, fn_name)) return ctx->pkg->private_functions[i];
      }
      for (i = 0; i < cvector_size(ctx->pkg->public_functions); i++) {
        if (!strcmp(ctx->pkg->public_functions[i]->name, fn_name)) return ctx->pkg->public_functions[i];
      }
      printf("unknown function referenced\n");
      exit(1);
    }
    default:
      printf("typecheck: unimplemented callee expression\n");
      break;
  }
  return NULL;
}

void typecheck_return(TypecheckContext *ctx, ReturnStmt *ret) {
  typecheck_expr(ctx, ret->v);
}