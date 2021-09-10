#include "parser.h"

#include <stdio.h>
#include <stdlib.h>

ParseContext *parsecontext_create(Token *toks) {
  ParseContext *ctx = malloc(sizeof(ParseContext));
  ctx->toks = toks;
  ctx->tok_ptr = 0;
  ctx->cur_tok = ctx->toks[ctx->tok_ptr];
  ctx->functions = NULL;
  return ctx;
}

void parser_fatal(const char *msg) {
  printf("%s\n", msg);
  exit(1);
}

Token parser_eat(ParseContext *ctx) {
  ctx->tok_ptr++;
  ctx->cur_tok = ctx->toks[ctx->tok_ptr];
  return ctx->cur_tok;
}

Token parser_expect(ParseContext *ctx, TokenType type, const char *msg) {
  if (ctx->cur_tok.type != type) {
    parser_fatal(msg);
  }
  Token tok = ctx->cur_tok;
  parser_eat(ctx);
  return tok;
}

Token parser_expect_range(ParseContext *ctx, TokenType begin, TokenType end, const char *msg) {
  if (ctx->cur_tok.type <= begin || ctx->cur_tok.type >= end)
    parser_fatal(msg);
  Token tok = ctx->cur_tok;
  parser_eat(ctx);
  return tok;
}

const char *parse_pkg(ParseContext *ctx) {
  parser_expect(ctx, TOKTYPE_PACKAGE, "expected 'pkg' in package statement");
  return parser_expect(ctx, TOKTYPE_IDENT, "expected identifier following 'pkg'").value;
}

FnDecl *parse_fn_decl(ParseContext *ctx, bool pub) {
  FnDecl *fn = malloc(sizeof(FnDecl));
  parser_expect(ctx, TOKTYPE_FN, "expected 'fn' in function declaration");

  FnReceiver *receiver = NULL;
  if (ctx->cur_tok.type == TOKTYPE_LPAREN) {
    receiver = parse_fn_receiver(ctx);
  }

  fn->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in function name").value;
  printf("%s\n", ctx->cur_tok.value);
  printf("%u\n", ctx->cur_tok.type);
  parser_expect(ctx, TOKTYPE_LPAREN, "expected '(' before function parameter listing");

  fn->params = parse_paramlist(ctx);
  parser_expect(ctx, TOKTYPE_RPAREN, "expected ')' after function parameter listing");

  VoidTypeExpr *voidTy = NULL;
  fn->return_type = (Expr *)voidTy;
  if (ctx->cur_tok.type == TOKTYPE_ARROW) {
    parser_eat(ctx);
    fn->return_type = parse_type_expr(ctx);
  }

  parser_expect(ctx, TOKTYPE_LBRACE, "expected '{' after function parameter listing");
  parser_expect(ctx, TOKTYPE_RBRACE, "expected '}' after function body");

  fn->pub = pub;
  return fn;
}

FnReceiver *parse_fn_receiver(ParseContext *ctx) {
  FnReceiver *recv = malloc(sizeof(FnReceiver));
  parser_expect(ctx, TOKTYPE_LPAREN, "expected '(' in function receiver");
  recv->type = parse_type_expr(ctx);
  recv->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier after receiver type").value;
  parser_expect(ctx, TOKTYPE_RPAREN, "expected ')' in function receiver");
  return recv;
}

cvector_vector_type(Param) parse_paramlist(ParseContext *ctx) {
  cvector_vector_type(Param) paramlist = NULL;
  while (ctx->cur_tok.type != TOKTYPE_RPAREN) {
    Param *p = parse_param(ctx);
    cvector_push_back(paramlist, *p);
    if (ctx->cur_tok.type == TOKTYPE_COMMA) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_RPAREN) {
      parser_fatal("expected ')' at end of param list");
    }
  }
  return paramlist;
}

Param *parse_param(ParseContext *ctx) {
  Param *p = malloc(sizeof(Param));
  p->mut = false;
  if (ctx->cur_tok.type == TOKTYPE_MUT) {
    p->mut = true;
    parser_eat(ctx);
  }
  p->type = parse_type_expr(ctx);
  p->name = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in parameter").value;
  return p;
}

