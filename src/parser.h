#ifndef PARSER_H
#define PARSER_H

#include <cvec.h>

#include "pi.h"
#include "token.h"

typedef struct TokPrecKeyVal {
  TokenType type;
  int prec;
} TokPrecKeyVal;

typedef struct ParseContext {
  Token *toks;
  Token cur_tok;
  const char *pkg;
  int tok_ptr;
  TokPrecKeyVal tok_precedence_map[15];
  cvector_vector_type(FnDecl) functions;
} ParseContext;

ParseContext *parsecontext_create(Token *toks);

void parser_fatal(const char *msg);
Token parser_eat(ParseContext *ctx);
Token parser_expect(ParseContext *ctx, TokenType type, const char *msg);
Token parser_expect_range(ParseContext *ctx, TokenType begin, TokenType end, const char *msg);
int parser_get_tokprec(ParseContext *ctx, TokenType tok);

const char *parse_pkg(ParseContext *ctx);
FnDecl *parse_fn_decl(ParseContext *ctx, bool pub);
FnReceiver *parse_fn_receiver(ParseContext *ctx);
Expr *parse_type_expr(ParseContext *ctx);
Expr *parse_primitive_type_expr(ParseContext *ctx);
cvector_vector_type(Param) parse_paramlist(ParseContext *ctx);
Param *parse_param(ParseContext *ctx);
Stmt *parse_stmt(ParseContext *ctx);
Stmt *parse_var_decl(ParseContext *ctx, bool pub, bool mut);
Expr *parse_expr(ParseContext *ctx);
Expr *parse_binary_expr(ParseContext *ctx, int prec1);
Expr *parse_unary_expr(ParseContext *ctx);
Expr *parse_primary_expr(ParseContext *ctx);
Expr *parse_ident_expr(ParseContext *ctx);
Expr *parse_basic_lit(ParseContext *ctx);
cvector_vector_type(Stmt) parse_block_stmt(ParseContext *ctx);
Stmt *parse_return_stmt(ParseContext *ctx);

void parse_pkg_file_tokens(ParseContext *ctx);

#endif