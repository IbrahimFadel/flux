#ifndef PARSER_H
#define PARSER_H

#include <c-vector/cvector.h>

struct ParseContext;

#include "pi.h"
#include "token.h"

typedef struct TokPrecKeyVal {
  TokenType type;
  int prec;
} TokPrecKeyVal;

typedef struct ParseContext {
  cvector_vector_type(Token *) toks;
  Token *cur_tok;
  const char *pkg;
  int tok_ptr;
  TokPrecKeyVal tok_precedence_map[15];
  cvector_vector_type(FnDecl *) functions;
  cvector_vector_type(TypeDecl *) types;
  cvector_vector_type(VarDecl *) variables;
  BlockStmt *cur_block;
} ParseContext;

ParseContext *parsecontext_create(cvector_vector_type(Token *) toks);
void parsecontext_destroy(ParseContext *ctx);

void parser_fatal(const char *msg);
Token *parser_eat(ParseContext *ctx);
Token *parser_expect(ParseContext *ctx, TokenType type, const char *msg);
Token *parser_expect_range(ParseContext *ctx, TokenType begin, TokenType end, const char *msg);
int parser_get_tokprec(ParseContext *ctx, TokenType tok);

Expr *ptr_type_make(Expr *to);

const char *parse_pkg(ParseContext *ctx);
FnDecl *parse_fn_decl(ParseContext *ctx, bool pub);
FnReceiver *parse_fn_receiver(ParseContext *ctx);
Expr *parse_type_expr(ParseContext *ctx);
Expr *parse_primitive_type_expr(ParseContext *ctx);
cvector_vector_type(Param *) parse_paramlist(ParseContext *ctx);
Param *parse_param(ParseContext *ctx);
Stmt *parse_stmt(ParseContext *ctx);
Stmt *parse_var_decl(ParseContext *ctx, bool pub, bool mut);
Expr *parse_expr(ParseContext *ctx);
Expr *parse_operand(ParseContext *ctx);
Expr *parse_binary_expr(ParseContext *ctx, int prec1);
Expr *parse_unary_expr(ParseContext *ctx);
Expr *parse_primary_expr(ParseContext *ctx);
Expr *parse_ident_expr(ParseContext *ctx);
Expr *parse_basic_lit(ParseContext *ctx);
BlockStmt *parse_block_stmt(ParseContext *ctx);
Stmt *parse_return_stmt(ParseContext *ctx);
TypeDecl *parse_type_decl(ParseContext *ctx, bool pub);
Expr *parse_interface_type_expr(ParseContext *ctx);
Method *parse_method_decl(ParseContext *ctx);
Expr *parse_struct_type_expr(ParseContext *ctx);
Property *parse_property(ParseContext *ctx);
Expr *parse_postfix_expr(ParseContext *ctx, Expr *x);
Expr *parse_fn_call(ParseContext *ctx, Expr *x);
cvector_vector_type(Expr *) parse_call_args(ParseContext *ctx);
Expr *parse_nil_expr(ParseContext *ctx);
Stmt *parse_if_stmt(ParseContext *ctx);
Expr *parse_indexed_memory_access(ParseContext *ctx, Expr *x);
Expr *parse_prop_access_expr(ParseContext *ctx, Expr *x, bool ptr_access);
Expr *parse_sizeof_expr(ParseContext *ctx);
Expr *parse_type_cast_expr(ParseContext *ctx);
Expr *parse_array_type_expr(ParseContext *ctx);

void parse_pkg_file_tokens(ParseContext *ctx);

#endif