Expr *parse_type_expr(ParseContext *ctx) {
  if (ctx->cur_tok.type > TOKTYPE_TYPES_BEGIN && ctx->cur_tok.type < TOKTYPE_TYPES_END)
    return parse_primitive_type_expr(ctx);
  else
    parser_fatal("unimplemented type expression");
  return NULL;
}

Expr *parse_primitive_type_expr(ParseContext *ctx) {
  Expr *expr = malloc(sizeof(Expr));
  TokenType ty = parser_expect_range(ctx, TOKTYPE_TYPES_BEGIN, TOKTYPE_TYPES_END, "expected a type in primitive type expression").type;

  expr->type = EXPRTYPE_PRIMITIVE;
  expr->value.primitive_type->type = ty;
  if (ctx->cur_tok.type != TOKTYPE_ASTERISK) {
    return (Expr *)expr;
  }

  expr->type = EXPRTYPE_PTR;
  expr->value.pointer_type->pointer_to_type = expr;
  parser_eat(ctx);
  while (ctx->cur_tok.type == TOKTYPE_ASTERISK) {
    expr->value.pointer_type->pointer_to_type = expr;
    parser_eat(ctx);
  }
  return expr;
}

Stmt *parse_stmt(ParseContext *ctx) {
  switch (ctx->cur_tok.type) {
    case TOKTYPE_MUT:
      parser_eat(ctx);
      return parse_var_decl(ctx, false, true);
    default:
      break;
  }
}

VarDecl *parse_var_decl(ParseContext *ctx, bool pub, bool mut) {
  VarDecl *var = malloc(sizeof *var);
  var->pub = pub;
  var->mut = mut;
  var->type = parse_type_expr(ctx);
  var->names = NULL;
  var->values = NULL;
  while (ctx->cur_tok.type != TOKTYPE_EQ && ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
    const char *ident = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in var decl").value;
    cvector_push_back(var->names, ident);

    if (ctx->cur_tok.type == TOKTYPE_COMMA) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_EQ && ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
      parser_fatal("expected '=' or ';' after var decl ident list");
    }
  }

  if (ctx->cur_tok.type == TOKTYPE_SEMICOLON) {
    parser_eat(ctx);
    return var;
  }

  parser_expect(ctx, TOKTYPE_EQ, "expected '=' in var decl");

  int i = 0;
  while (ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
    if (i == cvector_size(var->names)) {
      parser_fatal("too many values in var decl");
    }
    Expr *v = parse_expr(ctx);
    cvector_push_back(var->values, v);

    if (ctx->cur_tok.type == TOKTYPE_SEMICOLON) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
      parser_fatal("expected ';' after var decl value list");
    }
    i++;
  }

  parser_expect(ctx, TOKTYPE_SEMICOLON, "expected ';' in var decl");

  return var;
}

Expr *parse_expr(ParseContext *ctx) {
  return parse_binary_expr(ctx, 1);
}

Expr *parse_binary_expr(ParseContext *ctx, int prec1) {
  Expr *x = parse_unary_expr(ctx);
  while (true) {
    int oprec = parser_get_tokprec(&ctx->cur_tok);
    TokenType op = ctx->cur_tok.type;

    if (oprec < prec1) {
      return x;
    }

    parser_eat(ctx);
    Expr *y = parse_binary_expr(ctx, oprec + 1);
  }
}

Expr *parse_unary_expr(ParseContext *ctx) {
}

void parse_pkg_file_tokens(ParseContext *ctx) {
  if (ctx->cur_tok.type != TOKTYPE_PACKAGE) {
    parser_fatal("first token in file must be package statement");
  }
  int i = 0;
  while (ctx->cur_tok.type != TOKTYPE_EOF) {
    switch (ctx->cur_tok.type) {
      case TOKTYPE_PACKAGE:
        ctx->pkg = parse_pkg(ctx);
        break;
      case TOKTYPE_FN:
        cvector_push_back(ctx->functions, *parse_fn_decl(ctx, false));
        break;
      case TOKTYPE_PUB:
        parser_eat(ctx);
        cvector_push_back(ctx->functions, *parse_fn_decl(ctx, true));
        break;
      default:
        break;
    }
    i++;
  }
}
