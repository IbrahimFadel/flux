#include "parser.h"

#include <stdio.h>
#include <stdlib.h>

ParseContext *parsecontext_create(Token *toks) {
  ParseContext *ctx = malloc(sizeof(ParseContext));
  ctx->toks = toks;
  ctx->tok_ptr = 0;
  ctx->cur_tok = ctx->toks[ctx->tok_ptr];
  ctx->functions = NULL;
  // there is a better way im just stupid
  ctx->tok_precedence_map[0].type = TOKTYPE_EQ;
  ctx->tok_precedence_map[0].prec = 2;
  ctx->tok_precedence_map[1].type = TOKTYPE_CMP_AND;
  ctx->tok_precedence_map[1].prec = 3;
  ctx->tok_precedence_map[2].type = TOKTYPE_CMP_OR;
  ctx->tok_precedence_map[2].prec = 5;
  ctx->tok_precedence_map[3].type = TOKTYPE_CMP_LT;
  ctx->tok_precedence_map[3].prec = 10;
  ctx->tok_precedence_map[4].type = TOKTYPE_CMP_GT;
  ctx->tok_precedence_map[4].prec = 10;
  ctx->tok_precedence_map[5].type = TOKTYPE_CMP_LTE;
  ctx->tok_precedence_map[5].prec = 10;
  ctx->tok_precedence_map[6].type = TOKTYPE_CMP_GTE;
  ctx->tok_precedence_map[6].prec = 10;
  ctx->tok_precedence_map[7].type = TOKTYPE_CMP_EQ;
  ctx->tok_precedence_map[7].prec = 10;
  ctx->tok_precedence_map[8].type = TOKTYPE_CMP_NEQ;
  ctx->tok_precedence_map[8].prec = 10;
  ctx->tok_precedence_map[9].type = TOKTYPE_PLUS;
  ctx->tok_precedence_map[9].prec = 20;
  ctx->tok_precedence_map[10].type = TOKTYPE_MINUS;
  ctx->tok_precedence_map[10].prec = 20;
  ctx->tok_precedence_map[11].type = TOKTYPE_ASTERISK;
  ctx->tok_precedence_map[11].prec = 40;
  ctx->tok_precedence_map[12].type = TOKTYPE_SLASH;
  ctx->tok_precedence_map[12].prec = 40;
  ctx->tok_precedence_map[13].type = TOKTYPE_PERIOD;
  ctx->tok_precedence_map[13].prec = 50;
  ctx->tok_precedence_map[14].type = TOKTYPE_ARROW;
  ctx->tok_precedence_map[14].prec = 50;
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

int parser_get_tokprec(ParseContext *ctx, TokenType tok) {
  int i;
  for (i = 0; i < sizeof(ctx->tok_precedence_map) / sizeof(ctx->tok_precedence_map[0]); i++) {
    if (tok == ctx->tok_precedence_map[i].type) return ctx->tok_precedence_map[i].prec;
  }
  return -1;
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
  parser_expect(ctx, TOKTYPE_LPAREN, "expected '(' before function parameter listing");

  fn->params = parse_paramlist(ctx);
  parser_expect(ctx, TOKTYPE_RPAREN, "expected ')' after function parameter listing");

  fn->return_type = malloc(sizeof *fn->return_type);
  fn->return_type->type = EXPRTYPE_VOID;
  if (ctx->cur_tok.type == TOKTYPE_ARROW) {
    parser_eat(ctx);
    fn->return_type = parse_type_expr(ctx);
  }

  fn->body = parse_block_stmt(ctx);

  fn->pub = pub;
  return fn;
}

cvector_vector_type(Stmt) parse_block_stmt(ParseContext *ctx) {
  parser_expect(ctx, TOKTYPE_LBRACE, "expected '{' after function parameter listing");
  cvector_vector_type(Stmt) block = NULL;
  while (ctx->cur_tok.type != TOKTYPE_RBRACE)
    cvector_push_back(block, *parse_stmt(ctx));
  parser_expect(ctx, TOKTYPE_RBRACE, "expected '}' after function body");
  return block;
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
  expr->value.primitive_type = malloc(sizeof *expr->value.primitive_type);
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
  if (ctx->cur_tok.type > TOKTYPE_TYPES_BEGIN && ctx->cur_tok.type < TOKTYPE_TYPES_END)
    return parse_var_decl(ctx, false, false);
  switch (ctx->cur_tok.type) {
    case TOKTYPE_MUT:
      parser_eat(ctx);
      return parse_var_decl(ctx, false, true);
    case TOKTYPE_RETURN:
      return parse_return_stmt(ctx);
    default:
      parser_fatal("unknown token when parsing statement");
      break;
  }
  return NULL;
}

Stmt *parse_return_stmt(ParseContext *ctx) {
  parser_expect(ctx, TOKTYPE_RETURN, "expected 'return' in return statement");
  Stmt *ret = malloc(sizeof *ret);
  ret->type = STMTTYPE_RETURN;

  ret->value.ret = malloc(sizeof *ret->value.ret);
  ret->value.ret->v = malloc(sizeof *ret->value.ret);
  ret->value.ret->v->type = EXPRTYPE_VOID;
  if (ctx->cur_tok.type != TOKTYPE_SEMICOLON)
    ret->value.ret->v = parse_expr(ctx);

  parser_expect(ctx, TOKTYPE_SEMICOLON, "expected ';' after return value");

  return ret;
}

Stmt *parse_var_decl(ParseContext *ctx, bool pub, bool mut) {
  Stmt *var = malloc(sizeof *var);
  var->type = STMTTYPE_VARDECL;
  var->value.var_decl = malloc(sizeof *var->value.var_decl);
  var->value.var_decl->pub = pub;
  var->value.var_decl->mut = mut;
  var->value.var_decl->type = parse_type_expr(ctx);
  var->value.var_decl->names = NULL;
  var->value.var_decl->values = NULL;
  while (ctx->cur_tok.type != TOKTYPE_EQ && ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
    const char *ident = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier in var decl").value;
    cvector_push_back(var->value.var_decl->names, ident);

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
    if (i + 1 > cvector_size(var->value.var_decl->names)) {
      parser_fatal("too many values in var decl");
    }
    Expr *v = parse_expr(ctx);
    cvector_push_back(var->value.var_decl->values, v);

    if (ctx->cur_tok.type == TOKTYPE_COMMA) {
      parser_eat(ctx);
    } else if (ctx->cur_tok.type != TOKTYPE_SEMICOLON) {
      parser_fatal("expected ';' after var decl value list");
    }
    i++;
  }

  if (i < cvector_size(var->value.var_decl->names) && i != 1) {
    parser_fatal("too few values in var decl");
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
    int oprec = parser_get_tokprec(ctx, ctx->cur_tok.type);
    TokenType op = ctx->cur_tok.type;

    if (oprec < prec1) {
      return x;
    }

    parser_eat(ctx);
    Expr *y = parse_binary_expr(ctx, oprec + 1);

    Expr *binop = malloc(sizeof *binop);
    binop->type = EXPRTYPE_BINARY;
    binop->value.binop = malloc(sizeof *binop->value.binop);
    binop->value.binop->x = x;
    binop->value.binop->op = op;
    binop->value.binop->y = y;
    x = binop;
  }
}

Expr *parse_unary_expr(ParseContext *ctx) {
  switch (ctx->cur_tok.type) {
    case TOKTYPE_AMPERSAND:
      parser_fatal("unimplemented unary expression");
      break;
    default:
      return parse_primary_expr(ctx);
  }
  return NULL;
}

Expr *parse_primary_expr(ParseContext *ctx) {
  switch (ctx->cur_tok.type) {
    case TOKTYPE_IDENT:
      return parse_ident_expr(ctx);
    case TOKTYPE_INT:
    case TOKTYPE_FLOAT:
    case TOKTYPE_STRING_LIT:
    case TOKTYPE_CHAR_LIT:
      return parse_basic_lit(ctx);
    default:
      parser_fatal("unimplemented primary expr");
      break;
  }
  return NULL;
}

Expr *parse_ident_expr(ParseContext *ctx) {
  const char *value = parser_expect(ctx, TOKTYPE_IDENT, "expected identifier").value;
  Expr *ident = malloc(sizeof *ident);
  ident->type = EXPRTYPE_IDENT;
  ident->value.ident = malloc(sizeof *ident->value.ident);
  ident->value.ident->value = value;
  return ident;
}

Expr *parse_basic_lit(ParseContext *ctx) {
  Token tok = parser_expect_range(ctx, TOKTYPE_BASIC_LIT_BEGIN, TOKTYPE_BASIC_LIT_END, "expected literal");
  Expr *lit = malloc(sizeof *lit);
  lit->type = EXPRTYPE_BASIC_LIT;
  lit->value.basic_lit = malloc(sizeof *lit->value.basic_lit);
  lit->value.basic_lit->type = tok.type;
  lit->value.basic_lit->value = tok.value;
  return lit;
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
