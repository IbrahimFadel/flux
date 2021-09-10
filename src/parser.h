#ifndef PARSER_H
#define PARSER_H

#include <cvec.h>

#include "pi.h"
#include "token.h"

#define REALLOCATION_FACTOR 16

typedef struct ParseContext {
  Token *toks;
  Token cur_tok;
  const char *pkg;
  int tok_ptr;
  cvector_vector_type(FnDecl) functions;
} ParseContext;

ParseContext *parsecontext_create(Token *toks);

void parser_fatal(const char *msg);
Token parser_eat(ParseContext *ctx);
Token parser_expect(ParseContext *ctx, TokenType type, const char *msg);
Token parser_expect_range(ParseContext *ctx, TokenType begin, TokenType end, const char *msg);

const char *parse_pkg(ParseContext *ctx);
FnDecl *parse_fn_decl(ParseContext *ctx, bool pub);
FnReceiver *parse_fn_receiver(ParseContext *ctx);
Expr *parse_type_expr(ParseContext *ctx);
Expr *parse_primitive_type_expr(ParseContext *ctx);
cvector_vector_type(Param) parse_paramlist(ParseContext *ctx);
Param *parse_param(ParseContext *ctx);
Stmt *parse_stmt(ParseContext *ctx);
VarDecl *parse_var_decl(ParseContext *ctx, bool pub, bool mut);
Expr *parse_expr(ParseContext *ctx);
Expr *parse_binary_expr(ParseContext *ctx, int prec1);
Expr *parse_unary_expr(ParseContext *ctx);

void parse_pkg_file_tokens(ParseContext *ctx);

#endif