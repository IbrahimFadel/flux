#ifndef PARSER_H
#define PARSER_H

#include "ast.h"

static std::vector<std::shared_ptr<Token>> toks;
static std::shared_ptr<Token> cur_tok;
static std::shared_ptr<Token> last_tok;
static int tok_pointer = 0;
static std::map<std::string, int> binop_precedence;

void print_nodes(const Nodes &nodes);
Nodes parse_tokens(std::vector<std::shared_ptr<Token>> tokens);
static unique_ptr<Node> parse_token(std::shared_ptr<Token> token);

static unique_ptr<Expression_Node> parse_expression(bool needs_semicolon = true);
static unique_ptr<Expression_Node> parse_primary();
static unique_ptr<Expression_Node> parse_binop_rhs(int expression_precedence, unique_ptr<Expression_Node> lhs);
static unique_ptr<Expression_Node> parse_number_expression();
static unique_ptr<Expression_Node> parse_identifier_expression();
static unique_ptr<Function_Node> parse_fn_declaration();
static unique_ptr<Prototype_Node> parse_fn_prototype();
static unique_ptr<Then_Node> parse_then();
static unique_ptr<Variable_Declaration_Node> parse_variable_declaration();
static unique_ptr<If_Node> parse_if();
static std::tuple<std::vector<std::unique_ptr<Condition_Node>>, std::vector<Token_Type>> parse_condition(Token_Type end_token);
static unique_ptr<Function_Call_Node> parse_function_call_node(std::string name);
static unique_ptr<Import_Node> parse_import();
static unique_ptr<For_Node> parse_for();
static unique_ptr<Variable_Assignment_Node> parse_variable_assignment(std::string name, bool needs_semicolon = true);

static void throw_if_cur_tok_is_type(Token_Type type, const char *msg, int line, int position);
static void throw_if_cur_tok_not_type(Token_Type type, const char *msg, int line, int position);
static Variable_Type token_type_to_variable_type(Token_Type type);
static void get_next_token();
static int get_token_precedence();

#endif